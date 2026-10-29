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

//! ILI9342C on the Core2 v1.3 SPI bus. SCK GPIO18, MOSI GPIO23, MISO
//! GPIO38, CS GPIO5, DC GPIO15. Reset is the AXP192, already released
//! by `panel_power`. The command list is the ILI9342C sequence
//! (external command 0xC8, then the power and gamma registers).

use std::sync::Mutex;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::sys;

use crate::font::text_covers;
use crate::seam::Rect;
use crate::seam::Status;
use crate::ui::{DrawItem, Ui, DRAW_CAP};

const LCD_WIDTH: i32 = 320;
const LCD_HEIGHT: i32 = 240;
const LCD_SCLK: i32 = 18;
const LCD_MOSI: i32 = 23;
const LCD_MISO: i32 = 38;
const LCD_CS: i32 = 5;
const LCD_DC: i32 = 15;

static LCD_IO: Mutex<usize> = Mutex::new(0);

fn io_handle() -> sys::esp_lcd_panel_io_handle_t {
    let slot = LCD_IO.lock().unwrap_or_else(|e| e.into_inner());
    *slot as sys::esp_lcd_panel_io_handle_t
}

fn set_io(handle: sys::esp_lcd_panel_io_handle_t) {
    let mut slot = LCD_IO.lock().unwrap_or_else(|e| e.into_inner());
    *slot = handle as usize;
}

fn lcd_cmd(cmd: u8, data: &[u8]) -> bool {
    let ptr = if data.is_empty() {
        core::ptr::null()
    } else {
        data.as_ptr()
    };
    unsafe {
        sys::esp_lcd_panel_io_tx_param(io_handle(), cmd as i32, ptr.cast(), data.len())
            == sys::ESP_OK
    }
}

fn lcd_cmd1(cmd: u8, value: u8) -> bool {
    lcd_cmd(cmd, &[value])
}

fn ili9342c_init() -> bool {
    let extc = [0xFFu8, 0x93, 0x42];
    let pwctr1 = [0x12u8, 0x12];
    let ifctl = [0x01u8, 0x00, 0x00];
    let gamma_p = [
        0x00u8, 0x0C, 0x11, 0x04, 0x11, 0x08, 0x37, 0x89, 0x4C, 0x06, 0x0C, 0x0A, 0x2E, 0x34, 0x0F,
    ];
    let gamma_n = [
        0x00u8, 0x0B, 0x11, 0x05, 0x13, 0x09, 0x33, 0x67, 0x48, 0x07, 0x0E, 0x0B, 0x2E, 0x33, 0x0F,
    ];
    let dfun = [0x08u8, 0x82, 0x1D, 0x04];

    if !lcd_cmd(0x01, &[]) {
        return false;
    }
    FreeRtos::delay_ms(150);
    if !lcd_cmd(0xC8, &extc)
        || !lcd_cmd(0xC0, &pwctr1)
        || !lcd_cmd1(0xC1, 0x03)
        || !lcd_cmd1(0xC5, 0xF2)
        || !lcd_cmd1(0xB0, 0xE0)
        || !lcd_cmd(0xF6, &ifctl)
        || !lcd_cmd(0xE0, &gamma_p)
        || !lcd_cmd(0xE1, &gamma_n)
        || !lcd_cmd(0xB6, &dfun)
    {
        return false;
    }
    if !lcd_cmd(0x20, &[]) || !lcd_cmd(0x29, &[]) || !lcd_cmd(0x11, &[]) {
        return false;
    }
    FreeRtos::delay_ms(120);
    // 16-bit pixels, BGR, inversion on. That matches the Core2 glass.
    lcd_cmd1(0x3A, 0x55) && lcd_cmd1(0x36, 0x08) && lcd_cmd(0x21, &[])
}

fn lcd_bus() -> bool {
    let bus = sys::spi_bus_config_t {
        mosi_io_num: LCD_MOSI,
        miso_io_num: LCD_MISO,
        sclk_io_num: LCD_SCLK,
        quadwp_io_num: -1,
        quadhd_io_num: -1,
        max_transfer_sz: LCD_WIDTH * 2,
        ..Default::default()
    };
    let io_cfg = sys::esp_lcd_panel_io_spi_config_t {
        cs_gpio_num: LCD_CS,
        dc_gpio_num: LCD_DC,
        spi_mode: 0,
        pclk_hz: 40 * 1000 * 1000,
        trans_queue_depth: 1,
        lcd_cmd_bits: 8,
        lcd_param_bits: 8,
        ..Default::default()
    };
    unsafe {
        if sys::spi_bus_initialize(sys::spi_host_device_t_SPI3_HOST, &bus, sys::SPI_DMA_CH_AUTO)
            != sys::ESP_OK
        {
            return false;
        }
        let mut handle: sys::esp_lcd_panel_io_handle_t = core::ptr::null_mut();
        if sys::esp_lcd_new_panel_io_spi(
            sys::spi_host_device_t_SPI3_HOST as sys::esp_lcd_spi_bus_handle_t,
            &io_cfg,
            &mut handle,
        ) != sys::ESP_OK
        {
            sys::spi_bus_free(sys::spi_host_device_t_SPI3_HOST);
            return false;
        }
        set_io(handle);
        if !ili9342c_init() {
            sys::esp_lcd_panel_io_del(handle);
            sys::spi_bus_free(sys::spi_host_device_t_SPI3_HOST);
            set_io(core::ptr::null_mut());
            return false;
        }
    }
    true
}

fn inside(rect: &Rect, x: i32, y: i32) -> bool {
    x >= rect.x as i32
        && y >= rect.y as i32
        && x < rect.x as i32 + rect.w as i32
        && y < rect.y as i32 + rect.h as i32
}

fn paint(ui: &mut Ui, status: &Status) -> bool {
    let mut items = [DrawItem {
        rect: Rect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        },
        fill: 0,
        text: crate::seam::FixedText::new(),
        focused: false,
    }; DRAW_CAP];
    let nitems = ui.draw(status, &mut items);
    if nitems == 0 {
        return false;
    }
    let mut line = [0u8; (LCD_WIDTH as usize) * 2];
    for y in 0..LCD_HEIGHT {
        for x in 0..LCD_WIDTH {
            let mut color = items[0].fill;
            let mut ink = text_covers(items[0].text.as_str(), &items[0].rect, x, y);
            for item in items.iter().take(nitems).skip(1) {
                if inside(&item.rect, x, y) {
                    color = item.fill;
                    ink = text_covers(item.text.as_str(), &item.rect, x, y);
                }
            }
            if ink {
                // Light fills, the accent block, take black ink. Dark fills take white.
                color = if color & 0x07E0 > 0x03E0 {
                    0x0000
                } else {
                    0xFFFF
                };
            }
            let xi = x as usize;
            line[xi * 2] = (color >> 8) as u8;
            line[xi * 2 + 1] = color as u8;
        }
        let caset = [0u8, 0, ((LCD_WIDTH - 1) >> 8) as u8, (LCD_WIDTH - 1) as u8];
        let raset = [0u8, y as u8, 0, y as u8];
        if !lcd_cmd(0x2A, &caset) || !lcd_cmd(0x2B, &raset) {
            return false;
        }
        unsafe {
            if sys::esp_lcd_panel_io_tx_color(io_handle(), 0x2C, line.as_ptr().cast(), line.len())
                != sys::ESP_OK
            {
                return false;
            }
        }
        // tx_color returns before DMA finishes. A following command waits.
        if !lcd_cmd(0x00, &[]) {
            return false;
        }
    }
    true
}

/// Paints the current screen on the ILI9342C. The host test does not link this.
pub fn present(ui: &mut Ui, status: &Status) -> bool {
    if io_handle().is_null() && !lcd_bus() {
        log::error!("ili9342c init failed");
        return false;
    }
    if !paint(ui, status) {
        log::error!("ili9342c paint failed");
        return false;
    }
    log::info!("panel painted");
    true
}
