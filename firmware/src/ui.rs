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

use crate::seam::{
    Action, ActionKind, Command, CommandKind, FixedText, Queue, Rect, Status, LABEL_LEN, TITLE_LEN,
    VOLUME_MAX, VOLUME_STEP,
};

pub const JOINING_TITLE: &str = "Joining Wi-Fi";
const NOT_ON_NETWORK: &str = "Not on a network";

pub const DRAW_CAP: usize = 8;

const COLOR_BG: u16 = 0x18E3;
const COLOR_SURFACE: u16 = 0x2965;
const COLOR_ACCENT: u16 = 0x3BFF;
const BLOCK_X: i16 = 16;
const BLOCK_W: i16 = 288;
const BLOCK_H: i16 = 28;
const BLOCK_GAP: i16 = 8;
const BLOCK_Y0: i16 = 48;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Now = 0,
    Connect,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Play = 1,
    Volume,
    OpenConnect,
    OpenSettings,
    Info,
}

#[derive(Clone, Copy, Debug)]
pub struct BlockView {
    pub rect: Rect,
    pub role: Role,
    pub label: FixedText<LABEL_LEN>,
}

#[derive(Clone, Copy, Debug)]
pub struct DrawItem {
    pub rect: Rect,
    pub fill: u16,
    pub text: FixedText<LABEL_LEN>,
    pub focused: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Ui {
    stack: [Screen; 3],
    depth: u8,
    pub focus: u8,
    pub shown_playing: bool,
    shown_volume: u8,
    painted_wifi: bool,
    painted_ready: bool,
    painted_title: FixedText<TITLE_LEN>,
    pick_line: FixedText<LABEL_LEN>,
}

impl Ui {
    pub fn init(status: &Status) -> Self {
        Self {
            stack: [Screen::Now, Screen::Now, Screen::Now],
            depth: 1,
            focus: 0,
            shown_playing: status.playing,
            shown_volume: status.volume,
            painted_wifi: status.wifi_associated,
            painted_ready: status.session_ready,
            painted_title: status.title,
            pick_line: label_from("Pick porta in Spotify"),
        }
    }

    /// The phone picks this Connect name. An empty name stays `porta`.
    pub fn set_device_name(&mut self, name: &str) {
        let name = if name.is_empty() { "porta" } else { name };
        self.pick_line.set(&format!("Pick {name} in Spotify"));
    }

    pub fn status_changed(&self, status: &Status) -> bool {
        self.painted_wifi != status.wifi_associated
            || self.painted_ready != status.session_ready
            || self.painted_title.as_str() != status.title.as_str()
    }

    pub fn mark_painted(&mut self, status: &Status) {
        self.painted_wifi = status.wifi_associated;
        self.painted_ready = status.session_ready;
        self.painted_title.set(status.title.as_str());
    }

    pub fn screen(&self) -> Screen {
        self.stack[self.depth as usize - 1]
    }

    pub fn shown_volume(&self) -> u8 {
        self.shown_volume
    }

    pub fn blocks(&self, status: &Status, out: &mut [BlockView]) -> usize {
        match self.screen() {
            Screen::Connect => {
                if out.len() < 2 {
                    return 0;
                }
                place(
                    &mut out[0],
                    0,
                    Role::Info,
                    if status.wifi_associated {
                        "Wi-Fi up"
                    } else {
                        "Wi-Fi down"
                    },
                );
                place(&mut out[1], 1, Role::Info, self.connect_line(status));
                2
            }
            Screen::Settings => {
                if out.is_empty() {
                    return 0;
                }
                place(&mut out[0], 0, Role::Info, "No entries");
                1
            }
            Screen::Now => {
                if out.len() < 4 {
                    return 0;
                }
                place(&mut out[0], 0, Role::Play, "");
                out[0].label = flanked(if self.shown_playing { "Pause" } else { "Play" });
                place(&mut out[1], 1, Role::Volume, "");
                out[1].label = vol_word(self.shown_volume);
                place(&mut out[2], 2, Role::OpenConnect, "Connect");
                place(&mut out[3], 3, Role::OpenSettings, "Settings");
                4
            }
        }
    }

    pub fn apply(&mut self, status: &Status, queue: &mut Queue, action: Action) {
        if queue.count == 0 {
            self.shown_playing = status.playing;
            self.shown_volume = status.volume;
        }
        let mut blocks = [empty_block(); 5];
        let nblocks = self.blocks(status, &mut blocks);
        if nblocks == 0 {
            return;
        }
        if self.focus as usize >= nblocks {
            self.focus = 0;
        }
        match action.kind {
            ActionKind::Next => {
                self.focus = (self.focus + 1) % nblocks as u8;
            }
            ActionKind::Previous => {
                self.focus = (self.focus + nblocks as u8 - 1) % nblocks as u8;
            }
            ActionKind::Focus => {
                if (action.index as usize) < nblocks {
                    self.focus = action.index;
                }
            }
            ActionKind::Back => {
                if self.depth > 1 {
                    self.depth -= 1;
                    self.focus = 0;
                }
            }
            ActionKind::Activate => match blocks[self.focus as usize].role {
                Role::Play => self.transport(status, queue, action.index),
                Role::Volume => self.volume_side(queue, action.index),
                Role::OpenConnect => self.push_screen(Screen::Connect),
                Role::OpenSettings => self.push_screen(Screen::Settings),
                Role::Info => {}
            },
            ActionKind::Button3 | ActionKind::Screen => {}
        }
    }

    pub fn draw(&mut self, status: &Status, out: &mut [DrawItem]) -> usize {
        if out.is_empty() {
            return 0;
        }
        let mut n = 0usize;
        out[n] = DrawItem {
            rect: Rect {
                x: 0,
                y: 0,
                w: 320,
                h: 240,
            },
            fill: COLOR_BG,
            text: label_from(self.headline(status)),
            focused: false,
        };
        n += 1;
        let mut blocks = [empty_block(); 5];
        let nblocks = self.blocks(status, &mut blocks);
        for i in 0..nblocks {
            if n >= out.len() {
                break;
            }
            out[n] = DrawItem {
                rect: blocks[i].rect,
                fill: if i == self.focus as usize {
                    COLOR_ACCENT
                } else {
                    COLOR_SURFACE
                },
                text: blocks[i].label,
                focused: i == self.focus as usize,
            };
            n += 1;
        }
        n
    }

    fn headline<'a>(&'a self, status: &'a Status) -> &'a str {
        if status.session_ready {
            status.title.as_str()
        } else if status.wifi_associated {
            self.pick_line.as_str()
        } else if status.title.as_str() == JOINING_TITLE {
            JOINING_TITLE
        } else {
            "Wi-Fi down"
        }
    }

    fn connect_line<'a>(&'a self, status: &'a Status) -> &'a str {
        if status.session_ready {
            status.title.as_str()
        } else if status.wifi_associated {
            self.pick_line.as_str()
        } else {
            NOT_ON_NETWORK
        }
    }

    /// Index 0 is play or pause. 1 is previous. 2 is next.
    /// Without a session the block does not start the tone.
    fn transport(&mut self, status: &Status, queue: &mut Queue, zone: u8) {
        if !status.session_ready {
            return;
        }
        match zone {
            1 => {
                send(queue, CommandKind::Previous, 0);
            }
            2 => {
                send(queue, CommandKind::Next, 0);
            }
            _ => {
                let next_playing = !self.shown_playing;
                let kind = if next_playing {
                    CommandKind::Play
                } else {
                    CommandKind::Pause
                };
                if send(queue, kind, 0) {
                    self.shown_playing = next_playing;
                }
            }
        }
    }

    /// Index 1 is quieter. Index 2 is louder. The center is the readout.
    fn volume_side(&mut self, queue: &mut Queue, zone: u8) {
        match zone {
            1 => self.bump(queue, -VOLUME_STEP),
            2 => self.bump(queue, VOLUME_STEP),
            _ => {}
        }
    }

    fn bump(&mut self, queue: &mut Queue, step: i8) {
        if step > 0 {
            if self.shown_volume >= VOLUME_MAX {
                return;
            }
            if send(queue, CommandKind::SetVolume, step) {
                let room = VOLUME_MAX - self.shown_volume;
                let add = if room < step as u8 { room } else { step as u8 };
                self.shown_volume += add;
            }
        } else if step < 0 {
            if self.shown_volume == 0 {
                return;
            }
            if send(queue, CommandKind::SetVolume, step) {
                let down = (-step) as u8;
                if self.shown_volume >= down {
                    self.shown_volume -= down;
                } else {
                    self.shown_volume = 0;
                }
            }
        }
    }

    fn push_screen(&mut self, screen: Screen) {
        if self.depth >= 3 {
            return;
        }
        self.stack[self.depth as usize] = screen;
        self.depth += 1;
        self.focus = 0;
    }
}

fn send(queue: &mut Queue, kind: CommandKind, step: i8) -> bool {
    queue.push(Command {
        kind,
        volume_step: step,
    })
}

fn place(block: &mut BlockView, index: usize, role: Role, label: &str) {
    block.rect = Rect {
        x: BLOCK_X,
        y: BLOCK_Y0 + index as i16 * (BLOCK_H + BLOCK_GAP),
        w: BLOCK_W,
        h: BLOCK_H,
    };
    block.role = role;
    block.label.set(label);
}

fn empty_block() -> BlockView {
    BlockView {
        rect: Rect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        },
        role: Role::Info,
        label: FixedText::new(),
    }
}

fn label_from(src: &str) -> FixedText<LABEL_LEN> {
    let mut text = FixedText::new();
    text.set(src);
    text
}

/// `<`, the word centered, `>`. Twenty-three columns, so the chevrons sit
/// on the block edges once the line itself is centered.
fn flanked(word: &str) -> FixedText<LABEL_LEN> {
    const WIDTH: usize = LABEL_LEN - 1;
    let mut bytes = [b' '; LABEL_LEN];
    let word_bytes = word.as_bytes();
    let word_len = word_bytes.len().min(WIDTH - 2);
    let left = (WIDTH - 2 - word_len) / 2;
    bytes[0] = b'<';
    bytes[1 + left..1 + left + word_len].copy_from_slice(&word_bytes[..word_len]);
    bytes[WIDTH - 1] = b'>';
    label_from(core::str::from_utf8(&bytes[..WIDTH]).unwrap())
}

fn vol_word(volume: u8) -> FixedText<LABEL_LEN> {
    let mut raw = [0u8; 8];
    raw[..4].copy_from_slice(b"vol ");
    let n = if volume >= 100 {
        raw[4] = b'1';
        raw[5] = b'0';
        raw[6] = b'0';
        7
    } else if volume >= 10 {
        raw[4] = b'0' + volume / 10;
        raw[5] = b'0' + volume % 10;
        6
    } else {
        raw[4] = b'0' + volume;
        5
    };
    flanked(core::str::from_utf8(&raw[..n]).unwrap())
}
