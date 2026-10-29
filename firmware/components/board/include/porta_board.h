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

/* Internal I2C is GPIO21 and GPIO22. The AXP192 is at 0x34.
   Module Audio configuration A shares the internal codec pins.
   Hardware I2S stays off until a measured current limit exists. */
#define PORTA_I2C_SDA_GPIO 21
#define PORTA_I2C_SCL_GPIO 22
#define PORTA_AXP192_ADDR 0x34

typedef struct {
  bool bench_supply;
  bool limit_measured;
} porta_board_t;

void porta_board_init(porta_board_t *board);
void porta_board_use_bench(porta_board_t *board);
bool porta_board_i2s_allowed(const porta_board_t *board);
