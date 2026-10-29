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

#include <string.h>

void porta_app_init(porta_app_t *app)
{
  memset(app, 0, sizeof *app);
  porta_queue_init(&app->queue);
  porta_player_init(&app->player, &app->status);
  porta_ui_init(&app->ui, &app->status);
  porta_input_init(&app->input);
  porta_sink_init(&app->sink);
  porta_board_init(&app->board);
  porta_board_use_bench(&app->board);
  porta_sink_open(&app->sink, PORTA_TONE_RATE);
}

void porta_app_panel_step(porta_app_t *app, const porta_hw_t *hw)
{
  porta_block_view_t blocks[5];
  porta_rect_t rects[5];
  porta_action_t actions[8];
  size_t nblocks;
  size_t nactions;
  size_t i;

  nblocks = porta_ui_blocks(&app->ui, &app->status, blocks, 5);
  for (i = 0; i < nblocks; i++) {
    rects[i] = blocks[i].rect;
  }
  nactions = porta_input_poll(&app->input, hw, rects, nblocks, app->ui.focus,
                              actions, 8);
  for (i = 0; i < nactions; i++) {
    porta_ui_apply(&app->ui, &app->status, &app->queue, actions[i]);
  }
}

void porta_app_audio_step(porta_app_t *app, int16_t *stereo, size_t frames)
{
  /* Software PCM only. porta_board_i2s_allowed gates the codec, which this
     step does not start. */
  porta_player_drain(&app->queue, &app->status);
  porta_player_render(&app->player, &app->status, stereo, frames, app->sink.rate_hz);
  porta_sink_write(&app->sink, stereo, frames, app->sink.rate_hz);
}
