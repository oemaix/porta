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
    Action, ActionKind, Command, CommandKind, FixedText, Queue, Rect, Status, LABEL_LEN,
    VOLUME_MAX, VOLUME_STEP,
};

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
    Louder,
    Quieter,
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
    volume_text: FixedText<LABEL_LEN>,
}

impl Ui {
    pub fn init(status: &Status) -> Self {
        Self {
            stack: [Screen::Now, Screen::Now, Screen::Now],
            depth: 1,
            focus: 0,
            shown_playing: status.playing,
            shown_volume: status.volume,
            volume_text: FixedText::new(),
        }
    }

    pub fn screen(&self) -> Screen {
        self.stack[self.depth as usize - 1]
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
                place(&mut out[1], 1, Role::Info, "Tone stand-in");
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
                if out.len() < 5 {
                    return 0;
                }
                place(
                    &mut out[0],
                    0,
                    Role::Play,
                    if self.shown_playing { "Pause" } else { "Play" },
                );
                place(&mut out[1], 1, Role::Louder, "Louder");
                place(&mut out[2], 2, Role::Quieter, "Quieter");
                place(&mut out[3], 3, Role::OpenConnect, "Connect");
                place(&mut out[4], 4, Role::OpenSettings, "Settings");
                5
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
                Role::Play => {
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
                Role::Louder => {
                    if self.shown_volume >= VOLUME_MAX {
                        return;
                    }
                    if send(queue, CommandKind::SetVolume, VOLUME_STEP) {
                        let next = if self.shown_volume <= VOLUME_MAX - VOLUME_STEP as u8 {
                            self.shown_volume + VOLUME_STEP as u8
                        } else {
                            VOLUME_MAX
                        };
                        self.shown_volume = next;
                    }
                }
                Role::Quieter => {
                    if self.shown_volume == 0 {
                        return;
                    }
                    if send(queue, CommandKind::SetVolume, -VOLUME_STEP) {
                        if self.shown_volume >= VOLUME_STEP as u8 {
                            self.shown_volume -= VOLUME_STEP as u8;
                        } else {
                            self.shown_volume = 0;
                        }
                    }
                }
                Role::OpenConnect => self.push_screen(Screen::Connect),
                Role::OpenSettings => self.push_screen(Screen::Settings),
                Role::Info => {}
            },
            ActionKind::Button3 | ActionKind::Screen => {}
        }
    }

    pub fn draw(&mut self, status: &Status, out: &mut [DrawItem]) -> usize {
        if out.len() < 2 {
            return 0;
        }
        self.volume_text
            .set(&format!("Volume {}", self.shown_volume));
        let mut n = 0usize;
        out[n] = DrawItem {
            rect: Rect {
                x: 0,
                y: 0,
                w: 320,
                h: 240,
            },
            fill: COLOR_BG,
            text: label_from(status.title.as_str()),
            focused: false,
        };
        n += 1;
        out[n] = DrawItem {
            rect: Rect {
                x: BLOCK_X,
                y: 12,
                w: BLOCK_W,
                h: 28,
            },
            fill: COLOR_BG,
            text: self.volume_text,
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
