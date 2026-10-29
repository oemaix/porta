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

#include "porta_board.h"
#include "porta_input.h"
#include "porta_player.h"
#include "porta_seam.h"
#include "porta_sink.h"
#include "porta_ui.h"

#include <stddef.h>
#include <stdint.h>

typedef struct {
  porta_ui_t ui;
  porta_input_t input;
  porta_player_t player;
  porta_sink_t sink;
  porta_board_t board;
  porta_status_t status;
  porta_queue_t queue;
} porta_app_t;

void porta_app_init(porta_app_t *app);
void porta_app_panel_step(porta_app_t *app, const porta_hw_t *hw);
void porta_app_audio_step(porta_app_t *app, int16_t *stereo, size_t frames);
