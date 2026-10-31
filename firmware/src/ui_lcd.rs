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
//!
//! The frame is 150 KB in PSRAM. Blocks and glyphs are filled as areas.
//! SPI3 on this ESP32 cannot DMA from PSRAM, so each paint copies the
//! frame out through one internal DMA band at a time (P-0053).

use std::sync::Mutex;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::sys;

use crate::font::blit_text;
use crate::seam::Rect;
use crate::seam::Status;
use crate::ui::{DrawItem, Ui, DRAW_CAP};

const LCD_WIDTH: i32 = 320;
const LCD_HEIGHT: i32 = 240;
const ROW_BYTES: usize = (LCD_WIDTH as usize) * 2;
const FRAME_BYTES: usize = ROW_BYTES * (LCD_HEIGHT as usize);
/// Sixteen rows fit in internal DMA memory. 240 divides evenly.
const BAND_ROWS: i32 = 16;
const BAND_BYTES: usize = ROW_BYTES * (BAND_ROWS as usize);
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
        __bindgen_anon_1: sys::spi_bus_config_t__bindgen_ty_1 {
            mosi_io_num: LCD_MOSI,
        },
        __bindgen_anon_2: sys::spi_bus_config_t__bindgen_ty_2 {
            miso_io_num: LCD_MISO,
        },
        sclk_io_num: LCD_SCLK,
        __bindgen_anon_3: sys::spi_bus_config_t__bindgen_ty_3 { quadwp_io_num: -1 },
        __bindgen_anon_4: sys::spi_bus_config_t__bindgen_ty_4 { quadhd_io_num: -1 },
        max_transfer_sz: BAND_BYTES as i32,
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
        if sys::spi_bus_initialize(
            sys::spi_host_device_t_SPI3_HOST,
            &bus,
            sys::spi_common_dma_t_SPI_DMA_CH_AUTO,
        ) != sys::ESP_OK
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

#[derive(Clone, Copy)]
struct PanelBuf {
    frame: usize,
    bounce: usize,
}

fn panel_buf() -> Option<PanelBuf> {
    static SLOT: Mutex<Option<PanelBuf>> = Mutex::new(None);
    let mut slot = SLOT.lock().unwrap_or_else(|err| err.into_inner());
    if let Some(buf) = *slot {
        return Some(buf);
    }
    unsafe {
        let frame =
            sys::heap_caps_malloc(FRAME_BYTES, sys::MALLOC_CAP_SPIRAM | sys::MALLOC_CAP_8BIT);
        let bounce = sys::heap_caps_aligned_alloc(
            4,
            BAND_BYTES,
            sys::MALLOC_CAP_DMA | sys::MALLOC_CAP_INTERNAL,
        );
        if frame.is_null() || bounce.is_null() {
            if !frame.is_null() {
                sys::heap_caps_free(frame);
            }
            if !bounce.is_null() {
                sys::heap_caps_free(bounce);
            }
            log::error!("panel buffer failed");
            return None;
        }
        let buf = PanelBuf {
            frame: frame as usize,
            bounce: bounce as usize,
        };
        *slot = Some(buf);
        Some(buf)
    }
}

fn put_px(frame: &mut [u8], x: i32, y: i32, color: u16) {
    if x < 0 || y < 0 || x >= LCD_WIDTH || y >= LCD_HEIGHT {
        return;
    }
    let i = (y as usize * LCD_WIDTH as usize + x as usize) * 2;
    frame[i] = (color >> 8) as u8;
    frame[i + 1] = color as u8;
}

fn fill_rect(frame: &mut [u8], rect: Rect, color: u16) {
    let hi = (color >> 8) as u8;
    let lo = color as u8;
    let x0 = (rect.x as i32).clamp(0, LCD_WIDTH) as usize;
    let x1 = (rect.x as i32 + rect.w as i32).clamp(0, LCD_WIDTH) as usize;
    let y0 = (rect.y as i32).clamp(0, LCD_HEIGHT) as usize;
    let y1 = (rect.y as i32 + rect.h as i32).clamp(0, LCD_HEIGHT) as usize;
    for y in y0..y1 {
        let row = y * ROW_BYTES;
        for x in x0..x1 {
            let i = row + x * 2;
            frame[i] = hi;
            frame[i + 1] = lo;
        }
    }
}

/// Light fills take black ink. Dark fills take white.
fn ink_on(fill: u16) -> u16 {
    if fill & 0x07E0 > 0x03E0 {
        0x0000
    } else {
        0xFFFF
    }
}

fn compose(frame: &mut [u8], items: &[DrawItem]) {
    for item in items {
        fill_rect(frame, item.rect, item.fill);
        let ink = ink_on(item.fill);
        blit_text(item.text.as_str(), &item.rect, |x, y| {
            put_px(frame, x, y, ink);
        });
    }
}

fn flush(frame: &[u8], bounce: &mut [u8]) -> bool {
    let caset = [0u8, 0, ((LCD_WIDTH - 1) >> 8) as u8, (LCD_WIDTH - 1) as u8];
    if !lcd_cmd(0x2A, &caset) {
        return false;
    }
    let mut y = 0i32;
    while y < LCD_HEIGHT {
        let rows = BAND_ROWS.min(LCD_HEIGHT - y);
        let y0 = y as u16;
        let y1 = (y + rows - 1) as u16;
        let raset = [(y0 >> 8) as u8, y0 as u8, (y1 >> 8) as u8, y1 as u8];
        if !lcd_cmd(0x2B, &raset) {
            return false;
        }
        let start = y as usize * ROW_BYTES;
        let len = rows as usize * ROW_BYTES;
        bounce[..len].copy_from_slice(&frame[start..start + len]);
        unsafe {
            if sys::esp_lcd_panel_io_tx_color(io_handle(), 0x2C, bounce.as_ptr().cast(), len)
                != sys::ESP_OK
            {
                return false;
            }
        }
        // tx_color returns before DMA finishes. A following command waits.
        if !lcd_cmd(0x00, &[]) {
            return false;
        }
        y += rows;
    }
    true
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
    let Some(buf) = panel_buf() else {
        return false;
    };
    unsafe {
        let frame = std::slice::from_raw_parts_mut(buf.frame as *mut u8, FRAME_BYTES);
        let bounce = std::slice::from_raw_parts_mut(buf.bounce as *mut u8, BAND_BYTES);
        compose(frame, &items[..nitems]);
        flush(frame, bounce)
    }
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
