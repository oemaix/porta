# Requirements

Status: baseline · 2026-10-28

Phase 1 requirements are the acceptance bar for that phase. Phase 2 requirements are not implemented early ([P-0012](pitfalls.md#p-0012-phase-creep)).

## Functional

| Id | Phase | Requirement | Use case |
| --- | --- | --- | --- |
| FR-01 | 1 | The Core2 plays a Wi-Fi audio stream through the Module Audio headphone output. | UC-01 |
| FR-02 | 1 | The player UI can start, pause, and resume that stream. | UC-01 |
| FR-03 | 1 | The Stamp Pico transmits I2S audio to Bluetooth headphones using SBC, and uses AAC when the session supports it. | UC-02 |
| FR-04 | 1 | A UI API models a screen stack, a focus index, and the actions next, previous, activate, and back. Input devices call that API. They do not draw their own navigation. | UC-03, UC-04 |
| FR-05 | 1 | The first UI is small and is complete: a listener can reach playback and leave every screen without a host attached. | UC-01, UC-03, UC-04 |
| FR-06 | 1 | Module HMI wheel motion moves focus, the wheel press activates, and button index 2 goes back. | UC-03, UC-04 |
| FR-07 | 2 | Auxiliary mode plays the TRS input on Bluetooth headphones. | UC-05 |
| FR-08 | 2 | Porta appears as an AirPlay 1 receiver, decodes ALAC, and plays it on the jack and on Bluetooth. | UC-06 |
| FR-09 | 2 | Porta appears as a DLNA renderer for 16-bit stereo LPCM, WAV PCM, and ALAC at 44.1 or 48 kHz, and plays that audio on the jack and on Bluetooth. | UC-07 |
| FR-10 | 2 | Login to the Spotify path takes fewer steps than the phase 1 method. | UC-01 |
| FR-11 | 2 | The UI shows battery level and powers off at a defined low threshold. | UC-08 |
| FR-12 | 2 | The screen layout is elaborated without changing the phase 1 actions. | UC-03 |

FR-01 is the Connect session on the Core2 ([D-0029](decisions.md#d-0029-spotify-transport)). The bytes on Wi-Fi are the track cspot fetches. FR-10 waits on [D-0030](decisions.md#d-0030-login).

## Non-functional

| Id | Requirement |
| --- | --- |
| NFR-01 | Wheel, touch, and the three screen buttons emit the actions in FR-04. There is one focus model ([D-0010](decisions.md#d-0010-interaction-model)). |
| NFR-02 | Wi-Fi association stays on the Core2. A2DP source stays on the Stamp Pico ([D-0008](decisions.md#d-0008-radio-split)). |
| NFR-03 | The UI is flat color blocks on a 320×240 panel, readable as a list of large targets ([D-0011](decisions.md#d-0011-visual-language)). |
| NFR-04 | Phase 1 playback must not brown out the Core2 when Wi-Fi and I2S start together ([P-0018](pitfalls.md#p-0018-brownout)). |
| NFR-05 | Cells are not paralleled. Charge and fuel measurement follow [D-0033](decisions.md#d-0033-power-architecture). |
| NFR-06 | A clean Linux host with Nix can enter the devshell from `flake.nix`. The devcontainer is optional ([D-0013](decisions.md#d-0013-build-entry)). |
| NFR-07 | Decisions and pitfalls keep stable ids. Completed gaps are written down instead of left as comments in code. |

## Trace to the notes

| Note | Captured by |
| --- | --- |
| `what.txt` | FR-01, FR-07, FR-08, FR-09, [D-0002](decisions.md#d-0002-product-functions) |
| `hardware.txt` | [D-0003](decisions.md#d-0003-host-board) through [D-0007](decisions.md#d-0007-bluetooth-transmitter), [D-0027](decisions.md#d-0027-microsd-purpose) |
| `ui.txt` | FR-04, FR-06, NFR-01, NFR-03, [D-0026](decisions.md#d-0026-second-hmi-button) |
| `readmap.txt` | [D-0012](decisions.md#d-0012-phase-split), FR-05, FR-10, FR-11, FR-12 |
