# Roadmap

Status: baseline · 2026-10-28

## Current status

**Phase 0 is the current phase for the Core2.** The specification and the Nix skeleton are in the working tree. The Core2 application is not written. Postern's phase-1 path is in the submodule and has not been run on a board.

| Phase | State | Exit |
| --- | --- | --- |
| 0. Specification and skeleton | Current | Docs stand alone, `nix develop` opens a shell, editor and agent entry points exist |
| 1. Separate prototypes and a usable UI | Not started | UC-01 through UC-04 pass on hardware |
| 2. One product | Not started | UC-05 through UC-08 pass, and the phase 1 actions are unchanged |

## Phase 1

From the design notes, kept as two prototypes ([D-0023](decisions.md#d-0023-separate-phase-1-prototypes)):

- M5Stamp Pico audio bridge: the Postern component, I2S in, Bluetooth out, AAC with an SBC fallback ([D-0039](decisions.md#d-0039-firmware-repos)). The phase-1 path in that component is I2S slave, the I2C register driver, and A2DP SBC. AAC is not in that code. It has not been run on a board.
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

## Blockers before phase 1 code

Close, or explicitly accept a prototype stand-in for:

- [D-0028](decisions.md#d-0028-firmware-framework) is closed: ESP-IDF. `idf.py` is not in the shell yet.
- [D-0029](decisions.md#d-0029-spotify-transport) is closed: cspot on the Core2. Catalog search is still later, via the Web API.
- [D-0031](decisions.md#d-0031-board-link) and [D-0032](decisions.md#d-0032-i2s-format) are closed. Phase 1 may still bring the boards up separately.
- [D-0033](decisions.md#d-0033-power-architecture) before the stack is powered as a product. A bench supply is a valid phase 1 stand-in if the decision says so.
