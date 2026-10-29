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
pub mod net;
pub mod player;
pub mod seam;
pub mod sink;
pub mod ui;

#[cfg(target_os = "espidf")]
mod board_panel;
#[cfg(target_os = "espidf")]
mod ui_lcd;

#[cfg(target_os = "espidf")]
pub fn firmware_main() {
    use esp_idf_svc::log::EspLogger;

    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();
    let mut app = app::App::init();
    // One painted frame. This entry does not start I2S or Wi-Fi.
    // An unmeasured AXP192 limit does not forbid them.
    if !board_panel::panel_power() {
        log::error!("panel power failed");
        return;
    }
    if !ui_lcd::present(&mut app.ui, &app.status) {
        log::error!("panel draw failed");
    }
}
