# Core2 application

Rust crate on ESP-IDF v5.5.6 through `esp-idf-svc` ([D-0046](../docs/decisions.md#d-0046-core2-language)). The host check, from the Porta devshell:

```sh
cargo test --manifest-path firmware/Cargo.toml
```

The image target is `xtensa-esp32-espidf`. `firmware/.cargo/config.toml` pins ESP-IDF v5.5.6. From this directory, so Cargo reads that file:

```sh
cargo build --target xtensa-esp32-espidf
```

`embuild` clones that ESP-IDF tree into `.embuild/` on the build ([P-0049](../docs/pitfalls.md#p-0049-esp-rs-rustc)). A passing `cargo test` is not a flashed image ([P-0048](../docs/pitfalls.md#p-0048-core2-image-target)). The tree powers the panel and paints Now Playing. It has not started Wi-Fi or the codec yet ([D-0043](../docs/decisions.md#d-0043-firmware-tree)).

Phase 1 behavior that the host test covers:

- `input` turns the wheel, the three HMI buttons, touch, and the screen-button strip into actions.
- `ui` keeps the screen stack and the focus index. Now Playing, Connect, and Settings are the phase 1 screens.
- `player` accepts play, pause, next, previous, and set volume. The sound is a 440 Hz triangle titled "Tone stand-in". It is not Spotify.
- `sink` accepts 16-bit stereo at 44100 or 48000 Hz in software. Hardware I2S stays off until the AXP192 current limit is measured.

`firmware_main` calls `App::init`. This tree has not started the codec or Wi-Fi yet. Component scopes are in [software](../docs/software.md).
