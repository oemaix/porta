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

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define PORTA_TITLE_LEN 48
#define PORTA_QUEUE_CAP 8
#define PORTA_VOLUME_MAX 100
#define PORTA_VOLUME_STEP 10

/* Listener-facing actions are next, previous, activate, and back.
   Focus is how a tap names a block. Button 3 and the screen buttons are
   reported and dropped by ui. */
typedef enum {
  PORTA_ACTION_NEXT = 1,
  PORTA_ACTION_PREVIOUS,
  PORTA_ACTION_ACTIVATE,
  PORTA_ACTION_BACK,
  PORTA_ACTION_FOCUS,
  PORTA_ACTION_BUTTON3,
  PORTA_ACTION_SCREEN,
} porta_action_kind_t;

typedef struct {
  porta_action_kind_t kind;
  uint8_t index;
} porta_action_t;

typedef enum {
  PORTA_CMD_PLAY = 1,
  PORTA_CMD_PAUSE,
  PORTA_CMD_NEXT,
  PORTA_CMD_PREVIOUS,
  PORTA_CMD_SET_VOLUME,
} porta_command_kind_t;

typedef struct {
  porta_command_kind_t kind;
  int8_t volume_step;
} porta_command_t;

typedef struct {
  char title[PORTA_TITLE_LEN];
  bool playing;
  uint8_t volume;
  bool session_ready;
  bool wifi_associated;
} porta_status_t;

typedef struct {
  porta_command_t items[PORTA_QUEUE_CAP];
  uint8_t head;
  uint8_t count;
} porta_queue_t;

typedef struct {
  int16_t x;
  int16_t y;
  int16_t w;
  int16_t h;
} porta_rect_t;

void porta_queue_init(porta_queue_t *queue);
bool porta_queue_push(porta_queue_t *queue, porta_command_t command);
bool porta_queue_pop(porta_queue_t *queue, porta_command_t *command);
