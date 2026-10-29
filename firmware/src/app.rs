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

use crate::board::Board;
use crate::input::{Hw, Input};
use crate::player::{Player, TONE_RATE};
use crate::seam::{Action, ActionKind, FixedText, Queue, Rect, Status};
use crate::sink::Sink;
use crate::ui::{BlockView, Role, Ui};

pub struct App {
    pub ui: Ui,
    pub input: Input,
    pub player: Player,
    pub sink: Sink,
    pub board: Board,
    pub status: Status,
    pub queue: Queue,
}

impl App {
    pub fn init() -> Self {
        let mut status = Status::new();
        let player = Player::init(&mut status);
        let ui = Ui::init(&status);
        let mut board = Board::init();
        board.use_bench();
        let mut sink = Sink::init();
        sink.open(TONE_RATE);
        Self {
            ui,
            input: Input::init(),
            player,
            sink,
            board,
            status,
            queue: Queue::new(),
        }
    }

    pub fn panel_step(&mut self, hw: &Hw) {
        let mut blocks = [BlockView {
            rect: Rect {
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
            role: Role::Info,
            label: FixedText::new(),
        }; 5];
        let nblocks = self.ui.blocks(&self.status, &mut blocks);
        let mut rects = [Rect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        }; 5];
        for i in 0..nblocks {
            rects[i] = blocks[i].rect;
        }
        let mut actions = [Action {
            kind: ActionKind::Next,
            index: 0,
        }; 8];
        let nactions = self
            .input
            .poll(hw, &rects[..nblocks], self.ui.focus, &mut actions);
        for action in actions.iter().take(nactions) {
            self.ui.apply(&self.status, &mut self.queue, *action);
        }
    }

    pub fn audio_step(&mut self, stereo: &mut [i16], frames: usize) {
        // Software PCM only. This step does not start the codec.
        Player::drain(&mut self.queue, &mut self.status);
        let rate = self.sink.rate_hz;
        self.player.render(&self.status, stereo, frames, rate);
        self.sink.write(Some(stereo), frames, rate);
    }
}
