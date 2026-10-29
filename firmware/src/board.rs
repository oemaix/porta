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

//! Internal I2C is GPIO21 and GPIO22. The AXP192 is at 0x34.
//! Module Audio configuration A shares the internal codec pins.
//! `limit_measured` records a written-down current limit. It does not
//! forbid I2S or Wi-Fi.

pub const I2C_SDA_GPIO: i32 = 21;
pub const I2C_SCL_GPIO: i32 = 22;
pub const AXP192_ADDR: u8 = 0x34;

#[derive(Clone, Copy, Debug)]
pub struct Board {
    pub bench_supply: bool,
    limit_measured: bool,
}

impl Board {
    pub fn init() -> Self {
        Self {
            bench_supply: false,
            limit_measured: false,
        }
    }

    pub fn use_bench(&mut self) {
        self.bench_supply = true;
    }

    /// False until the AXP192 current limit is written down.
    /// Callers may still start I2S and Wi-Fi. See P-0040.
    pub fn limit_measured(&self) -> bool {
        self.limit_measured
    }

    pub fn set_limit_measured(&mut self, measured: bool) {
        self.limit_measured = measured;
    }
}

#[cfg(test)]
mod tests {
    use super::Board;

    #[test]
    fn limit_starts_clear_and_can_be_recorded() {
        let mut board = Board::init();
        assert!(!board.limit_measured());
        board.set_limit_measured(true);
        assert!(board.limit_measured());
    }
}
