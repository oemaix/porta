// Copyright 2026 oemaix
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys;
use esp_idf_svc::wifi::{
    AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi, PmfConfiguration,
};
use heapless::String;

use crate::local_env::{self, BenchEnv};

/// The IDF default from `WIFI_INIT_CONFIG_DEFAULT`. esp-idf-svc 0.53 omits
/// `wifi_task_stack_size`, and a zero stack fails task create as `ESP_ERR_NO_MEM`
/// while internal RAM is still free (P-0062).
fn init_driver() -> Result<(), sys::EspError> {
    let cfg = sys::wifi_init_config_t {
        osi_funcs: core::ptr::addr_of_mut!(sys::g_wifi_osi_funcs),
        wpa_crypto_funcs: unsafe { sys::g_wifi_default_wpa_crypto_funcs },
        static_rx_buf_num: sys::CONFIG_ESP_WIFI_STATIC_RX_BUFFER_NUM as _,
        dynamic_rx_buf_num: sys::CONFIG_ESP_WIFI_DYNAMIC_RX_BUFFER_NUM as _,
        tx_buf_type: sys::CONFIG_ESP_WIFI_TX_BUFFER_TYPE as _,
        static_tx_buf_num: sys::WIFI_STATIC_TX_BUFFER_NUM as _,
        dynamic_tx_buf_num: sys::WIFI_DYNAMIC_TX_BUFFER_NUM as _,
        rx_mgmt_buf_type: sys::CONFIG_ESP_WIFI_DYNAMIC_RX_MGMT_BUF as _,
        rx_mgmt_buf_num: sys::WIFI_RX_MGMT_BUF_NUM_DEF as _,
        cache_tx_buf_num: sys::WIFI_CACHE_TX_BUFFER_NUM as _,
        csi_enable: sys::WIFI_CSI_ENABLED as _,
        ampdu_rx_enable: sys::WIFI_AMPDU_RX_ENABLED as _,
        ampdu_tx_enable: sys::WIFI_AMPDU_TX_ENABLED as _,
        amsdu_tx_enable: sys::WIFI_AMSDU_TX_ENABLED as _,
        nvs_enable: sys::WIFI_NVS_ENABLED as _,
        nano_enable: sys::WIFI_NANO_FORMAT_ENABLED as _,
        rx_ba_win: sys::WIFI_DEFAULT_RX_BA_WIN as _,
        wifi_task_core_id: sys::WIFI_TASK_CORE_ID as _,
        beacon_max_len: sys::WIFI_SOFTAP_BEACON_MAX_LEN as _,
        mgmt_sbuf_num: sys::WIFI_MGMT_SBUF_NUM as _,
        feature_caps: u64::from(sys::WIFI_FEATURE_CAPS),
        sta_disconnected_pm: sys::WIFI_STA_DISCONNECTED_PM_ENABLED != 0,
        espnow_max_encrypt_num: sys::CONFIG_ESP_WIFI_ESPNOW_MAX_ENCRYPT_NUM as _,
        tx_hetb_queue_num: sys::WIFI_TX_HETB_QUEUE_NUM as _,
        dump_hesigb_enable: sys::WIFI_DUMP_HESIGB_ENABLED != 0,
        wifi_task_stack_size: sys::WIFI_TASK_STACK_SIZE as _,
        magic: sys::WIFI_INIT_CONFIG_MAGIC as _,
    };
    sys::EspError::convert(unsafe { sys::esp_wifi_init(&cfg) })
}

/// Joins the first bench network that accepts the station. The driver is
/// leaked for the life of the process. Passphrases are not logged.
pub fn join(modem: Modem<'static>) -> bool {
    let env = local_env::load();
    if env.client_id.is_empty() || env.client_secret.is_empty() {
        log::error!("spotify client missing");
        return false;
    }
    let sysloop = match EspSystemEventLoop::take() {
        Ok(loop_) => loop_,
        Err(err) => {
            log::error!("event loop {err}");
            return false;
        }
    };
    let nvs = match EspDefaultNvsPartition::take() {
        Ok(nvs) => nvs,
        Err(err) => {
            log::error!("nvs {err}");
            return false;
        }
    };
    let internal = sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_8BIT;
    let dma = internal | sys::MALLOC_CAP_DMA;
    log::info!(
        "internal free {} largest {} dma free {} largest {}",
        unsafe { sys::heap_caps_get_free_size(internal) },
        unsafe { sys::heap_caps_get_largest_free_block(internal) },
        unsafe { sys::heap_caps_get_free_size(dma) },
        unsafe { sys::heap_caps_get_largest_free_block(dma) },
    );
    // esp-idf-svc 0.53 builds this struct from Default and leaves the stack
    // size at 0. A later esp_wifi_init sees the driver already up and returns.
    if let Err(err) = init_driver() {
        log::error!("wifi init {err}");
        return false;
    }
    let mut wifi = match EspWifi::new(modem, sysloop.clone(), Some(nvs)) {
        Ok(wifi) => match BlockingWifi::wrap(wifi, sysloop) {
            Ok(wifi) => wifi,
            Err(err) => {
                log::error!("wifi wrap {err}");
                return false;
            }
        },
        Err(err) => {
            log::error!("wifi new {err}");
            return false;
        }
    };
    // The IDF default is "espressif". The lease keeps that name unless this
    // is set before DHCP (P-0065).
    set_station_hostname(&mut wifi, env.device_name);
    let mut joined = false;
    for (index, (ssid, pass)) in env.wifi.iter().enumerate() {
        if ssid.is_empty() {
            continue;
        }
        log::info!("wifi try {} {ssid}", index + 1);
        if connect_one(&mut wifi, ssid, pass) {
            joined = true;
            break;
        }
    }
    if !joined {
        log::error!("wifi down");
        return false;
    }
    unsafe {
        sys::esp_wifi_set_ps(sys::wifi_ps_type_t_WIFI_PS_NONE);
    }
    let wifi = Box::leak(Box::new(wifi));
    let _keep = wifi;
    if !start_connect(&env) {
        return false;
    }
    true
}

const RELEASE_POLLS: u32 = 20;
const RELEASE_POLL_MS: u32 = 100;

/// Stops a previous attempt. The blocking driver calls wait with no deadline,
/// so this polls instead and then moves on.
fn release(wifi: &mut BlockingWifi<EspWifi<'static>>) {
    if wifi.is_connected().unwrap_or(false) {
        if let Err(err) = wifi.wifi_mut().disconnect() {
            log::error!("wifi disconnect {err}");
        }
    }
    for _ in 0..RELEASE_POLLS {
        if !wifi.is_connected().unwrap_or(false) {
            break;
        }
        FreeRtos::delay_ms(RELEASE_POLL_MS);
    }
    if wifi.is_started().unwrap_or(false) {
        if let Err(err) = wifi.wifi_mut().stop() {
            log::error!("wifi stop {err}");
        }
    }
    for _ in 0..RELEASE_POLLS {
        if !wifi.is_started().unwrap_or(false) {
            break;
        }
        FreeRtos::delay_ms(RELEASE_POLL_MS);
    }
}

fn ensure_started(wifi: &mut BlockingWifi<EspWifi<'static>>) -> bool {
    if wifi.is_started().unwrap_or(false) {
        return true;
    }
    if let Err(err) = wifi.wifi_mut().start() {
        log::error!("wifi start {err}");
        return false;
    }
    for _ in 0..RELEASE_POLLS {
        if wifi.is_started().unwrap_or(false) {
            return true;
        }
        FreeRtos::delay_ms(RELEASE_POLL_MS);
    }
    log::error!("wifi start timed out");
    false
}

fn connect_one(wifi: &mut BlockingWifi<EspWifi<'static>>, ssid: &str, pass: &str) -> bool {
    let ssid: String<32> = match String::try_from(ssid) {
        Ok(ssid) => ssid,
        Err(_) => {
            log::error!("wifi ssid too long");
            return false;
        }
    };
    let password: String<64> = match String::try_from(pass) {
        Ok(password) => password,
        Err(_) => {
            log::error!("wifi passphrase too long");
            return false;
        }
    };
    // Capable, not required: a station that advertises no PMF is refused by
    // many access points. WPA/WPA2 is the minimum; WPA2-only rejects WPA.
    let conf = Configuration::Client(ClientConfiguration {
        ssid,
        password,
        auth_method: AuthMethod::WPAWPA2Personal,
        pmf_cfg: PmfConfiguration::Capable { required: false },
        ..Default::default()
    });
    release(wifi);
    if let Err(err) = wifi.set_configuration(&conf) {
        log::error!("wifi config {err}");
        return false;
    }
    if !ensure_started(wifi) {
        return false;
    }
    if let Err(err) = wifi.connect() {
        log::error!("wifi auth {err}");
        release(wifi);
        return false;
    }
    if let Err(err) = wifi.wait_netif_up() {
        log::error!("wifi address {err}");
        release(wifi);
        return false;
    }
    log::info!("wifi up");
    true
}

fn set_station_hostname(wifi: &mut BlockingWifi<EspWifi<'static>>, name: &str) {
    let label = local_env::station_hostname(name);
    let Ok(hostname) = std::ffi::CString::new(label.as_str()) else {
        log::error!("wifi hostname rejected");
        return;
    };
    let rc = unsafe {
        sys::esp_netif_set_hostname(wifi.wifi_mut().sta_netif_mut().handle(), hostname.as_ptr())
    };
    if let Err(err) = sys::EspError::convert(rc) {
        log::error!("wifi hostname {err}");
    } else {
        log::info!("wifi hostname {}", label.as_str());
    }
}

fn start_connect(env: &BenchEnv) -> bool {
    let name = std::ffi::CString::new(env.device_name).ok();
    let host = std::ffi::CString::new(local_env::station_hostname(env.device_name).as_str()).ok();
    let id = std::ffi::CString::new(env.client_id).ok();
    let secret = std::ffi::CString::new(env.client_secret).ok();
    let (Some(name), Some(host), Some(id), Some(secret)) = (name, host, id, secret) else {
        log::error!("spotify client rejected");
        return false;
    };
    unsafe {
        porta_connect_start(name.as_ptr(), host.as_ptr(), id.as_ptr(), secret.as_ptr());
    }
    log::info!("cspot started");
    true
}

unsafe extern "C" {
    fn porta_connect_start(
        name: *const core::ffi::c_char,
        hostname: *const core::ffi::c_char,
        client_id: *const core::ffi::c_char,
        client_secret: *const core::ffi::c_char,
    );
    fn porta_connect_logged_in() -> i32;
    fn porta_connect_command(kind: i32, volume: i32);
    fn porta_connect_take_pcm(dst: *mut i16, frames: usize) -> usize;
    fn porta_connect_take_title(dst: *mut core::ffi::c_char, cap: usize) -> usize;
    fn porta_connect_take_pause() -> i32;
    fn porta_connect_take_flush() -> i32;
}

pub fn logged_in() -> bool {
    unsafe { porta_connect_logged_in() != 0 }
}

pub fn command(kind: i32, volume: i32) {
    unsafe { porta_connect_command(kind, volume) }
}

pub fn take_pcm(dst: &mut [i16]) -> usize {
    let frames = dst.len() / 2;
    unsafe { porta_connect_take_pcm(dst.as_mut_ptr(), frames) }
}

/// `Some(true)` is a pause from the session. `None` means no new edge.
pub fn take_remote_pause() -> Option<bool> {
    match unsafe { porta_connect_take_pause() } {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

pub fn take_flush() -> bool {
    unsafe { porta_connect_take_flush() != 0 }
}

pub fn take_title() -> Option<std::string::String> {
    let mut buf = [0 as core::ffi::c_char; 48];
    let n = unsafe { porta_connect_take_title(buf.as_mut_ptr(), buf.len()) };
    if n == 0 {
        return None;
    }
    let bytes: Vec<u8> = buf[..n].iter().map(|b| *b as u8).collect();
    let text = std::string::String::from_utf8(bytes).unwrap_or_default();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}
