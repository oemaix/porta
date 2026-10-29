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

#include "porta_sink.h"

static bool legal_rate(uint32_t rate_hz)
{
  return rate_hz == PORTA_RATE_44K || rate_hz == PORTA_RATE_48K;
}

void porta_sink_init(porta_sink_t *sink)
{
  sink->open = false;
  sink->rate_hz = 0;
  sink->frames = 0;
}

bool porta_sink_open(porta_sink_t *sink, uint32_t rate_hz)
{
  if (!legal_rate(rate_hz)) {
    return false;
  }
  if (sink->open && sink->rate_hz != rate_hz) {
    return false;
  }
  sink->open = true;
  sink->rate_hz = rate_hz;
  return true;
}

bool porta_sink_write(porta_sink_t *sink, const int16_t *stereo, size_t frames, uint32_t rate_hz)
{
  if (!sink->open || stereo == NULL || rate_hz != sink->rate_hz) {
    return false;
  }
  sink->frames += frames;
  return true;
}

size_t porta_resample_s16(const int16_t *in, size_t in_frames, uint32_t in_rate,
                          int16_t *out, size_t out_cap, uint32_t out_rate)
{
  size_t out_frames;
  size_t i;

  if (!legal_rate(in_rate) || !legal_rate(out_rate) || in_frames == 0) {
    return 0;
  }
  if (in_rate == out_rate) {
    if (out_cap < in_frames) {
      return 0;
    }
    for (i = 0; i < in_frames * 2; i++) {
      out[i] = in[i];
    }
    return in_frames;
  }
  out_frames = (size_t)(((uint64_t)in_frames * out_rate) / in_rate);
  if (out_frames == 0 || out_frames > out_cap) {
    return 0;
  }
  for (i = 0; i < out_frames; i++) {
    uint64_t pos = (uint64_t)i * in_rate;
    size_t src = (size_t)(pos / out_rate);
    uint32_t frac = (uint32_t)(pos % out_rate);
    size_t next = src + 1;
    size_t ch;
    if (next >= in_frames) {
      next = src;
    }
    for (ch = 0; ch < 2; ch++) {
      int32_t a = in[src * 2 + ch];
      int32_t b = in[next * 2 + ch];
      int32_t mixed = a + (int32_t)(((int64_t)(b - a) * frac) / out_rate);
      out[i * 2 + ch] = (int16_t)mixed;
    }
  }
  return out_frames;
}
