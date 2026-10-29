/*
 * Copyright 2026 oemaix
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include "porta_app.h"
#include "porta_net.h"

#include <stdio.h>
#include <string.h>

static int g_failed;

static void expect(int cond, const char *name)
{
  if (!cond) {
    fprintf(stderr, "fail: %s\n", name);
    g_failed++;
  }
}

static porta_hw_t idle_hw(void)
{
  porta_hw_t hw;
  memset(&hw, 0, sizeof hw);
  return hw;
}

static void press(porta_app_t *app, int button)
{
  porta_hw_t down = idle_hw();
  porta_hw_t up = idle_hw();
  down.button_down[button] = true;
  porta_app_panel_step(app, &down);
  porta_app_panel_step(app, &up);
}

static void wheel(porta_app_t *app, int32_t steps)
{
  porta_hw_t hw = idle_hw();
  hw.increment = steps;
  porta_app_panel_step(app, &hw);
}

static int16_t peak(const int16_t *stereo, size_t frames)
{
  int16_t max = 0;
  size_t i;
  for (i = 0; i < frames * 2; i++) {
    int16_t v = stereo[i];
    if (v < 0) {
      v = (int16_t)(-v);
    }
    if (v > max) {
      max = v;
    }
  }
  return max;
}

static const porta_draw_item_t *focused(const porta_draw_item_t *items, size_t n)
{
  size_t i;
  for (i = 0; i < n; i++) {
    if (items[i].focused) {
      return &items[i];
    }
  }
  return NULL;
}

int main(void)
{
  porta_app_t app;
  porta_i2c_write_t leds[2];
  porta_block_view_t blocks[5];
  porta_draw_item_t draw[PORTA_DRAW_CAP];
  porta_hw_t hw;
  int16_t pcm[128];
  int16_t resampled[16];
  const porta_draw_item_t *item;
  uint8_t inc[4] = {2, 0, 0, 0};
  uint8_t touch[5] = {1, 0x01, 0x2c, 0x00, 0x10};
  bool valid = false;
  int16_t x = 0;
  int16_t y = 0;
  size_t n;

  expect(porta_hmi_led_off(leds, 2) == 2, "led writes");
  expect(leds[0].reg == PORTA_HMI_REG_LED1 && leds[0].value == 0, "led1 off");
  expect(leds[1].reg == PORTA_HMI_REG_LED2 && leds[1].value == 0, "led2 off");
  expect(porta_hmi_increment(inc) == 2, "increment");
  porta_ft6336_point(touch, &valid, &x, &y);
  expect(valid && x == 300 && y == 16, "touch point");

  porta_app_init(&app);
  expect(strcmp(app.status.title, "Tone stand-in") == 0, "title");
  expect(!app.status.session_ready, "session not ready");
  expect(!porta_board_i2s_allowed(&app.board), "i2s held");
  expect(app.board.bench_supply, "bench supply");
  n = porta_ui_draw(&app.ui, &app.status, draw, PORTA_DRAW_CAP);
  item = focused(draw, n);
  expect(item != NULL && strcmp(item->text, "Play") == 0, "focus play");

  press(&app, 0);
  memset(pcm, 0, sizeof pcm);
  porta_app_audio_step(&app, pcm, 64);
  expect(app.status.playing, "playing");
  expect(peak(pcm, 64) > 0, "tone");
  expect(strcmp(app.status.title, "Tone stand-in") == 0, "title stays");

  press(&app, 0);
  porta_app_audio_step(&app, pcm, 64);
  expect(!app.status.playing, "paused");
  expect(peak(pcm, 64) == 0, "silence");

  porta_queue_push(&app.queue, (porta_command_t){PORTA_CMD_NEXT, 0});
  porta_player_drain(&app.queue, &app.status);
  expect(strcmp(app.status.title, "Tone stand-in") == 0, "next is not a catalog");

  wheel(&app, 1);
  press(&app, 0);
  press(&app, 0);
  porta_app_audio_step(&app, pcm, 8);
  expect(app.status.volume == 70, "two louder steps");

  wheel(&app, 1);
  press(&app, 0);
  press(&app, 0);
  press(&app, 0);
  press(&app, 0);
  press(&app, 0);
  press(&app, 0);
  press(&app, 0);
  press(&app, 0);
  porta_app_audio_step(&app, pcm, 8);
  expect(app.status.volume == 0, "volume floor");

  porta_app_init(&app);
  wheel(&app, 3);
  press(&app, 0);
  expect(porta_ui_screen(&app.ui) == PORTA_SCREEN_CONNECT, "connect");
  n = porta_ui_blocks(&app.ui, &app.status, blocks, 5);
  expect(n == 2 && strcmp(blocks[0].label, "Wi-Fi down") == 0, "wifi down");
  porta_net_set_associated(&app.status, true);
  porta_ui_blocks(&app.ui, &app.status, blocks, 5);
  expect(strcmp(blocks[0].label, "Wi-Fi up") == 0, "wifi up");
  expect(strcmp(blocks[1].label, "Tone stand-in") == 0, "login placeholder");
  press(&app, 2);
  expect(porta_ui_screen(&app.ui) == PORTA_SCREEN_CONNECT, "button 3 ignored");
  press(&app, 1);
  expect(porta_ui_screen(&app.ui) == PORTA_SCREEN_NOW, "back");
  press(&app, 1);
  expect(porta_ui_screen(&app.ui) == PORTA_SCREEN_NOW, "back on root");

  wheel(&app, 4);
  press(&app, 0);
  expect(porta_ui_screen(&app.ui) == PORTA_SCREEN_SETTINGS, "settings");
  n = porta_ui_blocks(&app.ui, &app.status, blocks, 5);
  expect(n == 1 && strcmp(blocks[0].label, "No entries") == 0, "no battery row");
  press(&app, 1);
  expect(porta_ui_screen(&app.ui) == PORTA_SCREEN_NOW, "leave settings");

  n = porta_ui_blocks(&app.ui, &app.status, blocks, 5);
  hw = idle_hw();
  hw.touch_valid = true;
  hw.x = blocks[3].rect.x + 4;
  hw.y = blocks[3].rect.y + 4;
  porta_app_panel_step(&app, &hw);
  expect(app.ui.focus == 3, "tap moves focus");
  hw = idle_hw();
  porta_app_panel_step(&app, &hw);
  hw.touch_valid = true;
  hw.x = blocks[3].rect.x + 4;
  hw.y = blocks[3].rect.y + 4;
  porta_app_panel_step(&app, &hw);
  expect(porta_ui_screen(&app.ui) == PORTA_SCREEN_CONNECT, "tap activates");

  porta_app_init(&app);
  hw = idle_hw();
  hw.touch_valid = true;
  hw.x = 2;
  hw.y = 2;
  porta_app_panel_step(&app, &hw);
  expect(app.ui.focus == 0, "edge ignored");
  hw = idle_hw();
  porta_app_panel_step(&app, &hw);
  hw.touch_valid = true;
  hw.x = 10;
  hw.y = 260;
  porta_app_panel_step(&app, &hw);
  expect(app.queue.count == 0, "screen button dropped");
  expect(porta_ui_screen(&app.ui) == PORTA_SCREEN_NOW, "screen button stays");

  expect(!porta_sink_open(&app.sink, 22050), "odd rate rejected");
  expect(porta_resample_s16(pcm, 441, PORTA_RATE_44K, resampled, 16, PORTA_RATE_48K) == 0,
         "resample respects cap");
  {
    int16_t in[441 * 2];
    int16_t out[480 * 2];
    memset(in, 0, sizeof in);
    in[0] = 1000;
    expect(porta_resample_s16(in, 441, PORTA_RATE_44K, out, 480, PORTA_RATE_48K) == 480,
           "resample length");
  }
  expect(!porta_sink_write(&app.sink, pcm, 4, PORTA_RATE_48K), "wrong rate rejected");
  expect(app.sink.frames == 0, "rejected write is not counted");

  porta_app_init(&app);
  hw = idle_hw();
  hw.increment = 1;
  hw.button_down[0] = true;
  porta_app_panel_step(&app, &hw);
  porta_app_audio_step(&app, pcm, 8);
  expect(app.status.playing, "press confirms the previous focus");
  expect(app.ui.focus == 1, "wheel step follows the press");

  {
    uint8_t raw[3] = {0, 1, 0};
    bool down[3] = {false, false, false};
    porta_hmi_buttons(raw, down);
    expect(!down[0] && down[1] && !down[2], "button registers");
    porta_app_init(&app);
    wheel(&app, 3);
    press(&app, 0);
    hw = idle_hw();
    porta_hmi_buttons(raw, hw.button_down);
    porta_app_panel_step(&app, &hw);
    expect(porta_ui_screen(&app.ui) == PORTA_SCREEN_NOW, "register 0x21 goes back");
  }

  porta_app_init(&app);
  hw = idle_hw();
  hw.increment = 10;
  porta_app_panel_step(&app, &hw);
  expect(app.ui.focus == 3, "eight steps this poll");
  hw = idle_hw();
  porta_app_panel_step(&app, &hw);
  expect(app.ui.focus == 0, "leftover steps wait");

  porta_app_init(&app);
  app.status.volume = PORTA_VOLUME_MAX;
  wheel(&app, 1);
  press(&app, 0);
  expect(app.queue.count == 0, "louder at the ceiling is not queued");
  expect(app.status.volume == PORTA_VOLUME_MAX, "ceiling held");

  porta_app_init(&app);
  app.status.volume = 0;
  wheel(&app, 2);
  press(&app, 0);
  expect(app.queue.count == 0, "quieter at the floor is not queued");

  porta_app_init(&app);
  {
    porta_command_t filler = {PORTA_CMD_NEXT, 0};
    int i;
    porta_input_t held;
    porta_action_t one;
    for (i = 0; i < PORTA_QUEUE_CAP; i++) {
      expect(porta_queue_push(&app.queue, filler), "fill queue");
    }
    press(&app, 0);
    expect(!app.ui.shown_playing, "play label stays when the queue is full");
    expect(app.queue.count == PORTA_QUEUE_CAP, "full queue unchanged");
    expect(!app.status.playing, "play was not applied");

    porta_input_init(&held);
    hw = idle_hw();
    hw.button_down[0] = true;
    expect(porta_input_poll(&held, &hw, NULL, 0, 0, &one, 0) == 0, "no room for the edge");
    expect(!held.button_down[0], "edge stays armed");
    expect(porta_input_poll(&held, &hw, NULL, 0, 0, &one, 1) == 1, "edge retries");
    expect(one.kind == PORTA_ACTION_ACTIVATE, "retried press");
  }

  if (g_failed != 0) {
    fprintf(stderr, "%d failed\n", g_failed);
    return 1;
  }
  return 0;
}
