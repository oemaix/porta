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

//! Commands, the status snapshot, and the eight-deep queue. No driver.

pub const TITLE_LEN: usize = 48;
pub const LABEL_LEN: usize = 24;
pub const QUEUE_CAP: usize = 8;
pub const VOLUME_MAX: u8 = 100;
pub const VOLUME_STEP: i8 = 10;

/// Listener-facing actions are next, previous, activate, and back.
/// Focus is how a tap names a block. Button 3 and the screen buttons are
/// reported and dropped by ui.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionKind {
    Next = 1,
    Previous,
    Activate,
    Back,
    Focus,
    Button3,
    Screen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Action {
    pub kind: ActionKind,
    pub index: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandKind {
    Play = 1,
    Pause,
    Next,
    Previous,
    SetVolume,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command {
    pub kind: CommandKind,
    pub volume_step: i8,
}

#[derive(Clone, Copy, Debug)]
pub struct FixedText<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> FixedText<N> {
    pub const fn new() -> Self {
        Self { bytes: [0; N] }
    }

    pub fn set(&mut self, src: &str) {
        self.bytes = [0; N];
        let max = N.saturating_sub(1);
        let mut end = src.len().min(max);
        while end > 0 && !src.is_char_boundary(end) {
            end -= 1;
        }
        self.bytes[..end].copy_from_slice(&src.as_bytes()[..end]);
    }

    pub fn as_str(&self) -> &str {
        let end = self.bytes.iter().position(|b| *b == 0).unwrap_or(N);
        match core::str::from_utf8(&self.bytes[..end]) {
            Ok(text) => text,
            Err(err) => core::str::from_utf8(&self.bytes[..err.valid_up_to()]).unwrap_or(""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FixedText;

    #[test]
    fn set_keeps_a_whole_character() {
        let mut text = FixedText::<4>::new();
        text.set("éé");
        assert_eq!(text.as_str(), "é");
        let mut tight = FixedText::<2>::new();
        tight.set("é");
        assert_eq!(tight.as_str(), "");
    }
}

impl<const N: usize> Default for FixedText<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Status {
    pub title: FixedText<TITLE_LEN>,
    pub playing: bool,
    pub volume: u8,
    pub session_ready: bool,
    pub wifi_associated: bool,
}

impl Status {
    pub fn new() -> Self {
        Self {
            title: FixedText::new(),
            playing: false,
            volume: 0,
            session_ready: false,
            wifi_associated: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
}

#[derive(Clone, Copy, Debug)]
pub struct Queue {
    items: [Command; QUEUE_CAP],
    head: u8,
    pub count: u8,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            items: [Command {
                kind: CommandKind::Next,
                volume_step: 0,
            }; QUEUE_CAP],
            head: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, command: Command) -> bool {
        if self.count >= QUEUE_CAP as u8 {
            return false;
        }
        let tail = (self.head + self.count) % QUEUE_CAP as u8;
        self.items[tail as usize] = command;
        self.count += 1;
        true
    }

    pub fn pop(&mut self) -> Option<Command> {
        if self.count == 0 {
            return None;
        }
        let command = self.items[self.head as usize];
        self.head = (self.head + 1) % QUEUE_CAP as u8;
        self.count -= 1;
        Some(command)
    }
}
