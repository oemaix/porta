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

#include "porta_input.h"

enum {
  EDGE = 8,
  WHEEL_CAP = 8,
};

void porta_input_init(porta_input_t *input)
{
  size_t i;

  for (i = 0; i < 3; i++) {
    input->button_down[i] = false;
    input->screen_down[i] = false;
  }
  input->panel_down = false;
  input->pending_steps = 0;
}

size_t porta_hmi_led_off(porta_i2c_write_t *out, size_t cap)
{
  if (cap < 2) {
    return 0;
  }
  out[0].reg = PORTA_HMI_REG_LED1;
  out[0].value = 0;
  out[1].reg = PORTA_HMI_REG_LED2;
  out[1].value = 0;
  return 2;
}

int32_t porta_hmi_increment(const uint8_t bytes[4])
{
  uint32_t value = (uint32_t)bytes[0]
      | ((uint32_t)bytes[1] << 8)
      | ((uint32_t)bytes[2] << 16)
      | ((uint32_t)bytes[3] << 24);
  return (int32_t)value;
}

void porta_hmi_buttons(const uint8_t bytes[3], bool down[3])
{
  size_t i;

  for (i = 0; i < 3; i++) {
    down[i] = bytes[i] != 0;
  }
}

void porta_ft6336_point(const uint8_t sample[5], bool *valid, int16_t *x, int16_t *y)
{
  uint8_t points = (uint8_t)(sample[0] & 0x0f);

  if (points == 0) {
    *valid = false;
    *x = 0;
    *y = 0;
    return;
  }
  *valid = true;
  *x = (int16_t)(((sample[1] & 0x0f) << 8) | sample[2]);
  *y = (int16_t)(((sample[3] & 0x0f) << 8) | sample[4]);
}

static bool push_action(porta_action_t *out, size_t cap, size_t *n, porta_action_kind_t kind, uint8_t index)
{
  if (*n >= cap) {
    return false;
  }
  out[*n].kind = kind;
  out[*n].index = index;
  (*n)++;
  return true;
}

static bool inside(const porta_rect_t *rect, int16_t x, int16_t y)
{
  return x >= rect->x && y >= rect->y && x < rect->x + rect->w && y < rect->y + rect->h;
}

static void emit_wheel(porta_input_t *input, porta_action_t *out, size_t cap, size_t *n)
{
  int32_t budget = WHEEL_CAP;
  int32_t steps = input->pending_steps;

  while (steps != 0 && budget > 0) {
    porta_action_kind_t kind = steps > 0 ? PORTA_ACTION_NEXT : PORTA_ACTION_PREVIOUS;
    if (!push_action(out, cap, n, kind, 0)) {
      break;
    }
    steps += steps > 0 ? -1 : 1;
    budget--;
  }
  input->pending_steps = steps;
}

size_t porta_input_poll(porta_input_t *input, const porta_hw_t *hw,
                        const porta_rect_t *blocks, size_t nblocks, uint8_t focus,
                        porta_action_t *out, size_t cap)
{
  size_t n = 0;
  size_t i;
  bool panel;
  int64_t summed;

  for (i = 0; i < 3; i++) {
    bool down = hw->button_down[i];
    if (down && !input->button_down[i]) {
      porta_action_kind_t kind = PORTA_ACTION_ACTIVATE;
      if (i == 1) {
        kind = PORTA_ACTION_BACK;
      } else if (i == 2) {
        kind = PORTA_ACTION_BUTTON3;
      }
      if (!push_action(out, cap, &n, kind, 0)) {
        continue;
      }
    }
    input->button_down[i] = down;
  }

  panel = hw->touch_valid
      && hw->x >= EDGE && hw->x < PORTA_PANEL_W - EDGE
      && hw->y >= EDGE && hw->y < PORTA_PANEL_H - EDGE;
  if (panel && !input->panel_down) {
    bool claimed = false;
    for (i = 0; i < nblocks; i++) {
      if (!inside(&blocks[i], hw->x, hw->y)) {
        continue;
      }
      claimed = true;
      if (i == focus) {
        if (push_action(out, cap, &n, PORTA_ACTION_ACTIVATE, 0)) {
          input->panel_down = true;
        }
      } else if (push_action(out, cap, &n, PORTA_ACTION_FOCUS, (uint8_t)i)) {
        input->panel_down = true;
      }
      break;
    }
    if (!claimed) {
      input->panel_down = true;
    }
  } else if (!panel) {
    input->panel_down = false;
  }

  if (hw->touch_valid && hw->y >= PORTA_PANEL_H && hw->x >= 0 && hw->x < PORTA_PANEL_W) {
    uint8_t col = (uint8_t)(((uint16_t)hw->x * 3) / PORTA_PANEL_W);
    bool down_now[3] = {false, false, false};
    if (col > 2) {
      col = 2;
    }
    down_now[col] = true;
    for (i = 0; i < 3; i++) {
      if (down_now[i] && !input->screen_down[i]) {
        if (!push_action(out, cap, &n, PORTA_ACTION_SCREEN, (uint8_t)i)) {
          continue;
        }
      }
      input->screen_down[i] = down_now[i];
    }
  } else {
    for (i = 0; i < 3; i++) {
      input->screen_down[i] = false;
    }
  }

  summed = (int64_t)input->pending_steps + hw->increment;
  if (summed > 32) {
    summed = 32;
  } else if (summed < -32) {
    summed = -32;
  }
  input->pending_steps = (int32_t)summed;
  emit_wheel(input, out, cap, &n);
  return n;
}
