#include "porta_connect.h"

#include <atomic>
#include <cstring>
#include <exception>
#include <map>
#include <memory>
#include <mutex>
#include <string>

#include "BellUtils.h"

#include "BellHTTPServer.h"
#include "BellTask.h"
#include "CSpotContext.h"
#include "LoginBlob.h"
#include "MDNSService.h"
#include "SpircHandler.h"
#include "TrackPlayer.h"
#include "civetweb.h"
#include "esp_err.h"
#include "esp_heap_caps.h"
#include "esp_log.h"
#include "mdns.h"

static const char *TAG = "porta_connect";

static constexpr size_t RING_BYTES = 256 * 1024;
static constexpr int KIND_PLAY = 1;
static constexpr int KIND_PAUSE = 2;
static constexpr int KIND_NEXT = 3;
static constexpr int KIND_PREV = 4;
static constexpr int KIND_VOLUME = 5;

static std::mutex ring_mu;
static int16_t *ring = nullptr;
static size_t ring_cap = 0;
static size_t ring_r = 0;
static size_t ring_w = 0;
static size_t ring_n = 0;

static std::mutex title_mu;
static char title[48];

static std::atomic<int> logged_in{0};
static std::atomic<int> pause_edge{-1};
static std::atomic<int> flush_req{0};
static std::atomic<int> want_play{-1};
static std::atomic<int> want_skip{0};
static std::atomic<int> want_volume{-1};
static std::shared_ptr<cspot::SpircHandler> handler;
static std::mutex handler_mu;

static void ring_init() {
    if (ring != nullptr) {
        return;
    }
    ring = static_cast<int16_t *>(
        heap_caps_malloc(RING_BYTES, MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT));
    if (ring == nullptr) {
        ESP_LOGE(TAG, "pcm ring alloc failed");
        return;
    }
    ring_cap = RING_BYTES / sizeof(int16_t);
}

static void ring_write(const int16_t *src, size_t n) {
    if (ring == nullptr || n == 0) {
        return;
    }
    std::lock_guard<std::mutex> lock(ring_mu);
    for (size_t i = 0; i < n; i++) {
        if (ring_n == ring_cap) {
            return;
        }
        ring[ring_w] = src[i];
        ring_w = (ring_w + 1) % ring_cap;
        ring_n++;
    }
}

static void ring_clear() {
    std::lock_guard<std::mutex> lock(ring_mu);
    ring_r = 0;
    ring_w = 0;
    ring_n = 0;
}

static bool replace_json_string(std::string &json, const char *key, const char *value) {
    auto at = json.find(std::string("\"") + key + "\"");
    if (at == std::string::npos) {
        return false;
    }
    auto i = at + std::strlen(key) + 2;
    while (i < json.size() && (json[i] == ' ' || json[i] == '\t')) {
        i++;
    }
    if (i >= json.size() || json[i] != ':') {
        return false;
    }
    i++;
    while (i < json.size() && (json[i] == ' ' || json[i] == '\t')) {
        i++;
    }
    if (i >= json.size() || json[i] != '"') {
        return false;
    }
    auto start = i + 1;
    auto end = json.find('"', start);
    if (end == std::string::npos) {
        return false;
    }
    json.replace(start, end - start, value);
    return true;
}

static std::string zeroconf_info(const std::shared_ptr<cspot::LoginBlob> &blob) {
    std::string info = blob->buildZeroconfInfo();
    replace_json_string(info, "deviceType", "SPEAKER");
    if (info.find("\"spotifyError\"") == std::string::npos) {
        auto open = info.find('{');
        if (open != std::string::npos) {
            info.insert(open + 1, "\"spotifyError\":0,");
        }
    }
    return info;
}

static constexpr long BODY_MAX = 12 * 1024;

static bool read_body(struct mg_connection *conn, long length, std::string &body) {
    if (length <= 0 || length > BODY_MAX) {
        return false;
    }
    body.assign(static_cast<size_t>(length), '\0');
    size_t got = 0;
    while (got < body.size()) {
        int n = mg_read(conn, body.data() + got, body.size() - got);
        if (n <= 0) {
            return false;
        }
        got += static_cast<size_t>(n);
    }
    return true;
}

static bool login_form(const std::map<std::string, std::string> &query) {
    auto filled = [&](const char *key) {
        auto it = query.find(key);
        return it != query.end() && !it->second.empty();
    };
    return filled("userName") && filled("blob") && filled("clientKey");
}

static void set_title(const std::string &name) {
    std::lock_guard<std::mutex> lock(title_mu);
    std::memset(title, 0, sizeof(title));
    std::strncpy(title, name.c_str(), sizeof(title) - 1);
}

static void apply_commands() {
    std::shared_ptr<cspot::SpircHandler> current;
    {
        std::lock_guard<std::mutex> lock(handler_mu);
        current = handler;
    }
    if (!current) {
        return;
    }
    int play = want_play.exchange(-1);
    if (play == 0) {
        current->setPause(true);
    } else if (play == 1) {
        current->setPause(false);
    }
    int skip = want_skip.exchange(0);
    if (skip < 0) {
        current->previousSong();
    } else if (skip > 0) {
        current->nextSong();
    }
    int volume = want_volume.exchange(-1);
    if (volume >= 0) {
        uint32_t full = static_cast<uint32_t>(volume) * 65535u / 100u;
        current->setRemoteVolume(static_cast<int>(full));
    }
}

class PortaConnect : public bell::Task {
 public:
    PortaConnect(std::string name, std::string hostname, std::string client_id,
                 std::string client_secret)
        : bell::Task("cspot", 32 * 1024, 0, 1),
          name(std::move(name)),
          hostname(std::move(hostname)),
          client_id(std::move(client_id)),
          client_secret(std::move(client_secret)) {
        startTask();
    }

    void runTask() override {
        mdns_init();
        // mdns_service_add refuses the service while the hostname is still
        // null. Bell does not check that error (P-0065).
        esp_err_t host = mdns_hostname_set(hostname.c_str());
        if (host != ESP_OK) {
            ESP_LOGE(TAG, "mdns hostname failed %s", esp_err_to_name(host));
        }
        esp_err_t inst = mdns_instance_name_set(name.c_str());
        if (inst != ESP_OK) {
            ESP_LOGE(TAG, "mdns instance failed %s", esp_err_to_name(inst));
        }
        auto blob = std::make_shared<cspot::LoginBlob>(name);
        std::atomic<bool> got_blob{false};
        auto server = std::make_unique<bell::BellHTTPServer>(8080);
        server->registerGet("/spotify_info", [&](struct mg_connection *) {
            return server->makeJsonResponse(zeroconf_info(blob));
        });
        server->registerPost(
            "/spotify_info", [&](struct mg_connection *conn) {
                auto info = mg_get_request_info(conn);
                std::string body;
                if (info == nullptr || !read_body(conn, info->content_length, body)) {
                    return server->makeJsonResponse(
                        "{\"status\":101,\"spotifyError\":1,\"statusString\":\"ERROR-BODY\"}");
                }
                mg_header fields[16];
                int count = mg_split_form_urlencoded(body.data(), fields, 16);
                std::map<std::string, std::string> query;
                for (int i = 0; i < count; i++) {
                    query[fields[i].name] = fields[i].value;
                }
                if (!login_form(query)) {
                    ESP_LOGW(TAG, "spotify login form incomplete");
                    return server->makeJsonResponse(
                        "{\"status\":101,\"spotifyError\":1,\"statusString\":\"ERROR-FORM\"}");
                }
                if (got_blob.load()) {
                    return server->makeJsonResponse(
                        "{\"status\":101,\"spotifyError\":1,\"statusString\":\"ERROR-BUSY\"}");
                }
                blob->loadZeroconfQuery(query);
                got_blob = true;
                return server->makeJsonResponse(
                    "{\"status\":101,\"spotifyError\":0,\"statusString\":\"ERROR-OK\"}");
            });
        auto mdns = bell::MDNSService::registerService(
            blob->getDeviceName(), "_spotify-connect", "_tcp", "", 8080,
            {{"VERSION", "1.0"}, {"CPath", "/spotify_info"}, {"Stack", "SP"}});
        if (!mdns_service_exists_with_instance(name.c_str(), "_spotify-connect", "_tcp",
                                               nullptr)) {
            ESP_LOGE(TAG, "spotify-connect was not announced");
        } else {
            ESP_LOGI(TAG, "spotify-connect announced as %s", name.c_str());
        }
        while (true) {
            ESP_LOGI(TAG, "waiting for a spotify app");
            while (!got_blob) {
                BELL_SLEEP_MS(200);
            }
            ESP_LOGI(TAG, "got credentials");
            std::shared_ptr<cspot::Context> ctx;
            try {
                auto created = cspot::Context::createFromBlob(blob);
                created->config.clientId = client_id;
                created->config.clientSecret = client_secret;
                created->session->connectWithRandomAp();
                auto token = created->session->authenticate(blob);
                if (token.empty()) {
                    ESP_LOGE(TAG, "spotify auth failed");
                } else {
                    ctx = created;
                }
            } catch (const std::exception &ex) {
                ESP_LOGE(TAG, "spotify session failed: %s", ex.what());
            }
            if (ctx) {
                try {
                    ctx->session->startTask();
                    auto created = std::make_shared<cspot::SpircHandler>(ctx);
                    created->subscribeToMercury();
                    created->getTrackPlayer()->setDataCallback(
                        [](uint8_t *data, size_t bytes, std::string_view) {
                            size_t samples = bytes / sizeof(int16_t);
                            ring_write(reinterpret_cast<int16_t *>(data), samples);
                            return bytes;
                        });
                    created->setEventHandler(
                        [](std::unique_ptr<cspot::SpircHandler::Event> event) {
                            using cspot::SpircHandler;
                            switch (event->eventType) {
                            case SpircHandler::EventType::TRACK_INFO:
                                if (auto *info =
                                        std::get_if<cspot::TrackInfo>(&event->data)) {
                                    set_title(info->name);
                                }
                                break;
                            case SpircHandler::EventType::PLAY_PAUSE:
                                if (auto *paused = std::get_if<bool>(&event->data)) {
                                    pause_edge.store(*paused ? 1 : 0);
                                    ESP_LOGI(TAG, "playback %s",
                                             *paused ? "paused" : "playing");
                                    if (*paused) {
                                        ring_clear();
                                    }
                                }
                                break;
                            case SpircHandler::EventType::DISC:
                                pause_edge.store(1);
                                ring_clear();
                                ESP_LOGI(TAG, "playback paused");
                                break;
                            case SpircHandler::EventType::FLUSH:
                                flush_req.store(1);
                                ring_clear();
                                break;
                            default:
                                break;
                            }
                        });
                    {
                        std::lock_guard<std::mutex> lock(handler_mu);
                        handler = created;
                    }
                    logged_in = 1;
                    while (true) {
                        apply_commands();
                        ctx->session->handlePacket();
                    }
                } catch (const std::exception &ex) {
                    ESP_LOGE(TAG, "spotify session failed: %s", ex.what());
                }
            }
            got_blob = false;
            logged_in = 0;
            {
                std::lock_guard<std::mutex> lock(handler_mu);
                handler.reset();
            }
        }
    }

 private:
    std::string name;
    std::string hostname;
    std::string client_id;
    std::string client_secret;
};

static PortaConnect *started = nullptr;

extern "C" void porta_wchar_keep(void);

extern "C" void porta_connect_start(const char *name, const char *hostname,
                                    const char *client_id, const char *client_secret) {
    porta_wchar_keep();
    if (started != nullptr || name == nullptr || hostname == nullptr ||
        client_id == nullptr || client_secret == nullptr) {
        return;
    }
    ring_init();
    bell::setDefaultLogger();
    started = new PortaConnect(name, hostname, client_id, client_secret);
}

extern "C" int porta_connect_logged_in(void) { return logged_in.load(); }

extern "C" void porta_connect_command(int kind, int volume) {
    if (kind == KIND_PLAY) {
        want_play = 1;
    } else if (kind == KIND_PAUSE) {
        want_play = 0;
    } else if (kind == KIND_NEXT) {
        want_skip = 1;
    } else if (kind == KIND_PREV) {
        want_skip = -1;
    } else if (kind == KIND_VOLUME) {
        if (volume < 0) {
            volume = 0;
        } else if (volume > 100) {
            volume = 100;
        }
        want_volume = volume;
    }
}

extern "C" size_t porta_connect_take_pcm(int16_t *dst, size_t frames) {
    if (dst == nullptr || ring == nullptr || frames == 0) {
        return 0;
    }
    std::lock_guard<std::mutex> lock(ring_mu);
    size_t samples = frames * 2;
    if (samples > ring_n) {
        samples = ring_n - (ring_n % 2);
    }
    for (size_t i = 0; i < samples; i++) {
        dst[i] = ring[ring_r];
        ring_r = (ring_r + 1) % ring_cap;
    }
    ring_n -= samples;
    return samples / 2;
}

extern "C" int porta_connect_take_pause(void) { return pause_edge.exchange(-1); }

extern "C" int porta_connect_take_flush(void) { return flush_req.exchange(0); }

extern "C" size_t porta_connect_take_title(char *dst, size_t cap) {
    if (dst == nullptr || cap == 0) {
        return 0;
    }
    std::lock_guard<std::mutex> lock(title_mu);
    std::strncpy(dst, title, cap - 1);
    dst[cap - 1] = '\0';
    return std::strlen(dst);
}
