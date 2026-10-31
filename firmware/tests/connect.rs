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
use porta::input::Hw;
use porta::net;
use porta::player::{CRED_CAP, TONE_TITLE, WAITING_TITLE};
use porta::seam::CommandKind;
use porta::ui::{self, Screen};

fn press(app: &mut App, button: usize) {
    let mut down = Hw::idle();
    down.button_down[button] = true;
    app.panel_step(&down);
    app.panel_step(&Hw::idle());
}

fn open_connect(app: &mut App) {
    let mut hw = Hw::idle();
    hw.increment = 2;
    app.panel_step(&hw);
    press(app, 0);
}

#[test]
fn credentials_wait_for_wifi() {
    let mut app = App::init();
    assert!(!app.player.store_credentials(&mut app.status, &[]));
    assert!(!app
        .player
        .store_credentials(&mut app.status, &[0u8; CRED_CAP + 1]));
    assert!(!app.player.has_credentials());
    let blob = b"zeroconf-blob";
    assert!(app.player.store_credentials(&mut app.status, blob));
    assert!(app.player.has_credentials());
    assert!(!app.status.session_ready);
    assert_eq!(app.status.title.as_str(), TONE_TITLE);
    assert_eq!(app.player.push_pcm(&app.status, &[1000, 1000]), 0);
    assert!(!app.player.note_track(&mut app.status, "Harbor"));

    net::set_associated(&mut app.status, true);
    app.player.sync(&mut app.status);
    assert!(app.status.session_ready);
    assert_eq!(app.status.title.as_str(), WAITING_TITLE);
    assert!(!app.player.note_track(&mut app.status, ""));
    assert_eq!(app.status.title.as_str(), WAITING_TITLE);
    assert!(app.player.note_track(&mut app.status, "Harbor"));
    assert_eq!(app.status.title.as_str(), "Harbor");
}

#[test]
fn pcm_replaces_the_tone() {
    let mut app = App::init();
    assert!(app
        .player
        .store_credentials(&mut app.status, b"zeroconf-blob"));
    net::set_associated(&mut app.status, true);
    app.player.sync(&mut app.status);
    assert_eq!(app.player.push_pcm(&app.status, &[1000, -2000]), 1);
    press(&mut app, 0);
    let mut pcm = [9i16; 4];
    app.audio_step(&mut pcm, 1);
    assert!(app.status.playing);
    assert_eq!(pcm[0], 500);
    assert_eq!(pcm[1], -1000);
    assert_eq!(
        app.player.take_session_command().map(|cmd| cmd.kind),
        Some(CommandKind::Play)
    );

    press(&mut app, 0);
    assert_eq!(app.player.push_pcm(&app.status, &[400, -400]), 1);
    app.audio_step(&mut pcm, 1);
    assert!(!app.status.playing);
    assert_eq!(&pcm[..2], &[0, 0]);
    press(&mut app, 0);
    app.audio_step(&mut pcm, 1);
    assert!(app.status.playing);
    assert_eq!(&pcm[..2], &[0, 0]);
    assert_eq!(app.player.push_pcm(&app.status, &[400, -400]), 1);
    app.audio_step(&mut pcm, 1);
    assert_eq!(pcm[0], 200);
    assert_eq!(pcm[1], -200);
}

#[test]
fn remote_playback_reaches_the_buffer() {
    let mut app = App::init();
    assert!(app.player.store_credentials(&mut app.status, b"blob"));
    net::set_associated(&mut app.status, true);
    app.player.sync(&mut app.status);
    assert_eq!(app.player.push_pcm(&app.status, &[1000, -2000]), 1);
    let mut pcm = [7i16; 8];
    let waiting = app.audio_step(&mut pcm, 4);
    assert!(!app.status.playing);
    assert_eq!(waiting.frames, 0);
    assert!(!waiting.keep);
    assert_eq!(&pcm[..2], &[0, 0]);

    app.player.apply_remote_pause(&mut app.status, false);
    let played = app.audio_step(&mut pcm, 4);
    assert!(app.status.playing);
    assert_eq!(played.frames, 1);
    assert!(played.keep);
    assert_eq!(pcm[0], 500);
    assert_eq!(pcm[1], -1000);
    assert_eq!(&pcm[2..8], &[0; 6]);

    app.player.apply_remote_pause(&mut app.status, true);
    let paused = app.audio_step(&mut pcm, 1);
    assert!(!app.status.playing);
    assert_eq!(paused.frames, 0);
    assert!(!paused.keep);
    assert_eq!(&pcm[..2], &[0, 0]);
    app.player.apply_remote_pause(&mut app.status, false);
    let empty = app.audio_step(&mut pcm, 1);
    assert_eq!(empty.frames, 0);
    assert!(empty.keep);
}

#[test]
fn skip_waits_for_the_next_title() {
    let mut app = App::init();
    assert!(app.player.store_credentials(&mut app.status, b"blob"));
    net::set_associated(&mut app.status, true);
    app.player.sync(&mut app.status);
    assert!(app.player.note_track(&mut app.status, "Harbor"));
    app.queue.push(porta::seam::Command {
        kind: CommandKind::Next,
        volume_step: 0,
    });
    app.audio_step(&mut [0i16; 4], 1);
    assert_eq!(app.status.title.as_str(), "Harbor");
    assert_eq!(
        app.player.take_session_command().map(|cmd| cmd.kind),
        Some(CommandKind::Next)
    );
    assert!(app.player.note_track(&mut app.status, "Other"));
    assert_eq!(app.status.title.as_str(), "Other");
}

#[test]
fn wifi_loss_returns_to_the_tone() {
    let mut app = App::init();
    assert!(app.player.store_credentials(&mut app.status, b"blob"));
    net::set_associated(&mut app.status, true);
    app.player.sync(&mut app.status);
    assert!(app.player.note_track(&mut app.status, "Harbor"));
    press(&mut app, 0);
    app.audio_step(&mut [0i16; 4], 1);
    assert!(app.status.playing);

    net::set_associated(&mut app.status, false);
    app.panel_step(&Hw::idle());
    assert!(!app.status.session_ready);
    assert!(!app.status.playing);
    assert!(!app.ui.shown_playing);
    assert_eq!(app.status.title.as_str(), TONE_TITLE);
    assert!(app.player.take_session_command().is_none());
    let mut pcm = [9i16; 4];
    app.audio_step(&mut pcm, 1);
    assert_eq!(&pcm[..2], &[0, 0]);
}

#[test]
fn connect_screen_shows_the_track() {
    let mut app = App::init();
    assert!(app.player.store_credentials(&mut app.status, b"blob"));
    net::set_associated(&mut app.status, true);
    let title = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123";
    app.player.sync(&mut app.status);
    assert!(app.player.note_track(&mut app.status, title));
    open_connect(&mut app);
    assert_eq!(app.ui.screen(), Screen::Connect);
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
    assert_eq!(n, 2);
    assert_eq!(blocks[0].label.as_str(), "Wi-Fi up");
    assert_eq!(blocks[1].label.as_str(), &title[..23]);
    assert_eq!(app.status.title.as_str(), title);
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
    }; ui::DRAW_CAP];
    app.ui.draw(&app.status, &mut draw);
    assert_eq!(draw[0].text.as_str(), &title[..23]);
}

#[test]
fn headlines_follow_the_station() {
    let mut app = App::init();
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
    }; ui::DRAW_CAP];
    app.ui.draw(&app.status, &mut draw);
    assert_eq!(draw[0].text.as_str(), "Wi-Fi down");
    app.status.title.set(ui::JOINING_TITLE);
    app.ui.draw(&app.status, &mut draw);
    assert_eq!(draw[0].text.as_str(), "Joining Wi-Fi");
    app.status.title.set(TONE_TITLE);
    net::set_associated(&mut app.status, true);
    app.ui.draw(&app.status, &mut draw);
    assert_eq!(draw[0].text.as_str(), "Pick porta in Spotify");
    app.ui.set_device_name("desk");
    app.ui.draw(&app.status, &mut draw);
    assert_eq!(draw[0].text.as_str(), "Pick desk in Spotify");
    app.ui.set_device_name("");
    app.ui.draw(&app.status, &mut draw);
    assert_eq!(draw[0].text.as_str(), "Pick porta in Spotify");
}
