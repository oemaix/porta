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

#pragma once

#include "porta_seam.h"

#include <stddef.h>
#include <stdint.h>

#define PORTA_LABEL_LEN 24
#define PORTA_DRAW_CAP 8

typedef enum {
  PORTA_SCREEN_NOW = 0,
  PORTA_SCREEN_CONNECT,
  PORTA_SCREEN_SETTINGS,
} porta_screen_t;

typedef enum {
  PORTA_ROLE_PLAY = 1,
  PORTA_ROLE_LOUDER,
  PORTA_ROLE_QUIETER,
  PORTA_ROLE_OPEN_CONNECT,
  PORTA_ROLE_OPEN_SETTINGS,
  PORTA_ROLE_INFO,
} porta_role_t;

typedef struct {
  porta_rect_t rect;
  porta_role_t role;
  char label[PORTA_LABEL_LEN];
} porta_block_view_t;

typedef struct {
  porta_rect_t rect;
  uint16_t fill;
  char text[PORTA_LABEL_LEN];
  bool focused;
} porta_draw_item_t;

typedef struct {
  porta_screen_t stack[3];
  uint8_t depth;
  uint8_t focus;
  bool shown_playing;
  uint8_t shown_volume;
  char volume_text[PORTA_LABEL_LEN];
} porta_ui_t;

void porta_ui_init(porta_ui_t *ui, const porta_status_t *status);
porta_screen_t porta_ui_screen(const porta_ui_t *ui);
size_t porta_ui_blocks(porta_ui_t *ui, const porta_status_t *status,
                       porta_block_view_t *out, size_t cap);
void porta_ui_apply(porta_ui_t *ui, const porta_status_t *status,
                    porta_queue_t *queue, porta_action_t action);
size_t porta_ui_draw(porta_ui_t *ui, const porta_status_t *status,
                     porta_draw_item_t *out, size_t cap);
