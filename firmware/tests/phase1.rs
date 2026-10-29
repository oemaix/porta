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

use porta::app::App;
use porta::input::{self, Hw, Input};
use porta::net;
use porta::player::Player;
use porta::seam::{Command, CommandKind, QUEUE_CAP, VOLUME_MAX};
use porta::sink::{self, RATE_44K, RATE_48K};
use porta::ui::{self, Screen, DRAW_CAP};

fn press(app: &mut App, button: usize) {
    let mut down = Hw::idle();
    down.button_down[button] = true;
    app.panel_step(&down);
    app.panel_step(&Hw::idle());
}

fn wheel(app: &mut App, steps: i32) {
    let mut hw = Hw::idle();
    hw.increment = steps;
    app.panel_step(&hw);
}

fn peak(stereo: &[i16], frames: usize) -> i16 {
    stereo[..frames * 2]
        .iter()
        .map(|v| v.saturating_abs())
        .max()
        .unwrap_or(0)
}

#[test]
fn phase1() {
    let mut failed: Vec<&str> = Vec::new();
    let mut expect = |cond: bool, name: &'static str| {
        if !cond {
            failed.push(name);
        }
    };

    let mut leds = [input::I2cWrite { reg: 0, value: 0 }; 2];
    expect(input::hmi_led_off(&mut leds) == 2, "led writes");
    expect(
        leds[0].reg == input::HMI_REG_LED1 && leds[0].value == 0,
        "led1 off",
    );
    expect(
        leds[1].reg == input::HMI_REG_LED2 && leds[1].value == 0,
        "led2 off",
    );
    expect(input::hmi_increment(&[2, 0, 0, 0]) == 2, "increment");
    let (valid, x, y) = input::ft6336_point(&[1, 0x01, 0x2c, 0x00, 0x10]);
    expect(valid && x == 300 && y == 16, "touch point");

    let mut app = App::init();
    expect(app.status.title.as_str() == "Tone stand-in", "title");
    expect(!app.status.session_ready, "session not ready");
    expect(!app.board.limit_measured(), "limit not measured");
    expect(app.board.bench_supply, "bench supply");
    let mut draw = [ui::DrawItem {
        rect: porta::seam::Rect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        },
        fill: 0,
        text: porta::seam::FixedText::new(),
        focused: false,
    }; DRAW_CAP];
    let n = app.ui.draw(&app.status, &mut draw);
    let item = draw[..n].iter().find(|item| item.focused);
    expect(
        item.is_some_and(|item| item.text.as_str() == "Play"),
        "focus play",
    );

    press(&mut app, 0);
    let mut pcm = [0i16; 128];
    app.audio_step(&mut pcm, 64);
    expect(app.status.playing, "playing");
    expect(peak(&pcm, 64) > 0, "tone");
    expect(app.status.title.as_str() == "Tone stand-in", "title stays");

    press(&mut app, 0);
    app.audio_step(&mut pcm, 64);
    expect(!app.status.playing, "paused");
    expect(peak(&pcm, 64) == 0, "silence");

    app.queue.push(Command {
        kind: CommandKind::Next,
        volume_step: 0,
    });
    Player::drain(&mut app.queue, &mut app.status);
    expect(
        app.status.title.as_str() == "Tone stand-in",
        "next is not a catalog",
    );

    wheel(&mut app, 1);
    press(&mut app, 0);
    press(&mut app, 0);
    app.audio_step(&mut pcm, 8);
    expect(app.status.volume == 70, "two louder steps");

    wheel(&mut app, 1);
    for _ in 0..7 {
        press(&mut app, 0);
    }
    app.audio_step(&mut pcm, 8);
    expect(app.status.volume == 0, "volume floor");

    let mut app = App::init();
    wheel(&mut app, 3);
    press(&mut app, 0);
    expect(app.ui.screen() == Screen::Connect, "connect");
    let mut blocks = [ui::BlockView {
        rect: porta::seam::Rect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        },
        role: ui::Role::Info,
        label: porta::seam::FixedText::new(),
    }; 5];
    let n = app.ui.blocks(&app.status, &mut blocks);
    expect(
        n == 2 && blocks[0].label.as_str() == "Wi-Fi down",
        "wifi down",
    );
    net::set_associated(&mut app.status, true);
    app.ui.blocks(&app.status, &mut blocks);
    expect(blocks[0].label.as_str() == "Wi-Fi up", "wifi up");
    expect(
        blocks[1].label.as_str() == "Tone stand-in",
        "login placeholder",
    );
    press(&mut app, 2);
    expect(app.ui.screen() == Screen::Connect, "button 3 ignored");
    press(&mut app, 1);
    expect(app.ui.screen() == Screen::Now, "back");
    press(&mut app, 1);
    expect(app.ui.screen() == Screen::Now, "back on root");

    wheel(&mut app, 4);
    press(&mut app, 0);
    expect(app.ui.screen() == Screen::Settings, "settings");
    let n = app.ui.blocks(&app.status, &mut blocks);
    expect(
        n == 1 && blocks[0].label.as_str() == "No entries",
        "no battery row",
    );
    press(&mut app, 1);
    expect(app.ui.screen() == Screen::Now, "leave settings");

    let n = app.ui.blocks(&app.status, &mut blocks);
    let _ = n;
    let mut hw = Hw::idle();
    hw.touch_valid = true;
    hw.x = blocks[3].rect.x + 4;
    hw.y = blocks[3].rect.y + 4;
    app.panel_step(&hw);
    expect(app.ui.focus == 3, "tap moves focus");
    app.panel_step(&Hw::idle());
    hw.touch_valid = true;
    hw.x = blocks[3].rect.x + 4;
    hw.y = blocks[3].rect.y + 4;
    app.panel_step(&hw);
    expect(app.ui.screen() == Screen::Connect, "tap activates");

    let mut app = App::init();
    let mut hw = Hw::idle();
    hw.touch_valid = true;
    hw.x = 2;
    hw.y = 2;
    app.panel_step(&hw);
    expect(app.ui.focus == 0, "edge ignored");
    app.panel_step(&Hw::idle());
    hw.touch_valid = true;
    hw.x = 10;
    hw.y = 260;
    app.panel_step(&hw);
    expect(app.queue.count == 0, "screen button dropped");
    expect(app.ui.screen() == Screen::Now, "screen button stays");

    expect(!app.sink.open(22050), "odd rate rejected");
    let mut resampled = [0i16; 32];
    expect(
        sink::resample_s16(&pcm, 441, RATE_44K, &mut resampled, 16, RATE_48K) == 0,
        "resample respects cap",
    );
    let mut input = [0i16; 441 * 2];
    let mut output = [0i16; 480 * 2];
    input[0] = 1000;
    expect(
        sink::resample_s16(&input, 441, RATE_44K, &mut output, 480, RATE_48K) == 480,
        "resample length",
    );
    expect(
        !app.sink.write(Some(&pcm), 4, RATE_48K),
        "wrong rate rejected",
    );
    expect(app.sink.frames == 0, "rejected write is not counted");

    let mut app = App::init();
    let mut hw = Hw::idle();
    hw.increment = 1;
    hw.button_down[0] = true;
    app.panel_step(&hw);
    app.audio_step(&mut pcm, 8);
    expect(app.status.playing, "press confirms the previous focus");
    expect(app.ui.focus == 1, "wheel step follows the press");

    let raw = [0u8, 1, 0];
    let mut down = [false; 3];
    input::hmi_buttons(&raw, &mut down);
    expect(!down[0] && down[1] && !down[2], "button registers");
    let mut app = App::init();
    wheel(&mut app, 3);
    press(&mut app, 0);
    let mut hw = Hw::idle();
    input::hmi_buttons(&raw, &mut hw.button_down);
    app.panel_step(&hw);
    expect(app.ui.screen() == Screen::Now, "register 0x21 goes back");

    let mut app = App::init();
    let mut hw = Hw::idle();
    hw.increment = 10;
    app.panel_step(&hw);
    expect(app.ui.focus == 3, "eight steps this poll");
    app.panel_step(&Hw::idle());
    expect(app.ui.focus == 0, "leftover steps wait");

    let mut app = App::init();
    app.status.volume = VOLUME_MAX;
    wheel(&mut app, 1);
    press(&mut app, 0);
    expect(app.queue.count == 0, "louder at the ceiling is not queued");
    expect(app.status.volume == VOLUME_MAX, "ceiling held");

    let mut app = App::init();
    app.status.volume = 0;
    wheel(&mut app, 2);
    press(&mut app, 0);
    expect(app.queue.count == 0, "quieter at the floor is not queued");

    let mut app = App::init();
    let filler = Command {
        kind: CommandKind::Next,
        volume_step: 0,
    };
    for _ in 0..QUEUE_CAP {
        expect(app.queue.push(filler), "fill queue");
    }
    press(&mut app, 0);
    expect(
        !app.ui.shown_playing,
        "play label stays when the queue is full",
    );
    expect(app.queue.count == QUEUE_CAP as u8, "full queue unchanged");
    expect(!app.status.playing, "play was not applied");

    let mut held = Input::init();
    let mut hw = Hw::idle();
    hw.button_down[0] = true;
    expect(held.poll(&hw, &[], 0, &mut []) == 0, "no room for the edge");
    expect(!held.button_down[0], "edge stays armed");
    let mut one = [porta::seam::Action {
        kind: porta::seam::ActionKind::Next,
        index: 0,
    }; 1];
    expect(held.poll(&hw, &[], 0, &mut one) == 1, "edge retries");
    expect(
        one[0].kind == porta::seam::ActionKind::Activate,
        "retried press",
    );

    assert!(failed.is_empty(), "{}", failed.join(", "));
}
