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

pub mod app;
pub mod board;
pub mod font;
pub mod input;
pub mod local_env;
pub mod net;
pub mod player;
pub mod seam;
pub mod sink;
pub mod ui;

#[cfg(target_os = "espidf")]
mod board_input;
#[cfg(target_os = "espidf")]
mod board_panel;
#[cfg(target_os = "espidf")]
mod board_wifi;
#[cfg(target_os = "espidf")]
mod ui_lcd;

#[cfg(target_os = "espidf")]
pub fn firmware_main() {
    use esp_idf_svc::hal::delay::FreeRtos;
    use esp_idf_svc::log::EspLogger;

    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();
    let mut app = app::App::init();
    // The same I2C driver feeds the AXP192, the HMI, and the touch controller.
    // Wi-Fi starts after the panel is up. I2S stays off.
    // An unmeasured AXP192 limit does not forbid Wi-Fi.
    let mut opened = match board_panel::open_i2c() {
        Some(opened) => opened,
        None => return,
    };
    if !board_panel::panel_power(&mut opened.i2c) {
        log::error!("panel power failed");
        return;
    }
    board_panel::mbus_power(&mut opened.i2c);
    board_input::leds_off(&mut opened.i2c);
    app.ui.set_device_name(local_env::load().device_name);
    app.status.title.set(ui::JOINING_TITLE);
    if !ui_lcd::present(&mut app.ui, &app.status) {
        log::error!("panel draw failed");
        return;
    }
    if board_wifi::join(opened.modem) {
        crate::net::set_associated(&mut app.status, true);
    }
    app.status.title.set(player::TONE_TITLE);
    if !ui_lcd::present(&mut app.ui, &app.status) {
        log::error!("panel draw failed");
    }
    let i2c = &mut opened.i2c;
    let mut bus = board_input::BusState::new();
    let mut pcm = [0i16; 1024];
    loop {
        if board_wifi::logged_in() && !app.player.has_credentials() {
            let _ = app.player.store_credentials(&mut app.status, b"session");
        }
        if let Some(title) = board_wifi::take_title() {
            let _ = app.player.note_track(&mut app.status, &title);
        }
        let frames = board_wifi::take_pcm(&mut pcm);
        if frames > 0 {
            let _ = app.player.push_pcm(&app.status, &pcm[..frames * 2]);
        }
        app.audio_step(&mut pcm, 256);
        while let Some(command) = app.player.take_session_command() {
            board_wifi::command(command.kind as i32, app.status.volume as i32);
        }
        let hw = board_input::sample(i2c, &mut bus);
        if app.panel_step(&hw) && !ui_lcd::present(&mut app.ui, &app.status) {
            log::error!("panel draw failed");
        }
        FreeRtos::delay_ms(20);
    }
}
