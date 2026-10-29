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

#include "porta_player.h"

static void copy_title(char *dst, const char *src)
{
  size_t i = 0;

  while (src[i] != '\0' && i + 1 < PORTA_TITLE_LEN) {
    dst[i] = src[i];
    i++;
  }
  dst[i] = '\0';
}

void porta_player_init(porta_player_t *player, porta_status_t *status)
{
  player->phase = 0;
  copy_title(status->title, "Tone stand-in");
  status->playing = false;
  status->volume = 50;
  status->session_ready = false;
}

void porta_player_drain(porta_queue_t *queue, porta_status_t *status)
{
  porta_command_t command;
  int volume;

  while (porta_queue_pop(queue, &command)) {
    switch (command.kind) {
    case PORTA_CMD_PLAY:
      status->playing = true;
      break;
    case PORTA_CMD_PAUSE:
      status->playing = false;
      break;
    case PORTA_CMD_NEXT:
    case PORTA_CMD_PREVIOUS:
      break;
    case PORTA_CMD_SET_VOLUME:
      volume = (int)status->volume + command.volume_step;
      if (volume < 0) {
        volume = 0;
      } else if (volume > PORTA_VOLUME_MAX) {
        volume = PORTA_VOLUME_MAX;
      }
      status->volume = (uint8_t)volume;
      break;
    }
  }
}

void porta_player_render(porta_player_t *player, const porta_status_t *status,
                         int16_t *stereo, size_t frames, uint32_t rate_hz)
{
  size_t i;
  uint32_t step = 0;

  if (rate_hz != 0) {
    step = (uint32_t)(((uint64_t)PORTA_TONE_HZ << 32) / rate_hz);
  }
  for (i = 0; i < frames; i++) {
    int32_t sample = 0;
    if (status->playing && status->volume > 0 && step != 0) {
      uint16_t saw = (uint16_t)(player->phase >> 16);
      int32_t tri = (saw < 32768) ? (int32_t)saw : (int32_t)(65535 - saw);
      tri -= 16384;
      sample = tri * (int32_t)status->volume / 256;
      player->phase += step;
    }
    stereo[i * 2] = (int16_t)sample;
    stereo[i * 2 + 1] = (int16_t)sample;
  }
}
