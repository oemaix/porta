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

#include "porta_ui.h"

#include <stdio.h>

enum {
  /* Interim palette from D-0035, RGB565. Drop these when a palette is chosen. */
  COLOR_BG = 0x18E3,
  COLOR_SURFACE = 0x2965,
  COLOR_ACCENT = 0x3BFF,
  BLOCK_X = 16,
  BLOCK_W = 288,
  BLOCK_H = 28,
  BLOCK_GAP = 8,
  BLOCK_Y0 = 48,
};

static void copy_label(char *dst, const char *src)
{
  size_t i = 0;

  while (src[i] != '\0' && i + 1 < PORTA_LABEL_LEN) {
    dst[i] = src[i];
    i++;
  }
  dst[i] = '\0';
}

static void place(porta_block_view_t *block, size_t index, porta_role_t role, const char *label)
{
  block->rect.x = BLOCK_X;
  block->rect.y = (int16_t)(BLOCK_Y0 + (int)index * (BLOCK_H + BLOCK_GAP));
  block->rect.w = BLOCK_W;
  block->rect.h = BLOCK_H;
  block->role = role;
  copy_label(block->label, label);
}

void porta_ui_init(porta_ui_t *ui, const porta_status_t *status)
{
  ui->stack[0] = PORTA_SCREEN_NOW;
  ui->depth = 1;
  ui->focus = 0;
  ui->shown_playing = status->playing;
  ui->shown_volume = status->volume;
  ui->volume_text[0] = '\0';
}

porta_screen_t porta_ui_screen(const porta_ui_t *ui)
{
  return ui->stack[ui->depth - 1];
}

size_t porta_ui_blocks(porta_ui_t *ui, const porta_status_t *status,
                       porta_block_view_t *out, size_t cap)
{
  porta_screen_t screen = porta_ui_screen(ui);

  if (screen == PORTA_SCREEN_CONNECT) {
    if (cap < 2) {
      return 0;
    }
    place(&out[0], 0, PORTA_ROLE_INFO, status->wifi_associated ? "Wi-Fi up" : "Wi-Fi down");
    place(&out[1], 1, PORTA_ROLE_INFO, "Tone stand-in");
    return 2;
  }
  if (screen == PORTA_SCREEN_SETTINGS) {
    if (cap < 1) {
      return 0;
    }
    place(&out[0], 0, PORTA_ROLE_INFO, "No entries");
    return 1;
  }
  if (cap < 5) {
    return 0;
  }
  place(&out[0], 0, PORTA_ROLE_PLAY, ui->shown_playing ? "Pause" : "Play");
  place(&out[1], 1, PORTA_ROLE_LOUDER, "Louder");
  place(&out[2], 2, PORTA_ROLE_QUIETER, "Quieter");
  place(&out[3], 3, PORTA_ROLE_OPEN_CONNECT, "Connect");
  place(&out[4], 4, PORTA_ROLE_OPEN_SETTINGS, "Settings");
  return 5;
}

static void push_screen(porta_ui_t *ui, porta_screen_t screen)
{
  if (ui->depth >= 3) {
    return;
  }
  ui->stack[ui->depth] = screen;
  ui->depth++;
  ui->focus = 0;
}

static bool send(porta_queue_t *queue, porta_command_kind_t kind, int8_t step)
{
  porta_command_t command;

  command.kind = kind;
  command.volume_step = step;
  return porta_queue_push(queue, command);
}

void porta_ui_apply(porta_ui_t *ui, const porta_status_t *status,
                    porta_queue_t *queue, porta_action_t action)
{
  porta_block_view_t blocks[5];
  size_t nblocks;
  uint8_t next;
  bool next_playing;

  if (queue->count == 0) {
    ui->shown_playing = status->playing;
    ui->shown_volume = status->volume;
  }
  nblocks = porta_ui_blocks(ui, status, blocks, 5);
  if (nblocks == 0) {
    return;
  }
  if (ui->focus >= nblocks) {
    ui->focus = 0;
  }

  switch (action.kind) {
  case PORTA_ACTION_NEXT:
    ui->focus = (uint8_t)((ui->focus + 1) % nblocks);
    break;
  case PORTA_ACTION_PREVIOUS:
    ui->focus = (uint8_t)((ui->focus + nblocks - 1) % nblocks);
    break;
  case PORTA_ACTION_FOCUS:
    if (action.index < nblocks) {
      ui->focus = action.index;
    }
    break;
  case PORTA_ACTION_BACK:
    if (ui->depth > 1) {
      ui->depth--;
      ui->focus = 0;
    }
    break;
  case PORTA_ACTION_ACTIVATE:
    switch (blocks[ui->focus].role) {
    case PORTA_ROLE_PLAY:
      next_playing = !ui->shown_playing;
      if (send(queue, next_playing ? PORTA_CMD_PLAY : PORTA_CMD_PAUSE, 0)) {
        ui->shown_playing = next_playing;
      }
      break;
    case PORTA_ROLE_LOUDER:
      if (ui->shown_volume >= PORTA_VOLUME_MAX) {
        break;
      }
      if (send(queue, PORTA_CMD_SET_VOLUME, PORTA_VOLUME_STEP)) {
        next = ui->shown_volume;
        if (next <= PORTA_VOLUME_MAX - PORTA_VOLUME_STEP) {
          next = (uint8_t)(next + PORTA_VOLUME_STEP);
        } else {
          next = PORTA_VOLUME_MAX;
        }
        ui->shown_volume = next;
      }
      break;
    case PORTA_ROLE_QUIETER:
      if (ui->shown_volume == 0) {
        break;
      }
      if (send(queue, PORTA_CMD_SET_VOLUME, -PORTA_VOLUME_STEP)) {
        if (ui->shown_volume >= PORTA_VOLUME_STEP) {
          ui->shown_volume = (uint8_t)(ui->shown_volume - PORTA_VOLUME_STEP);
        } else {
          ui->shown_volume = 0;
        }
      }
      break;
    case PORTA_ROLE_OPEN_CONNECT:
      push_screen(ui, PORTA_SCREEN_CONNECT);
      break;
    case PORTA_ROLE_OPEN_SETTINGS:
      push_screen(ui, PORTA_SCREEN_SETTINGS);
      break;
    case PORTA_ROLE_INFO:
      break;
    }
    break;
  case PORTA_ACTION_BUTTON3:
  case PORTA_ACTION_SCREEN:
    break;
  }
}

size_t porta_ui_draw(porta_ui_t *ui, const porta_status_t *status,
                     porta_draw_item_t *out, size_t cap)
{
  porta_block_view_t blocks[5];
  size_t nblocks;
  size_t i;
  size_t n = 0;

  if (cap < 2) {
    return 0;
  }
  snprintf(ui->volume_text, sizeof ui->volume_text, "Volume %u", (unsigned)ui->shown_volume);
  out[n].rect.x = 0;
  out[n].rect.y = 0;
  out[n].rect.w = 320;
  out[n].rect.h = 240;
  out[n].fill = COLOR_BG;
  copy_label(out[n].text, status->title);
  out[n].focused = false;
  n++;
  out[n].rect.x = BLOCK_X;
  out[n].rect.y = 12;
  out[n].rect.w = BLOCK_W;
  out[n].rect.h = 28;
  out[n].fill = COLOR_BG;
  copy_label(out[n].text, ui->volume_text);
  out[n].focused = false;
  n++;

  nblocks = porta_ui_blocks(ui, status, blocks, 5);
  for (i = 0; i < nblocks && n < cap; i++) {
    out[n].rect = blocks[i].rect;
    out[n].fill = (i == ui->focus) ? COLOR_ACCENT : COLOR_SURFACE;
    copy_label(out[n].text, blocks[i].label);
    out[n].focused = i == ui->focus;
    n++;
  }
  return n;
}
