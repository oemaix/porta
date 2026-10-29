// Copyright 2026 oemaix
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Module HMI, I2C 0x41. Addresses follow the M5Module-HMI library map:
//! increment is a little-endian int32, buttons are one byte each.
//! Button 1 is the encoder press. Non-zero is treated as down.

use crate::seam::{Action, ActionKind, Rect};

pub const HMI_ADDR: u8 = 0x41;
pub const HMI_REG_INCREMENT: u8 = 0x10;
pub const HMI_REG_BUTTON1: u8 = 0x20;
pub const HMI_REG_BUTTON2: u8 = 0x21;
pub const HMI_REG_BUTTON3: u8 = 0x22;
pub const HMI_REG_LED1: u8 = 0x30;
pub const HMI_REG_LED2: u8 = 0x31;

pub const FT6336_ADDR: u8 = 0x38;
pub const PANEL_W: i16 = 320;
pub const PANEL_H: i16 = 240;

const EDGE: i16 = 8;
const WHEEL_CAP: i32 = 8;

#[derive(Clone, Copy, Debug)]
pub struct I2cWrite {
    pub reg: u8,
    pub value: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct Hw {
    pub increment: i32,
    pub button_down: [bool; 3],
    pub touch_valid: bool,
    pub x: i16,
    pub y: i16,
}

impl Hw {
    pub fn idle() -> Self {
        Self {
            increment: 0,
            button_down: [false; 3],
            touch_valid: false,
            x: 0,
            y: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Input {
    pub button_down: [bool; 3],
    panel_down: bool,
    screen_down: [bool; 3],
    pending_steps: i32,
}

impl Input {
    pub fn init() -> Self {
        Self {
            button_down: [false; 3],
            panel_down: false,
            screen_down: [false; 3],
            pending_steps: 0,
        }
    }

    /// Button and touch actions come before wheel steps, so a press confirms
    /// the block that already has focus. Wheel steps that do not fit stay in
    /// `pending_steps`. A rising edge is kept when the action list is full.
    /// `blocks` are the current screen, in focus order.
    pub fn poll(&mut self, hw: &Hw, blocks: &[Rect], focus: u8, out: &mut [Action]) -> usize {
        let mut n = 0usize;
        for i in 0..3 {
            let down = hw.button_down[i];
            if down && !self.button_down[i] {
                let kind = if i == 1 {
                    ActionKind::Back
                } else if i == 2 {
                    ActionKind::Button3
                } else {
                    ActionKind::Activate
                };
                if !push_action(out, &mut n, kind, 0) {
                    continue;
                }
            }
            self.button_down[i] = down;
        }

        let panel = hw.touch_valid
            && hw.x >= EDGE
            && hw.x < PANEL_W - EDGE
            && hw.y >= EDGE
            && hw.y < PANEL_H - EDGE;
        if panel && !self.panel_down {
            let mut claimed = false;
            for (i, block) in blocks.iter().enumerate() {
                if !inside(block, hw.x, hw.y) {
                    continue;
                }
                claimed = true;
                if i == focus as usize {
                    if push_action(out, &mut n, ActionKind::Activate, 0) {
                        self.panel_down = true;
                    }
                } else if push_action(out, &mut n, ActionKind::Focus, i as u8) {
                    self.panel_down = true;
                }
                break;
            }
            if !claimed {
                self.panel_down = true;
            }
        } else if !panel {
            self.panel_down = false;
        }

        if hw.touch_valid && hw.y >= PANEL_H && hw.x >= 0 && hw.x < PANEL_W {
            let mut col = ((hw.x as u16) * 3) / (PANEL_W as u16);
            if col > 2 {
                col = 2;
            }
            let mut down_now = [false; 3];
            down_now[col as usize] = true;
            for i in 0..3 {
                if down_now[i]
                    && !self.screen_down[i]
                    && !push_action(out, &mut n, ActionKind::Screen, i as u8)
                {
                    continue;
                }
                self.screen_down[i] = down_now[i];
            }
        } else {
            self.screen_down = [false; 3];
        }

        let mut summed = self.pending_steps as i64 + hw.increment as i64;
        if summed > 32 {
            summed = 32;
        } else if summed < -32 {
            summed = -32;
        }
        self.pending_steps = summed as i32;
        self.emit_wheel(out, &mut n);
        n
    }

    fn emit_wheel(&mut self, out: &mut [Action], n: &mut usize) {
        let mut budget = WHEEL_CAP;
        let mut steps = self.pending_steps;
        while steps != 0 && budget > 0 {
            let kind = if steps > 0 {
                ActionKind::Next
            } else {
                ActionKind::Previous
            };
            if !push_action(out, n, kind, 0) {
                break;
            }
            steps += if steps > 0 { -1 } else { 1 };
            budget -= 1;
        }
        self.pending_steps = steps;
    }
}

/// Two writes that force the HMI LEDs off. The module can keep an old value.
pub fn hmi_led_off(out: &mut [I2cWrite]) -> usize {
    if out.len() < 2 {
        return 0;
    }
    out[0] = I2cWrite {
        reg: HMI_REG_LED1,
        value: 0,
    };
    out[1] = I2cWrite {
        reg: HMI_REG_LED2,
        value: 0,
    };
    2
}

pub fn hmi_increment(bytes: &[u8; 4]) -> i32 {
    let value = bytes[0] as u32
        | ((bytes[1] as u32) << 8)
        | ((bytes[2] as u32) << 16)
        | ((bytes[3] as u32) << 24);
    value as i32
}

/// `bytes` are registers 0x20, 0x21, 0x22. Index 0 is the encoder press.
pub fn hmi_buttons(bytes: &[u8; 3], down: &mut [bool; 3]) {
    for i in 0..3 {
        down[i] = bytes[i] != 0;
    }
}

/// `sample` is TD_STATUS then P1_XH, P1_XL, P1_YH, P1_YL.
pub fn ft6336_point(sample: &[u8; 5]) -> (bool, i16, i16) {
    let points = sample[0] & 0x0f;
    if points == 0 {
        return (false, 0, 0);
    }
    let x = (((sample[1] & 0x0f) as u16) << 8) | sample[2] as u16;
    let y = (((sample[3] & 0x0f) as u16) << 8) | sample[4] as u16;
    (true, x as i16, y as i16)
}

fn push_action(out: &mut [Action], n: &mut usize, kind: ActionKind, index: u8) -> bool {
    if *n >= out.len() {
        return false;
    }
    out[*n] = Action { kind, index };
    *n += 1;
    true
}

fn inside(rect: &Rect, x: i16, y: i16) -> bool {
    x >= rect.x && y >= rect.y && x < rect.x + rect.w && y < rect.y + rect.h
}
