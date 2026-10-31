# Roadmap

Status: baseline · 2026-10-31

## Current status

**Phase 1 is the current phase.** The basic UI is in use on the bench: the wheel moves focus, the wheel press activates, BtnA goes back, and touch uses that same model. Playback is still the labelled tone stand-in. Wi-Fi, Spotify Connect, and the headphone jack are not started. The Postern image is already written in the submodule. Wiring it into this product is phase 2.

| Phase | State | Exit |
| --- | --- | --- |
| 0. Specification and skeleton | Done | Docs stand alone, `nix develop` opens a shell, editor and agent entry points exist |
| 1. Spotify on the jack | Current | A Connect session plays on the Module Audio headphone jack, and the basic UI reaches that playback without a host |
| 2. Postern in the product | Not started | Bluetooth headphones can be connected and removed, the TRS jack is a source, the screen shows a simple power level, the panel can sleep without stopping audio, and the Module Audio LEDs show idle, Spotify, or the TRS source |
| 3. Other sources and the finished product | Not started | AirPlay 1 and DLNA play, the layout is elaborated, and advanced power management includes auto power-off |

## Phase 1

A workable Spotify Connect player on the Core2, with Wi-Fi, playing to the 3.5 mm headphone jack ([D-0029](decisions.md#d-0029-spotify-transport)). The UI already in use stays the surface: Now Playing, Connect, and Settings, on the same four actions.

The Postern image stays in its submodule. This phase does not connect it, does not start its Bluetooth link, and does not feed it audio ([D-0023](decisions.md#d-0023-separate-phase-1-prototypes)).

Not in this phase: Postern integration, line-in and the microphone input, a power-level indication, panel sleep, the Module Audio status LEDs, AirPlay, DLNA, the easier login, auto power-off, and the elaborated layout.

## Phase 2

Postern joins the product. The listener can connect a Bluetooth headphone and remove it. Audio from the Core2 can be sent across the cable in [D-0031](decisions.md#d-0031-board-link). The Stamp Pico remains the A2DP source ([D-0008](decisions.md#d-0008-radio-split)). The sink in this phase is the headphone. The Core2 does not become an A2DP receiver.

The TRS jack becomes an audio source: line-in and the microphone input ([P-0008](pitfalls.md#p-0008-trs-jack)). That path is auxiliary mode. The headphone output stays the TRRS jack.

A simple power level appears on the screen. Which cell is measured stays [D-0033](decisions.md#d-0033-power-architecture). This phase does not power the device off from that level.

After a period with no operation the LCD turns off. The audio stream keeps playing. The Module HMI, the touchscreen, or a touch button wakes the panel.

The three RGB LEDs on the side of Module Audio show which source is playing, including while that panel is dark ([D-0047](decisions.md#d-0047-module-audio-leds)). Idle and pause leave them off. Spotify Connect lights them green. The TRS source lights them red. They stay at the module's default brightness. AirPlay and DLNA use blue. That color stays unused until those sources exist in phase 3.

## Phase 3

- DLNA renderer and AirPlay 1, on the jack and on Bluetooth ([D-0036](decisions.md#d-0036-airplay-and-dlna)). While either source is playing, the Module Audio LEDs use the blue pattern from [D-0047](decisions.md#d-0047-module-audio-leds).
- The elaborated layout, on the same actions ([FR-12](requirements.md)).
- Advanced power management, including auto power-off at a low threshold ([UC-08](use-cases.md#uc-08-stop-before-the-battery-dies)).
- An easier Spotify login than phase 1, after [D-0030](decisions.md#d-0030-login).

## Still open for a hardware pass

- [D-0046](decisions.md#d-0046-core2-language) is closed: the Core2 crate is Rust on ESP-IDF v5.5.6. The Porta shell carries the esp-rs rustc 1.95.0.0 ([P-0049](pitfalls.md#p-0049-esp-rs-rustc)). ESP-IDF v5.5.6 is cloned by `embuild` when the image is built. A passing host test is not a flashed image ([P-0048](pitfalls.md#p-0048-core2-image-target)). Postern stays on ESP-IDF v6.1.
- [D-0029](decisions.md#d-0029-spotify-transport) is closed: cspot on the Core2. The tree uses the labelled tone stand-in until that library is linked ([P-0001](pitfalls.md#p-0001-spotify-on-the-esp32)). Catalog search is still later.
- [D-0031](decisions.md#d-0031-board-link) and [D-0032](decisions.md#d-0032-i2s-format) are closed. Phase 1 does not open the cable. Phase 2 is the first product use of that link.
- [D-0033](decisions.md#d-0033-power-architecture) stays open for the product. The prototypes accept a bench supply ([D-0043](decisions.md#d-0043-firmware-tree)). That supply is not a measured AXP192 limit. Wi-Fi and I2S may start before the measurement. Instability then is suspected there first ([P-0040](pitfalls.md#p-0040-unmeasured-rail)).
