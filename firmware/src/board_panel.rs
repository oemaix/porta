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

use esp_idf_svc::hal::delay::{FreeRtos, TickType};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::FromValueType;
use esp_idf_svc::sys;

use crate::board::{AXP192_ADDR, I2C_SCL_GPIO, I2C_SDA_GPIO};

const AXP_ID_REG: u8 = 0x03;
const AXP_ID_VALUE: u8 = 0x03;
const AXP_OUT_REG: u8 = 0x12;
const AXP_DCDC3_REG: u8 = 0x27;
const AXP_LDO_REG: u8 = 0x28;
const AXP_GPIO_LEVEL_REG: u8 = 0x94;
const AXP_GPIO4_FN_REG: u8 = 0x95;
const AXP_GPIO4_LEVEL_REG: u8 = 0x96;

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
    let ticks = TickType::new_millis(100).ticks();
    i2c.write_read(AXP192_ADDR, &[reg], &mut value, ticks)
        .ok()?;
    Some(value[0])
}

fn axp_write(i2c: &mut I2cDriver<'_>, reg: u8, value: u8) -> bool {
    let ticks = TickType::new_millis(100).ticks();
    i2c.write(AXP192_ADDR, &[reg, value], ticks).is_ok()
}

/// Enables the ILI9342C logic supply and backlight, and pulses LCD reset.
/// Does not write the VBUS current limit or set `limit_measured`.
/// The host test does not link this.
pub fn panel_power() -> bool {
    let _ = (I2C_SDA_GPIO, I2C_SCL_GPIO);
    hold_sd_cs();
    let peripherals = match Peripherals::take() {
        Ok(peripherals) => peripherals,
        Err(_) => {
            log::error!("i2c bus failed");
            return false;
        }
    };
    let config = I2cConfig::new()
        .baudrate(400_u32.kHz().into())
        .sda_enable_pullup(true)
        .scl_enable_pullup(true);
    let mut i2c = match I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &config,
    ) {
        Ok(i2c) => i2c,
        Err(_) => {
            log::error!("axp192 not at 0x34");
            return false;
        }
    };

    let ok = (|| {
        let id = axp_read(&mut i2c, AXP_ID_REG)?;
        if id != AXP_ID_VALUE {
            log::error!("axp192 id {id:#04x}");
            return None;
        }
        let reg = axp_read(&mut i2c, AXP_LDO_REG)?;
        if !axp_write(&mut i2c, AXP_LDO_REG, (reg & 0x0f) | AXP_LDO2_3V3) {
            return None;
        }
        let reg = axp_read(&mut i2c, AXP_DCDC3_REG)?;
        if !axp_write(&mut i2c, AXP_DCDC3_REG, (reg & 0x80) | AXP_DCDC3_2V8) {
            return None;
        }
        let reg = axp_read(&mut i2c, AXP_OUT_REG)?;
        if !axp_write(&mut i2c, AXP_OUT_REG, reg | AXP_OUT_DCDC3 | AXP_OUT_LDO2) {
            return None;
        }
        // NS4168 enable is active high. Clear it. Do not start I2S.
        let reg = axp_read(&mut i2c, AXP_GPIO_LEVEL_REG)?;
        if !axp_write(&mut i2c, AXP_GPIO_LEVEL_REG, reg & !AXP_GPIO2_HIGH) {
            return None;
        }
        // REG 95H bit 7 selects GPIO mode. Bits 3:2 = 01 make GPIO4 NMOS open-drain.
        let reg = axp_read(&mut i2c, AXP_GPIO4_FN_REG)?;
        if !axp_write(&mut i2c, AXP_GPIO4_FN_REG, (reg & 0x72) | 0x84) {
            return None;
        }
        let reg = axp_read(&mut i2c, AXP_GPIO4_LEVEL_REG)?;
        if !axp_write(&mut i2c, AXP_GPIO4_LEVEL_REG, reg & !AXP_GPIO4_RELEASE) {
            return None;
        }
        FreeRtos::delay_ms(10);
        if !axp_write(&mut i2c, AXP_GPIO4_LEVEL_REG, reg | AXP_GPIO4_RELEASE) {
            return None;
        }
        FreeRtos::delay_ms(120);
        Some(())
    })()
    .is_some();

    if ok {
        log::info!("panel supply on");
    } else {
        log::error!("panel supply failed");
    }
    ok
}
