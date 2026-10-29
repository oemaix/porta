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

#define PORTA_RATE_44K 44100u
#define PORTA_RATE_48K 48000u

typedef struct {
  bool open;
  uint32_t rate_hz;
  size_t frames;
} porta_sink_t;

void porta_sink_init(porta_sink_t *sink);

/* Accepts 16-bit stereo at 44100 or 48000. A second open rate is rejected. */
bool porta_sink_open(porta_sink_t *sink, uint32_t rate_hz);

/* Writes frames only when rate_hz is the opened rate. Another rate is
   rejected and not counted. Convert with porta_resample_s16 first. */
bool porta_sink_write(porta_sink_t *sink, const int16_t *stereo, size_t frames, uint32_t rate_hz);

/* Linear, 16-bit stereo, and only the two cable rates. Returns frames written. */
size_t porta_resample_s16(const int16_t *in, size_t in_frames, uint32_t in_rate,
                          int16_t *out, size_t out_cap, uint32_t out_rate);
