# Glossary

Status: baseline · 2026-10-28

| Term | Meaning |
| --- | --- |
| Porta | This product and this repository ([D-0001](decisions.md#d-0001-product-name)). |
| Listener | The person using the device. There is no separate account model yet. |
| Core2 v1.3 | The M5Stack host: ESP32-D0WDQ6-V3, 320×240 touchscreen, three virtual screen buttons, AXP192, internal I2S speaker and microphone. |
| Module Audio | M5Stack audio module, SKU M144. ES8388 codec, one TRS microphone jack, one TRRS headphone jack. |
| Module HMI | M5Stack HMI module. Rotary encoder, encoder press, two side buttons, two LEDs. I2C address `0x41`. |
| M5GO Bottom2 v1.3 | Core2 base used when modules are stacked. SKU A014-C-V13. Own 500 mAh cell and charger. |
| M5Stamp Pico | Separate ESP32-PICO-D4 board used only as a Bluetooth audio transmitter. No PSRAM. |
| Module Proto | Prototyping module that carries the wiring for the Stamp Pico. |
| M5-Bus | The 30-pin stack bus between the Core2 and its modules. |
| I2S | The digital audio link between the Core2, Module Audio, and the Stamp Pico. Format is [D-0032](decisions.md#d-0032-i2s-format). |
| AirPlay 1 | RAOP. The sender is a phone or a computer. The audio codec is ALAC ([D-0036](decisions.md#d-0036-airplay-and-dlna)). |
| DLNA renderer | A player that fetches a URI a controller sets. Codecs are LPCM, WAV PCM, and ALAC ([D-0036](decisions.md#d-0036-airplay-and-dlna)). |
| Spotify Connect | The session in which the Core2 fetches and plays a track while a phone only chooses the music ([D-0029](decisions.md#d-0029-spotify-transport)). |
| cspot | The C++ library that implements that session on the Core2. Use the philippe44 fork, not the last upstream snapshot ([P-0026](pitfalls.md#p-0026-cspot-fork)). |
| Postern | The ESP-IDF component that sends I2S audio to Bluetooth. It is the `postern` submodule ([D-0039](decisions.md#d-0039-firmware-repos)). |
| A2DP | Bluetooth profile used to send audio to headphones. |
| AAC / SBC | Codecs for that Bluetooth link. AAC is the goal. SBC is the fallback that must work first ([D-0007](decisions.md#d-0007-bluetooth-transmitter)). |
| Auxiliary mode | Phase 2 product mode that exposes the audio-bridge jobs inside the same UI. |
| Devshell | The environment from `nix develop`. Agent commands and the default terminal enter it. |
| Decision | A `D-xxxx` entry in [decisions](decisions.md). Ids are not reused. |
| Pitfall | A `P-xxxx` entry in [pitfalls](pitfalls.md). Open entries are binding. |
| Phase 0 | This specification and the development skeleton. Current phase. |
| Phase 1 | Core2 Wi-Fi-to-I2S player, Stamp Pico I2S-to-Bluetooth bridge, simple but complete UI. |
| Phase 2 | Elaborated UI, auxiliary bridge modes, AirPlay, DLNA, easier login, battery behaviour. |
