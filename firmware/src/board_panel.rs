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

//! Core2 v1.3 panel rails on the AXP192.
//! LDO2 is ILI9342C logic power, DCDC3 is the backlight, AXP GPIO4 is
//! LCD reset. LDO2 also feeds the microSD slot. ESP32 GPIO4 is that
//! slot's chip-select and is held high so the card ignores the bus.
//! This file does not write REG 0x30 and does not enable the NS4168
//! (AXP GPIO2, active high).

use std::time::Duration;

use esp_idf_svc::hal::delay::{FreeRtos, TickType};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::FromValueType;
use esp_idf_svc::sys;

use crate::board::{AXP192_ADDR, I2C_SCL_GPIO, I2C_SDA_GPIO};

const AXP_STATUS_REG: u8 = 0x00;
const AXP_EXTEN_REG: u8 = 0x10;
const AXP_GPIO0_FN_REG: u8 = 0x90;
const AXP_GPIO0_VOLT_REG: u8 = 0x91;
const AXP_ID_REG: u8 = 0x03;
const AXP_ID_VALUE: u8 = 0x03;
const AXP_OUT_REG: u8 = 0x12;
const AXP_DCDC3_REG: u8 = 0x27;
const AXP_LDO_REG: u8 = 0x28;
const AXP_GPIO_LEVEL_REG: u8 = 0x94;
const AXP_GPIO4_FN_REG: u8 = 0x95;
const AXP_GPIO4_LEVEL_REG: u8 = 0x96;

const AXP_VBUS_VHOLD: u8 = 1 << 3;
const AXP_EXTEN: u8 = 1 << 2;
const AXP_GPIO0_LDO: u8 = 0x02;
const AXP_GPIO0_FLOAT: u8 = 0x07;
const AXP_OUT_DCDC3: u8 = 1 << 1;
const AXP_OUT_LDO2: u8 = 1 << 2;
const AXP_GPIO2_HIGH: u8 = 1 << 2;
const AXP_GPIO4_RELEASE: u8 = 1 << 1;

/// 0.7 V + 84 * 25 mV = 2.8 V. Bit 7 of REG 27H stays as read.
const AXP_DCDC3_2V8: u8 = 0x54;
/// LDO2 3.3 V is the high nibble of REG 28H. LDO3 stays as read.
const AXP_LDO2_3V3: u8 = 0xF0;

const SD_CS_GPIO: i32 = 4;

fn hold_sd_cs() {
    let cfg = sys::gpio_config_t {
        pin_bit_mask: 1u64 << SD_CS_GPIO,
        mode: sys::gpio_mode_t_GPIO_MODE_OUTPUT,
        pull_up_en: sys::gpio_pullup_t_GPIO_PULLUP_DISABLE,
        pull_down_en: sys::gpio_pulldown_t_GPIO_PULLDOWN_DISABLE,
        intr_type: sys::gpio_int_type_t_GPIO_INTR_DISABLE,
        ..Default::default()
    };
    unsafe {
        sys::gpio_config(&cfg);
        sys::gpio_set_level(SD_CS_GPIO, 1);
    }
}

fn axp_read(i2c: &mut I2cDriver<'_>, reg: u8) -> Option<u8> {
    let mut value = [0u8; 1];
    let ticks = TickType::new_millis(40).ticks();
    match i2c.write_read(AXP192_ADDR, &[reg], &mut value, ticks) {
        Ok(()) => Some(value[0]),
        Err(err) => {
            log::error!("axp read {reg:#04x} {err}");
            None
        }
    }
}

fn axp_write(i2c: &mut I2cDriver<'_>, reg: u8, value: u8) -> bool {
    let ticks = TickType::new_millis(40).ticks();
    match i2c.write(AXP192_ADDR, &[reg, value], ticks) {
        Ok(()) => true,
        Err(err) => {
            log::error!("axp write {reg:#04x} {err}");
            false
        }
    }
}

/// Samples the idle levels. A low SCL is a slave holding the line (P-0056).
fn log_bus_levels() {
    let cfg = sys::gpio_config_t {
        pin_bit_mask: (1u64 << I2C_SDA_GPIO) | (1u64 << I2C_SCL_GPIO),
        mode: sys::gpio_mode_t_GPIO_MODE_INPUT,
        pull_up_en: sys::gpio_pullup_t_GPIO_PULLUP_ENABLE,
        pull_down_en: sys::gpio_pulldown_t_GPIO_PULLDOWN_DISABLE,
        intr_type: sys::gpio_int_type_t_GPIO_INTR_DISABLE,
        ..Default::default()
    };
    unsafe {
        sys::gpio_config(&cfg);
        let scl = sys::gpio_get_level(I2C_SCL_GPIO);
        let sda = sys::gpio_get_level(I2C_SDA_GPIO);
        log::info!("i2c levels scl={scl} sda={sda}");
    }
}

pub struct OpenedBus {
    pub i2c: I2cDriver<'static>,
    pub modem: Modem<'static>,
    pub audio: crate::board_audio::AudioPins,
}

/// Opens the internal I2C bus and keeps the Wi-Fi modem. The driver stays
/// alive for the AXP192, the FT6336U, and the Module HMI (P-0052).
pub fn open_i2c() -> Option<OpenedBus> {
    log_bus_levels();
    let peripherals = Peripherals::take().ok()?;
    // 100 kHz and a 12 ms stretch window. The three-byte button read
    // exceeded 8 ms and the following encoder read then timed out (P-0054).
    let config = I2cConfig::new()
        .baudrate(100_u32.kHz().into())
        .timeout(Duration::from_millis(12).into())
        .sda_enable_pullup(true)
        .scl_enable_pullup(true);
    match I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &config,
    ) {
        Ok(i2c) => Some(OpenedBus {
            i2c,
            modem: peripherals.modem,
            audio: crate::board_audio::AudioPins {
                i2s: peripherals.i2s0,
                mclk: peripherals.pins.gpio0,
                bclk: peripherals.pins.gpio19,
                ws: peripherals.pins.gpio27,
                dout: peripherals.pins.gpio2,
            },
        }),
        Err(_) => {
            log::error!("i2c bus failed");
            None
        }
    }
}

/// Enables the ILI9342C logic supply and backlight, and pulses LCD reset.
/// Does not write the VBUS current limit or set `limit_measured`.
/// The host test does not link this.
pub fn panel_power(i2c: &mut I2cDriver<'_>) -> bool {
    hold_sd_cs();
    for _ in 0..3 {
        if supply_once(i2c) {
            log::info!("panel supply on");
            return true;
        }
        FreeRtos::delay_ms(20);
    }
    log::error!("panel supply failed");
    false
}

fn supply_once(i2c: &mut I2cDriver<'_>) -> bool {
    let ok = (|| {
        let id = axp_read(i2c, AXP_ID_REG)?;
        if id != AXP_ID_VALUE {
            log::error!("axp192 id {id:#04x}");
            return None;
        }
        let reg = axp_read(i2c, AXP_LDO_REG)?;
        if !axp_write(i2c, AXP_LDO_REG, (reg & 0x0f) | AXP_LDO2_3V3) {
            return None;
        }
        let reg = axp_read(i2c, AXP_DCDC3_REG)?;
        if !axp_write(i2c, AXP_DCDC3_REG, (reg & 0x80) | AXP_DCDC3_2V8) {
            return None;
        }
        let reg = axp_read(i2c, AXP_OUT_REG)?;
        if !axp_write(i2c, AXP_OUT_REG, reg | AXP_OUT_DCDC3 | AXP_OUT_LDO2) {
            return None;
        }
        // NS4168 enable is active high. Clear it. Do not start I2S.
        let reg = axp_read(i2c, AXP_GPIO_LEVEL_REG)?;
        if !axp_write(i2c, AXP_GPIO_LEVEL_REG, reg & !AXP_GPIO2_HIGH) {
            return None;
        }
        // REG 95H bit 7 selects GPIO mode. Bits 3:2 = 01 make GPIO4 NMOS open-drain.
        let reg = axp_read(i2c, AXP_GPIO4_FN_REG)?;
        if !axp_write(i2c, AXP_GPIO4_FN_REG, (reg & 0x72) | 0x84) {
            return None;
        }
        let reg = axp_read(i2c, AXP_GPIO4_LEVEL_REG)?;
        if !axp_write(i2c, AXP_GPIO4_LEVEL_REG, reg & !AXP_GPIO4_RELEASE) {
            return None;
        }
        FreeRtos::delay_ms(10);
        if !axp_write(i2c, AXP_GPIO4_LEVEL_REG, reg | AXP_GPIO4_RELEASE) {
            return None;
        }
        FreeRtos::delay_ms(120);
        Some(())
    })()
    .is_some();
    ok
}

/// Puts 5 V on the M-Bus for the Module HMI (P-0055).
/// USB present passes that 5 V through. Otherwise the AXP192 boost
/// feeds the bus. This does not write REG 0x30 and does not record
/// a current limit.
pub fn mbus_power(i2c: &mut I2cDriver<'_>) -> bool {
    let Some(status) = axp_read(i2c, AXP_STATUS_REG) else {
        log::error!("mbus 5v failed");
        return false;
    };
    let usb = status & AXP_VBUS_VHOLD != 0;
    let ok = if usb {
        bus_from_usb(i2c)
    } else {
        bus_from_boost(i2c)
    };
    if !ok {
        log::error!("mbus 5v failed");
        return false;
    }
    // The rail needs a moment. The STM32 itself is polled later (P-0055).
    FreeRtos::delay_ms(20);
    if usb {
        log::info!("mbus 5v usb {status:#04x}");
    } else {
        log::info!("mbus 5v boost {status:#04x}");
    }
    true
}

/// N_VBUSEN floats, so the board pulldown connects USB 5 V to the bus.
/// The boost stays off.
fn bus_from_usb(i2c: &mut I2cDriver<'_>) -> bool {
    let Some(reg) = axp_read(i2c, AXP_EXTEN_REG) else {
        return false;
    };
    if !axp_write(i2c, AXP_EXTEN_REG, reg & !AXP_EXTEN) {
        return false;
    }
    let Some(reg) = axp_read(i2c, AXP_GPIO0_FN_REG) else {
        return false;
    };
    axp_write(i2c, AXP_GPIO0_FN_REG, (reg & 0xf8) | AXP_GPIO0_FLOAT)
}

/// GPIO0 pulls N_VBUSEN up, and EXTEN turns the 5 V boost on.
fn bus_from_boost(i2c: &mut I2cDriver<'_>) -> bool {
    let Some(reg) = axp_read(i2c, AXP_GPIO0_VOLT_REG) else {
        return false;
    };
    if !axp_write(i2c, AXP_GPIO0_VOLT_REG, (reg & 0x0f) | 0xf0) {
        return false;
    }
    let Some(reg) = axp_read(i2c, AXP_GPIO0_FN_REG) else {
        return false;
    };
    if !axp_write(i2c, AXP_GPIO0_FN_REG, (reg & 0xf8) | AXP_GPIO0_LDO) {
        return false;
    }
    let Some(reg) = axp_read(i2c, AXP_EXTEN_REG) else {
        return false;
    };
    axp_write(i2c, AXP_EXTEN_REG, reg | AXP_EXTEN)
}
