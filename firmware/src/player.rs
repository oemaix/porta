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

use crate::seam::{CommandKind, Queue, Status, VOLUME_MAX};

pub const TONE_HZ: u32 = 440;
pub const TONE_RATE: u32 = 44100;

#[derive(Clone, Copy, Debug)]
pub struct Player {
    phase: u32,
}

impl Player {
    pub fn init(status: &mut Status) -> Self {
        status.title.set("Tone stand-in");
        status.playing = false;
        status.volume = 50;
        status.session_ready = false;
        Self { phase: 0 }
    }

    pub fn drain(queue: &mut Queue, status: &mut Status) {
        while let Some(command) = queue.pop() {
            match command.kind {
                CommandKind::Play => status.playing = true,
                CommandKind::Pause => status.playing = false,
                CommandKind::Next | CommandKind::Previous => {}
                CommandKind::SetVolume => {
                    let mut volume = status.volume as i32 + command.volume_step as i32;
                    if volume < 0 {
                        volume = 0;
                    } else if volume > VOLUME_MAX as i32 {
                        volume = VOLUME_MAX as i32;
                    }
                    status.volume = volume as u8;
                }
            }
        }
    }

    /// Interleaved stereo s16. Silence while paused or at volume 0.
    /// The samples are a triangle labelled by the status title, not a track.
    pub fn render(&mut self, status: &Status, stereo: &mut [i16], frames: usize, rate_hz: u32) {
        let frames = frames.min(stereo.len() / 2);
        if frames == 0 {
            return;
        }
        let step = if rate_hz == 0 {
            0
        } else {
            (((TONE_HZ as u64) << 32) / rate_hz as u64) as u32
        };
        for i in 0..frames {
            let mut sample = 0i32;
            if status.playing && status.volume > 0 && step != 0 {
                let saw = (self.phase >> 16) as u16;
                let mut tri = if saw < 32768 {
                    saw as i32
                } else {
                    (65535 - saw) as i32
                };
                tri -= 16384;
                sample = tri * status.volume as i32 / 256;
                self.phase = self.phase.wrapping_add(step);
            }
            stereo[i * 2] = sample as i16;
            stereo[i * 2 + 1] = sample as i16;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Player;
    use crate::seam::Status;

    #[test]
    fn render_stops_at_the_buffer() {
        let mut status = Status::new();
        let mut player = Player::init(&mut status);
        status.playing = true;
        let mut stereo = [7i16; 4];
        player.render(&status, &mut stereo, 64, super::TONE_RATE);
        assert!(stereo.iter().any(|sample| *sample != 7));
        player.render(&status, &mut stereo, 64, super::TONE_RATE);
        assert_eq!(stereo.len(), 4);
    }
}
