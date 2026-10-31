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

//! Module Audio headphone path. I2S0 is the master. The ES8388 at 0x10
//! is the slave. Configuration A uses MCLK GPIO0, BCLK GPIO19, LRCK
//! GPIO27, and data GPIO2. The cable pins stay free. The NS4168 stays
//! off. The register list is the M5Module-Audio init, then headphone
//! output 1 and analog volume 40. The panel volume scales the samples.

use std::sync::{Arc, Mutex};

use esp_idf_svc::hal::delay::{FreeRtos, TickType};
use esp_idf_svc::hal::gpio::{Gpio0, Gpio2, Gpio19, Gpio27};
use esp_idf_svc::hal::i2c::I2cDriver;
use esp_idf_svc::hal::i2s::config::{
    Config, DataBitWidth, SlotMode, StdClkConfig, StdConfig, StdGpioConfig, StdSlotConfig,
};
use esp_idf_svc::hal::i2s::{I2sDriver, I2sTx, I2S0};
use esp_idf_svc::hal::task::thread::ThreadSpawnConfiguration;
use esp_idf_svc::sys;

use crate::player::TONE_RATE;

/// Frames the jack task can hold. About 186 ms at 44100 Hz.
const QUEUE_FRAMES: usize = 8192;
/// The panel loop keeps the queue at least this full before it paints.
pub const MARK_FRAMES: usize = 6144;

const ES8388_ADDR: u8 = 0x10;
/// Vendor `setDACVolume(40)`. Steps are `(volume * 33 + 50) / 100`.
const ANALOG_VOLUME: u8 = 13;

/// M5Module-Audio `ES8388::init`, in that order. The headphone mixer
/// stays on the DAC. The speak sketch's ADC mix is a microphone loopback.
const INIT: &[(u8, u8)] = &[
    (0x08, 0x00),
    (0x02, 0xFF),
    (0x2B, 0x80),
    (0x00, 0x05),
    (0x01, 0x40),
    (0x03, 0x00),
    (0x0A, 0x00),
    (0x0B, 0x00),
    (0x09, 0x88),
    (0x0C, 0x2C),
    (0x0D, 0x02),
    (0x0F, 0x28),
    (0x10, 0x00),
    (0x11, 0x00),
    (0x12, 0xEA),
    (0x13, 0xC0),
    (0x14, 0x12),
    (0x15, 0x06),
    (0x16, 0xC3),
    (0x04, 0x3F),
    (0x17, 0x18),
    (0x18, 0x02),
    (0x19, 0x00),
    (0x1A, 0x05),
    (0x1B, 0x05),
    (0x26, 0x00),
    (0x27, 0x90),
    (0x28, 0x38),
    (0x29, 0x38),
    (0x2A, 0x90),
    (0x2B, 0x80),
    (0x2E, 0x12),
    (0x2F, 0x12),
    (0x30, 0x00),
    (0x31, 0x00),
    (0x02, 0x00),
];

pub struct AudioPins {
    pub i2s: I2S0<'static>,
    pub mclk: Gpio0<'static>,
    pub bclk: Gpio19<'static>,
    pub ws: Gpio27<'static>,
    pub dout: Gpio2<'static>,
}

pub struct Headphone {
    queue: Arc<Mutex<PcmQueue>>,
}

pub fn open(i2c: &mut I2cDriver<'_>, pins: AudioPins) -> Option<Headphone> {
    if !program(i2c) {
        log::error!("es8388 init failed");
        return None;
    }
    let config = StdConfig::new(
        Config::default().auto_clear(true),
        StdClkConfig::from_sample_rate_hz(TONE_RATE),
        StdSlotConfig::philips_slot_default(DataBitWidth::Bits16, SlotMode::Stereo),
        StdGpioConfig::default(),
    );
    let i2s = match I2sDriver::new_std_tx(
        pins.i2s,
        &config,
        pins.bclk,
        pins.dout,
        Some(pins.mclk),
        pins.ws,
    ) {
        Ok(i2s) => i2s,
        Err(err) => {
            log::error!("i2s0 {err}");
            return None;
        }
    };
    let queue = Arc::new(Mutex::new(PcmQueue::alloc(QUEUE_FRAMES)?));
    let task_queue = Arc::clone(&queue);
    let mut spawn = ThreadSpawnConfiguration::default();
    // The pthread default keeps this stack in internal RAM. It is not a PSRAM stack.
    spawn.name = Some(c"i2s");
    spawn.stack_size = 4096;
    spawn.priority = 5;
    spawn.inherit = false;
    spawn.pin_to_core = None;
    if let Err(err) = spawn.set() {
        log::error!("i2s task {err}");
        return None;
    }
    if std::thread::Builder::new()
        .spawn(move || clock(i2s, task_queue))
        .is_err()
    {
        log::error!("i2s task failed");
        return None;
    }
    log::info!("headphone i2s0 {TONE_RATE} hz");
    Some(Headphone { queue })
}

impl Headphone {
    pub fn queued_frames(&self) -> usize {
        self.lock().queued_frames()
    }

    pub fn push(&self, samples: &[i16]) {
        self.lock().push(samples);
    }

    pub fn flush(&self) {
        self.lock().flush();
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, PcmQueue> {
        self.queue.lock().unwrap_or_else(|err| err.into_inner())
    }
}

fn clock(mut i2s: I2sDriver<'static, I2sTx>, queue: Arc<Mutex<PcmQueue>>) {
    let mut chunk = [0i16; 256];
    loop {
        let n = {
            let mut queue = queue.lock().unwrap_or_else(|err| err.into_inner());
            queue.pop(&mut chunk)
        };
        if n == 0 {
            FreeRtos::delay_ms(4);
            continue;
        }
        write_frames(&mut i2s, &chunk[..n]);
    }
}

fn write_frames(i2s: &mut I2sDriver<'static, I2sTx>, samples: &[i16]) {
    if samples.len() < 2 {
        return;
    }
    // ESP32 is little-endian. Philips 16-bit slots consume these bytes in order.
    let bytes = unsafe {
        core::slice::from_raw_parts(samples.as_ptr().cast::<u8>(), samples.len() * 2)
    };
    let ticks = TickType::new_millis(50).ticks();
    let mut off = 0;
    while off + 4 <= bytes.len() {
        match i2s.write(&bytes[off..], ticks) {
            Ok(n) if n >= 4 => off += n & !3,
            Ok(_) => FreeRtos::delay_ms(1),
            Err(err) if err.code() == sys::ESP_ERR_TIMEOUT => FreeRtos::delay_ms(1),
            Err(err) => {
                log::warn!("i2s write {err}");
                return;
            }
        }
    }
}

struct PcmQueue {
    ptr: *mut i16,
    cap: usize,
    read: usize,
    len: usize,
}

// The pointer is the PSRAM queue. Only the mutex moves it between tasks.
unsafe impl Send for PcmQueue {}

impl PcmQueue {
    fn alloc(frames: usize) -> Option<Self> {
        let cap = frames * 2;
        let ptr = unsafe {
            sys::heap_caps_malloc(cap * 2, sys::MALLOC_CAP_SPIRAM | sys::MALLOC_CAP_8BIT)
        };
        if ptr.is_null() {
            log::error!("pcm queue alloc failed");
            return None;
        }
        Some(Self {
            ptr: ptr.cast(),
            cap,
            read: 0,
            len: 0,
        })
    }

    fn queued_frames(&self) -> usize {
        self.len / 2
    }

    fn push(&mut self, src: &[i16]) {
        let n = src.len().min(self.cap - self.len) & !1;
        for i in 0..n {
            let at = (self.read + self.len) % self.cap;
            unsafe { self.ptr.add(at).write(src[i]) };
            self.len += 1;
        }
    }

    fn pop(&mut self, dst: &mut [i16]) -> usize {
        let n = dst.len().min(self.len) & !1;
        for i in 0..n {
            dst[i] = unsafe { self.ptr.add(self.read).read() };
            self.read = (self.read + 1) % self.cap;
            self.len -= 1;
        }
        n
    }

    fn flush(&mut self) {
        self.read = 0;
        self.len = 0;
    }
}

impl Drop for PcmQueue {
    fn drop(&mut self) {
        unsafe { sys::heap_caps_free(self.ptr.cast()) };
    }
}

fn program(i2c: &mut I2cDriver<'_>) -> bool {
    for _ in 0..8 {
        if write_all(i2c) {
            return true;
        }
        FreeRtos::delay_ms(100);
    }
    false
}

fn write_all(i2c: &mut I2cDriver<'_>) -> bool {
    for (reg, value) in INIT {
        if !write_reg(i2c, *reg, *value) {
            return false;
        }
    }
    // Headphone output 1, analog volume 40, and 44.1 kHz with MCLK/256.
    write_reg(i2c, 0x04, 0x30)
        && write_reg(i2c, 0x2E, ANALOG_VOLUME)
        && write_reg(i2c, 0x2F, ANALOG_VOLUME)
        && write_reg(i2c, 0x08, 0x40)
        && write_reg(i2c, 0x0D, 0x02)
        && write_reg(i2c, 0x18, 0x02)
}

fn write_reg(i2c: &mut I2cDriver<'_>, reg: u8, value: u8) -> bool {
    let ticks = TickType::new_millis(40).ticks();
    match i2c.write(ES8388_ADDR, &[reg, value], ticks) {
        Ok(()) => true,
        Err(err) => {
            log::error!("es8388 {reg:#04x} {err}");
            false
        }
    }
}
