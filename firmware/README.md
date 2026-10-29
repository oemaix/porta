# Core2 application

ESP-IDF v6.1 image for the Core2. Build it from the Porta devshell ([D-0028](../docs/decisions.md#d-0028-firmware-framework)):

```sh
idf.py set-target esp32
idf.py build
```

The first run installs the toolchain under `~/.espressif`. The image does not start Wi-Fi or the codec ([D-0043](../docs/decisions.md#d-0043-firmware-tree)).

Phase 1 behavior that runs on the host:

- `input` turns the wheel, the three HMI buttons, touch, and the screen-button strip into actions.
- `ui` keeps the screen stack and the focus index. Now Playing, Connect, and Settings are the phase 1 screens.
- `player` accepts play, pause, next, previous, and set volume. The sound is a 440 Hz triangle titled "Tone stand-in". It is not Spotify.
- `sink` accepts 16-bit stereo at 44100 or 48000 Hz in software. Hardware I2S stays off until the AXP192 current limit is measured.

`main` calls `porta_app_init` and does not start the codec or Wi-Fi. The host check is in [operations](../docs/operations.md#core2-host-test). Component scopes are in [software](../docs/software.md).
