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

use crate::seam::{Command, CommandKind, Queue, Status, QUEUE_CAP, VOLUME_MAX};

pub const TONE_HZ: u32 = 440;
pub const TONE_RATE: u32 = 44100;
pub const TONE_TITLE: &str = "Tone stand-in";
pub const WAITING_TITLE: &str = "No track";
pub const CRED_CAP: usize = 512;
const PCM_SAMPLES: usize = 1024;

#[derive(Debug)]
pub struct Player {
    phase: u32,
    creds: [u8; CRED_CAP],
    cred_len: u16,
    pcm: [i16; PCM_SAMPLES],
    pcm_r: u16,
    pcm_len: u16,
    pending: [Command; QUEUE_CAP],
    pending_head: u8,
    pending_count: u8,
}

impl Player {
    pub fn init(status: &mut Status) -> Self {
        status.title.set(TONE_TITLE);
        status.playing = false;
        status.volume = 50;
        status.session_ready = false;
        Self {
            phase: 0,
            creds: [0; CRED_CAP],
            cred_len: 0,
            pcm: [0; PCM_SAMPLES],
            pcm_r: 0,
            pcm_len: 0,
            pending: [Command {
                kind: CommandKind::Next,
                volume_step: 0,
            }; QUEUE_CAP],
            pending_head: 0,
            pending_count: 0,
        }
    }

    /// A stored blob plus Wi-Fi is a session. Losing either returns to the tone.
    pub fn sync(&mut self, status: &mut Status) {
        let ready = status.wifi_associated && self.cred_len > 0;
        if ready && !status.session_ready {
            status.session_ready = true;
            status.title.set(WAITING_TITLE);
            self.discard_pcm();
        } else if !ready && status.session_ready {
            status.session_ready = false;
            status.playing = false;
            status.title.set(TONE_TITLE);
            self.discard_pcm();
            self.pending_head = 0;
            self.pending_count = 0;
        }
    }

    /// Keeps the previous blob when `blob` is empty or longer than [`CRED_CAP`].
    pub fn store_credentials(&mut self, status: &mut Status, blob: &[u8]) -> bool {
        if blob.is_empty() || blob.len() > CRED_CAP {
            return false;
        }
        self.creds[..blob.len()].copy_from_slice(blob);
        self.cred_len = blob.len() as u16;
        self.sync(status);
        true
    }

    pub fn has_credentials(&self) -> bool {
        self.cred_len > 0
    }

    /// Sets the title only while a session is up. An empty title is refused.
    pub fn note_track(&mut self, status: &mut Status, title: &str) -> bool {
        self.sync(status);
        if !status.session_ready || title.is_empty() {
            return false;
        }
        status.title.set(title);
        true
    }

    pub fn free_frames(&self) -> usize {
        (PCM_SAMPLES - self.pcm_len as usize) / 2
    }

    pub fn discard_pcm(&mut self) {
        self.pcm_len = 0;
        self.pcm_r = 0;
    }

    /// The session's play or pause. Pause drops samples already queued here.
    pub fn apply_remote_pause(&mut self, status: &mut Status, paused: bool) {
        self.sync(status);
        if !status.session_ready {
            return;
        }
        status.playing = !paused;
        if paused {
            self.discard_pcm();
        }
    }

    /// Interleaved stereo. Returns frames stored. A full buffer keeps what it has.
    pub fn push_pcm(&mut self, status: &Status, stereo: &[i16]) -> usize {
        if !status.session_ready {
            return 0;
        }
        let samples = stereo.len() & !1;
        let room = PCM_SAMPLES - self.pcm_len as usize;
        let n = samples.min(room);
        if n == 0 {
            return 0;
        }
        let at = (self.pcm_r as usize + self.pcm_len as usize) % PCM_SAMPLES;
        let first = n.min(PCM_SAMPLES - at);
        self.pcm[at..at + first].copy_from_slice(&stereo[..first]);
        if n > first {
            self.pcm[..n - first].copy_from_slice(&stereo[first..n]);
        }
        self.pcm_len += n as u16;
        n / 2
    }

    pub fn take_session_command(&mut self) -> Option<Command> {
        if self.pending_count == 0 {
            return None;
        }
        let command = self.pending[self.pending_head as usize];
        self.pending_head = (self.pending_head + 1) % QUEUE_CAP as u8;
        self.pending_count -= 1;
        Some(command)
    }

    pub fn drain(&mut self, queue: &mut Queue, status: &mut Status) {
        self.sync(status);
        while let Some(command) = queue.pop() {
            match command.kind {
                CommandKind::Play => status.playing = true,
                CommandKind::Pause => {
                    status.playing = false;
                    self.discard_pcm();
                }
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
            if status.session_ready {
                self.enqueue_session(command);
            }
        }
    }

    fn enqueue_session(&mut self, command: Command) {
        if self.pending_count >= QUEUE_CAP as u8 {
            return;
        }
        let tail = (self.pending_head + self.pending_count) % QUEUE_CAP as u8;
        self.pending[tail as usize] = command;
        self.pending_count += 1;
    }

    /// Interleaved stereo s16. `frames` counts samples the jack should queue.
    /// `keep` is false when playback is stopped, so a queue can drop what it holds.
    /// Without a session the samples are a triangle, not a track.
    pub fn render(
        &mut self,
        status: &mut Status,
        stereo: &mut [i16],
        frames: usize,
        rate_hz: u32,
    ) -> Played {
        self.sync(status);
        let frames = frames.min(stereo.len() / 2);
        if frames == 0 {
            return Played {
                frames: 0,
                keep: true,
            };
        }
        if status.session_ready {
            return self.render_session(status, stereo, frames);
        }
        if !(status.playing && status.volume > 0) || rate_hz == 0 {
            silence(stereo, frames);
            return Played {
                frames: 0,
                keep: false,
            };
        }
        let step = (((TONE_HZ as u64) << 32) / rate_hz as u64) as u32;
        for i in 0..frames {
            let saw = (self.phase >> 16) as u16;
            let mut tri = if saw < 32768 {
                saw as i32
            } else {
                (65535 - saw) as i32
            };
            tri -= 16384;
            let sample = tri * status.volume as i32 / 256;
            self.phase = self.phase.wrapping_add(step);
            stereo[i * 2] = sample as i16;
            stereo[i * 2 + 1] = sample as i16;
        }
        Played {
            frames,
            keep: true,
        }
    }

    fn render_session(&mut self, status: &Status, stereo: &mut [i16], frames: usize) -> Played {
        if !(status.playing && status.volume > 0) {
            silence(stereo, frames);
            return Played {
                frames: 0,
                keep: false,
            };
        }
        let mut heard = 0;
        for i in 0..frames {
            let Some((left, right)) = self.pull_frame() else {
                silence(&mut stereo[i * 2..], frames - i);
                break;
            };
            stereo[i * 2] = scale(left, status.volume);
            stereo[i * 2 + 1] = scale(right, status.volume);
            heard += 1;
        }
        Played {
            frames: heard,
            keep: true,
        }
    }

    fn pull_frame(&mut self) -> Option<(i16, i16)> {
        if self.pcm_len < 2 {
            return None;
        }
        let at = self.pcm_r as usize;
        let left = self.pcm[at];
        let right = self.pcm[(at + 1) % PCM_SAMPLES];
        self.pcm_r = ((at + 2) % PCM_SAMPLES) as u16;
        self.pcm_len -= 2;
        Some((left, right))
    }
}

/// What [`Player::render`] wrote for the jack.
pub struct Played {
    /// Frames of sound at the start of the buffer. The rest is silence.
    pub frames: usize,
    /// The jack keeps samples it already has. False drops them.
    pub keep: bool,
}

fn silence(stereo: &mut [i16], frames: usize) {
    stereo[..frames * 2].fill(0);
}

fn scale(sample: i16, volume: u8) -> i16 {
    (sample as i32 * volume as i32 / VOLUME_MAX as i32) as i16
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
        player.render(&mut status, &mut stereo, 64, super::TONE_RATE);
        assert!(stereo.iter().any(|sample| *sample != 7));
        player.render(&mut status, &mut stereo, 64, super::TONE_RATE);
        assert_eq!(stereo.len(), 4);
    }

    #[test]
    fn queued_samples_survive_a_wrap() {
        let mut status = Status::new();
        let mut player = Player::init(&mut status);
        assert!(player.store_credentials(&mut status, b"blob"));
        status.wifi_associated = true;
        player.sync(&mut status);
        status.playing = true;
        status.volume = 100;
        let mut frame = [0i16; 2];
        for i in 0..400 {
            frame[0] = i as i16;
            frame[1] = 1;
            assert_eq!(player.push_pcm(&status, &frame), 1);
        }
        let mut out = [0i16; 600];
        let played = player.render(&mut status, &mut out, 100, super::TONE_RATE);
        assert_eq!(played.frames, 100);
        assert_eq!(out[0], 0);
        assert_eq!(out[198], 99);
        for i in 0..200 {
            frame[0] = 1000 + i as i16;
            frame[1] = 1;
            assert_eq!(player.push_pcm(&status, &frame), 1);
        }
        let played = player.render(&mut status, &mut out, 300, super::TONE_RATE);
        assert_eq!(played.frames, 300);
        assert_eq!(out[0], 100);
        assert_eq!(out[598], 399);
        let played = player.render(&mut status, &mut out, 112, super::TONE_RATE);
        assert_eq!(played.frames, 112);
        assert_eq!(out[0], 1000);
        assert_eq!(out[222], 1111);
        let played = player.render(&mut status, &mut out, 1, super::TONE_RATE);
        assert_eq!(played.frames, 1);
        assert!(played.keep);
        assert_eq!(out[0], 1112);
    }
}
