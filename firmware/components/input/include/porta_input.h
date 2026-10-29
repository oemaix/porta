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

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/* Module HMI, I2C 0x41. Addresses follow the M5Module-HMI library map:
   increment is a little-endian int32, buttons are one byte each.
   Button 1 is the encoder press. Non-zero is treated as down.
   Confirm both on the board before calling a miss a UI bug. */
#define PORTA_HMI_ADDR 0x41
#define PORTA_HMI_REG_INCREMENT 0x10
#define PORTA_HMI_REG_BUTTON1 0x20
#define PORTA_HMI_REG_BUTTON2 0x21
#define PORTA_HMI_REG_BUTTON3 0x22
#define PORTA_HMI_REG_LED1 0x30
#define PORTA_HMI_REG_LED2 0x31

#define PORTA_FT6336_ADDR 0x38
#define PORTA_PANEL_W 320
#define PORTA_PANEL_H 240

typedef struct {
  uint8_t reg;
  uint8_t value;
} porta_i2c_write_t;

typedef struct {
  int32_t increment;
  bool button_down[3];
  bool touch_valid;
  int16_t x;
  int16_t y;
} porta_hw_t;

typedef struct {
  bool button_down[3];
  bool panel_down;
  bool screen_down[3];
  int32_t pending_steps;
} porta_input_t;

void porta_input_init(porta_input_t *input);

/* Two writes that force the HMI LEDs off. The module can keep an old value. */
size_t porta_hmi_led_off(porta_i2c_write_t *out, size_t cap);

int32_t porta_hmi_increment(const uint8_t bytes[4]);

/* bytes are registers 0x20, 0x21, 0x22. Index 0 is the encoder press. */
void porta_hmi_buttons(const uint8_t bytes[3], bool down[3]);

/* sample is TD_STATUS then P1_XH, P1_XL, P1_YH, P1_YL. */
void porta_ft6336_point(const uint8_t sample[5], bool *valid, int16_t *x, int16_t *y);

/* Button and touch actions come before wheel steps, so a press confirms
   the block that already has focus. Wheel steps that do not fit stay in
   pending_steps. A rising edge is kept when the action list is full.
   blocks are the current screen, in focus order. */
size_t porta_input_poll(porta_input_t *input, const porta_hw_t *hw,
                        const porta_rect_t *blocks, size_t nblocks, uint8_t focus,
                        porta_action_t *out, size_t cap);
