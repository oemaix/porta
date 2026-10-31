# Glossary

Status: baseline · 2026-10-31

| Term | Meaning |
| --- | --- |
| Porta | This product and this repository ([D-0001](decisions.md#d-0001-product-name)). |
| Listener | The person using the device. There is no separate account model yet. |
| Core2 v1.3 | The M5Stack host: ESP32-D0WDQ6-V3, 320×240 touchscreen, three virtual screen buttons, AXP192, internal I2S speaker and microphone. |
| Module Audio | M5Stack audio module, SKU M144. ES8388 codec, one TRS microphone jack, one TRRS headphone jack, three WS2812C LEDs that show the playing source ([D-0047](decisions.md#d-0047-module-audio-leds)). |
| Module HMI | M5Stack HMI module. Rotary encoder, encoder switch S, side keys BtnA and BtnB, two LEDs. I2C address `0x41`. BtnA is Back. |
| M5GO Bottom2 v1.3 | Core2 base used when modules are stacked. SKU A014-C-V13. Own 500 mAh cell and charger. |
| M5Stamp Pico | Separate ESP32-PICO-D4 board used only as a Bluetooth audio transmitter. No PSRAM. |
| Module Proto | Prototyping module that carries the wiring for the Stamp Pico. |
| M5-Bus | The 30-pin stack bus between the Core2 and its modules. |
| I2S | The digital audio link between the Core2, Module Audio, and the Stamp Pico. Format is [D-0032](decisions.md#d-0032-i2s-format). Wiring is the [Postern cable note](https://github.com/oemaix/postern/blob/main/docs/cable.md). |
| AirPlay 1 | RAOP. The sender is a phone or a computer. The audio codec is ALAC ([D-0036](decisions.md#d-0036-airplay-and-dlna)). |
| DLNA renderer | A player that fetches a URI a controller sets. Codecs are LPCM, WAV PCM, and ALAC ([D-0036](decisions.md#d-0036-airplay-and-dlna)). |
| Spotify Connect | The session in which the Core2 fetches and plays a track while a phone only chooses the music ([D-0029](decisions.md#d-0029-spotify-transport)). |
| cspot | The C++ library that implements that session on the Core2. Use the philippe44 fork, not the last upstream snapshot ([P-0026](pitfalls.md#p-0026-cspot-fork)). |
| Player command | play, pause, next, previous, or set volume. Sent by `ui` to `player` ([D-0042](decisions.md#d-0042-core2-components)). |
| Status snapshot | Fixed-size record `ui` reads: title, transport, volume, session ready, and Wi-Fi. PCM is not in it. |
| Postern | The Stamp Pico firmware. It is the `postern` submodule, developed on its own. The reusable bridge is `postern/components/postern` ([D-0045](decisions.md#d-0045-postern-image)). |
| A2DP | Bluetooth profile used to send audio to headphones. |
| AAC / SBC | Codecs for that Bluetooth link. AAC is the goal. SBC is the fallback that must work first ([D-0007](decisions.md#d-0007-bluetooth-transmitter)). |
| Auxiliary mode | Phase 2 product mode that plays the TRS jack through the product, including to Bluetooth headphones. |
| Devshell | The environment from `nix develop`. Agent commands and the default terminal enter it. |
| Decision | A `D-xxxx` entry in [decisions](decisions.md). Ids are not reused. |
| Pitfall | A `P-xxxx` entry in [pitfalls](pitfalls.md). Open entries are binding. |
| Phase 0 | This specification and the development skeleton. Done. |
| Phase 1 | Spotify Connect on the headphone jack, with Wi-Fi, and the basic UI. Current phase. |
| Phase 2 | Postern in the product, TRS line-in and mic, a simple power level, panel sleep that leaves audio running, and Module Audio LEDs for the playing source. |
| Phase 3 | AirPlay 1, DLNA, the elaborated UI, easier login, and auto power-off. |
