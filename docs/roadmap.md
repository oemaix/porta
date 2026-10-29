# Roadmap

Status: baseline · 2026-10-29

## Current status

**Phase 1 is the current phase for the Core2.** The application tree is `firmware/` ([D-0043](decisions.md#d-0043-firmware-tree)). It has not exited: UC-01 through UC-04 have not passed on hardware. Playback is the labelled tone stand-in. Hardware I2S and Wi-Fi are not started. Postern's phase-1 path is in the submodule and has not been run on a board.

| Phase | State | Exit |
| --- | --- | --- |
| 0. Specification and skeleton | Done | Docs stand alone, `nix develop` opens a shell, editor and agent entry points exist |
| 1. Separate prototypes and a usable UI | Current | UC-01 through UC-04 pass on hardware |
| 2. One product | Not started | UC-05 through UC-08 pass, and the phase 1 actions are unchanged |

## Phase 1

From the design notes, kept as two prototypes ([D-0023](decisions.md#d-0023-separate-phase-1-prototypes)):

- M5Stamp Pico audio bridge: the Postern image, I2S in, Bluetooth out, AAC with an SBC fallback ([D-0045](decisions.md#d-0045-postern-image)). The component encodes AAC-LC and falls back to SBC. It has not been run on a board.
- Spotify Connect player on Core2 v1.3: the Core2 fetches the track and plays it to I2S ([D-0029](decisions.md#d-0029-spotify-transport)).
- A simple UI that is complete enough to use, built on an API that can carry later screens.
- Module HMI working for focus, confirm, and back.

Not in this phase: AirPlay, DLNA, auxiliary mode, the easier login, battery indication, and the unbound second HMI button.

## Phase 2

- Elaborated UI on the same actions.
- Auxiliary mode for the bridge jobs, including line-in to Bluetooth.
- AirPlay 1 (ALAC) and a DLNA renderer for LPCM, WAV PCM, and ALAC, on the jack and on Bluetooth ([D-0036](decisions.md#d-0036-airplay-and-dlna)).
- An easier login than phase 1, after [D-0030](decisions.md#d-0030-login).
- Battery level and a low-battery power-off, after [D-0033](decisions.md#d-0033-power-architecture).

## Still open for a hardware pass

- [D-0028](decisions.md#d-0028-firmware-framework) is closed: ESP-IDF v6.1, with `idf.py` in the Porta shell. The chip tools install under `~/.espressif` on first use.
- [D-0029](decisions.md#d-0029-spotify-transport) is closed: cspot on the Core2. The tree uses the labelled tone stand-in until that library is linked ([P-0039](pitfalls.md#p-0039-tone-stand-in)). Catalog search is still later.
- [D-0031](decisions.md#d-0031-board-link) and [D-0032](decisions.md#d-0032-i2s-format) are closed. Phase 1 still brings the boards up separately.
- [D-0033](decisions.md#d-0033-power-architecture) stays open for the product. The prototypes accept a bench supply ([D-0043](decisions.md#d-0043-firmware-tree)). That is not a measured AXP192 limit, so I2S and Wi-Fi stay off ([P-0040](pitfalls.md#p-0040-unmeasured-rail)).
