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

pub const RATE_44K: u32 = 44100;
pub const RATE_48K: u32 = 48000;

#[derive(Clone, Copy, Debug)]
pub struct Sink {
    open: bool,
    pub rate_hz: u32,
    pub frames: usize,
}

fn legal_rate(rate_hz: u32) -> bool {
    rate_hz == RATE_44K || rate_hz == RATE_48K
}

impl Sink {
    pub fn init() -> Self {
        Self {
            open: false,
            rate_hz: 0,
            frames: 0,
        }
    }

    /// Accepts 16-bit stereo at 44100 or 48000. A second open rate is rejected.
    pub fn open(&mut self, rate_hz: u32) -> bool {
        if !legal_rate(rate_hz) {
            return false;
        }
        if self.open && self.rate_hz != rate_hz {
            return false;
        }
        self.open = true;
        self.rate_hz = rate_hz;
        true
    }

    /// Writes frames only when `rate_hz` is the opened rate. Another rate is
    /// rejected and not counted. Convert with [`resample_s16`] first.
    pub fn write(&mut self, stereo: Option<&[i16]>, frames: usize, rate_hz: u32) -> bool {
        if !self.open || stereo.is_none() || rate_hz != self.rate_hz {
            return false;
        }
        self.frames += frames;
        true
    }
}

/// Linear, 16-bit stereo, and only the two cable rates. Returns frames written.
pub fn resample_s16(
    input: &[i16],
    in_frames: usize,
    in_rate: u32,
    out: &mut [i16],
    out_cap: usize,
    out_rate: u32,
) -> usize {
    let in_frames = in_frames.min(input.len() / 2);
    let out_cap = out_cap.min(out.len() / 2);
    if !legal_rate(in_rate) || !legal_rate(out_rate) || in_frames == 0 {
        return 0;
    }
    if in_rate == out_rate {
        if out_cap < in_frames {
            return 0;
        }
        let n = in_frames * 2;
        out[..n].copy_from_slice(&input[..n]);
        return in_frames;
    }
    let out_frames = ((in_frames as u64 * out_rate as u64) / in_rate as u64) as usize;
    if out_frames == 0 || out_frames > out_cap {
        return 0;
    }
    for i in 0..out_frames {
        let pos = i as u64 * in_rate as u64;
        let src = (pos / out_rate as u64) as usize;
        let frac = (pos % out_rate as u64) as u32;
        let next = if src + 1 >= in_frames { src } else { src + 1 };
        for ch in 0..2 {
            let a = input[src * 2 + ch] as i32;
            let b = input[next * 2 + ch] as i32;
            let mixed = a + ((b - a) as i64 * frac as i64 / out_rate as i64) as i32;
            out[i * 2 + ch] = mixed as i16;
        }
    }
    out_frames
}

#[cfg(test)]
mod tests {
    use super::{resample_s16, RATE_44K, RATE_48K};

    #[test]
    fn short_slices_are_not_read_past_the_end() {
        let input = [1000i16, 0, 0, 0];
        let mut output = [0i16; 32];
        let n = resample_s16(&input, 441, RATE_44K, &mut output, 480, RATE_48K);
        assert!(n <= 2);
        let wide = [1000i16; 16];
        let mut tiny = [0i16; 4];
        assert_eq!(resample_s16(&wide, 8, RATE_44K, &mut tiny, 8, RATE_44K), 0);
    }
}
