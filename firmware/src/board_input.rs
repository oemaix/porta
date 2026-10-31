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

//! Reads the Module HMI and the FT6336U on the bus opened for the AXP192.
//! The host test does not link this. It feeds `Hw` itself.

use esp_idf_svc::hal::delay::{FreeRtos, TickType};
use esp_idf_svc::hal::i2c::I2cDriver;

use crate::input::{
    ft6336_point, hmi_increment, hmi_led_off, Hw, I2cWrite, WheelDetent, FT6336_ADDR, HMI_ADDR,
    HMI_REG_BUTTON1, HMI_REG_ENCODER,
};

const FT6336_REG_POINTS: u8 = 0x02;
const I2C_WAIT_MS: u64 = 30;

pub struct BusState {
    wheel: WheelDetent,
    buttons: [bool; 3],
    touching: bool,
    hmi_up: Option<bool>,
    button_err: bool,
    phase: u8,
}

impl BusState {
    pub fn new() -> Self {
        Self {
            wheel: WheelDetent::new(),
            buttons: [false; 3],
            touching: false,
            hmi_up: None,
            button_err: false,
            phase: 0,
        }
    }
}

/// Register pointer, stop, then a fresh read. A repeated start on the
/// second HMI transaction left the STM32 holding SCL (P-0054).
fn read_hmi(
    i2c: &mut I2cDriver<'_>,
    reg: u8,
    buf: &mut [u8],
) -> Result<(), esp_idf_svc::sys::EspError> {
    let ticks = TickType::new_millis(I2C_WAIT_MS).ticks();
    i2c.write(HMI_ADDR, &[reg], ticks)?;
    i2c.read(HMI_ADDR, buf, ticks)
}

fn read_at(
    i2c: &mut I2cDriver<'_>,
    addr: u8,
    reg: u8,
    buf: &mut [u8],
) -> Result<(), esp_idf_svc::sys::EspError> {
    let ticks = TickType::new_millis(I2C_WAIT_MS).ticks();
    i2c.write_read(addr, &[reg], buf, ticks)
}

fn write_reg(
    i2c: &mut I2cDriver<'_>,
    addr: u8,
    reg: u8,
    value: u8,
) -> Result<(), esp_idf_svc::sys::EspError> {
    let ticks = TickType::new_millis(I2C_WAIT_MS).ticks();
    i2c.write(addr, &[reg, value], ticks)
}

/// Clears both HMI LEDs. The module can keep an old value across reset.
/// The STM32 NACKs for several hundred milliseconds after M-Bus 5 V.
pub fn leds_off(i2c: &mut I2cDriver<'_>) {
    let mut writes = [I2cWrite { reg: 0, value: 0 }; 2];
    let n = hmi_led_off(&mut writes);
    let mut last_err = None;
    for _ in 0..20 {
        let mut failed = false;
        for write in writes.iter().take(n) {
            if let Err(err) = write_reg(i2c, HMI_ADDR, write.reg, write.value) {
                last_err = Some(err);
                failed = true;
                break;
            }
        }
        if !failed {
            log::info!("hmi leds off");
            return;
        }
        FreeRtos::delay_ms(50);
    }
    if let Some(err) = last_err {
        log::error!("hmi led off failed: {err}");
    }
    log_acks(i2c);
}

/// One pass over the bus after the HMI address NACKs. A NACK returns
/// at once, so this does not wait out the command timeout per address.
fn log_acks(i2c: &mut I2cDriver<'_>) {
    let ticks = TickType::new_millis(5).ticks();
    let mut list = String::new();
    for addr in 0x08u8..=0x77 {
        if i2c.write(addr, &[], ticks).is_ok() {
            list.push_str(&format!(" {addr:02x}"));
        }
    }
    log::info!("i2c ack{list}");
}

/// One sample. The first encoder count is the baseline. Four counts
/// are one click and move focus once (P-0057).
/// A missing device leaves that part of `Hw` idle.
pub fn sample(i2c: &mut I2cDriver<'_>, state: &mut BusState) -> Hw {
    let mut hw = Hw::idle();
    // Touch first. A stretch abort on the HMI clears the bus for 50 ms
    // and then the FT6336U misses this sample (P-0054).
    let mut touch = [0u8; 5];
    if read_at(i2c, FT6336_ADDR, FT6336_REG_POINTS, &mut touch).is_ok() {
        let (valid, x, y) = ft6336_point(&touch);
        hw.touch_valid = valid;
        hw.x = x;
        hw.y = y;
        if valid && !state.touching {
            log::info!("touch {x},{y}");
        }
        state.touching = valid;
    }
    // One HMI register per sample. A 3-byte read at 0x20 timed out and
    // the next encoder read timed out with it (P-0054).
    hw.button_down = state.buttons;
    state.phase = state.phase.wrapping_add(1);
    if state.phase % 2 == 1 {
        let mut count = [0u8; 4];
        match read_hmi(i2c, HMI_REG_ENCODER, &mut count) {
            Ok(()) => {
                if state.hmi_up != Some(true) {
                    log::info!("hmi 0x41 ready");
                    state.hmi_up = Some(true);
                }
                // A rising count on this module moves focus backward.
                // One click is four counts (P-0057).
                hw.increment = -state.wheel.observe(hmi_increment(&count));
            }
            Err(err) => {
                if state.hmi_up != Some(false) {
                    log::error!("hmi {HMI_REG_ENCODER:#04x} {err}");
                    state.hmi_up = Some(false);
                }
            }
        }
    } else if state.hmi_up == Some(true) && !state.button_err {
        let idx = ((state.phase / 2) % 3) as usize;
        let reg = HMI_REG_BUTTON1 + idx as u8;
        let mut byte = [0u8; 1];
        match read_hmi(i2c, reg, &mut byte) {
            Ok(()) => {
                let down = byte[0] != 0;
                if down && !state.buttons[idx] {
                    log::info!("hmi reg {reg:#04x} down");
                }
                state.buttons[idx] = down;
                hw.button_down = state.buttons;
            }
            Err(err) => {
                log::error!("hmi {reg:#04x} {err}");
                state.button_err = true;
            }
        }
    }
    hw
}
