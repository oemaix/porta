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

#define PORTA_TONE_HZ 440
#define PORTA_TONE_RATE 44100

typedef struct {
  uint32_t phase;
} porta_player_t;

void porta_player_init(porta_player_t *player, porta_status_t *status);
void porta_player_drain(porta_queue_t *queue, porta_status_t *status);

/* Interleaved stereo s16. Silence while paused or at volume 0.
   The samples are a triangle labelled by the status title, not a track. */
void porta_player_render(porta_player_t *player, const porta_status_t *status,
                         int16_t *stereo, size_t frames, uint32_t rate_hz);
