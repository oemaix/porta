# Vision and scope

Status: baseline · 2026-10-29

## Vision

Porta should feel like a single-purpose player, not like a development kit. The listener moves through a few color-block screens with a wheel, confirms with a press, and backs out with one button. Playback, a line-in bridge, and a network receiver are modes of that same object.

The Blackberry 7290 and 8100 are the reference for the wheel grammar only. They are not a visual target ([D-0011](decisions.md#d-0011-visual-language), [P-0021](pitfalls.md#p-0021-blackberry-pixels)).

## In scope

- The three jobs in [D-0002](decisions.md#d-0002-product-functions), delivered in the phases in [D-0012](decisions.md#d-0012-phase-split).
- The named M5Stack hardware, as identified in [D-0003](decisions.md#d-0003-host-board) through [D-0007](decisions.md#d-0007-bluetooth-transmitter).
- A UI API that is larger than the first screens, so later screens do not invent a second interaction model ([D-0010](decisions.md#d-0010-interaction-model)).
- One Core2 image whose panel and player meet at commands and a status snapshot, so later sources keep that seam ([D-0042](decisions.md#d-0042-core2-components)).
- A Nix development shell, and an optional Ubuntu LTS devcontainer, as the way people build ([D-0013](decisions.md#d-0013-build-entry)).

## Out of scope

- A phone or desktop app as a Porta product surface. The Spotify app on a phone is the remote for Connect ([D-0029](decisions.md#d-0029-spotify-transport)). It does not carry the audio.
- A local music library other than the offline-Spotify idea in [D-0027](decisions.md#d-0027-microsd-purpose). That idea is not currently possible through a supported API.
- Voice assistants, equalizers, multi-room groups, and playlists beyond what the chosen Spotify path already provides.
- A separate enclosure, radio approval, and safety approval ([D-0038](decisions.md#d-0038-enclosure-and-approvals)). The license is Apache 2.0 ([D-0037](decisions.md#d-0037-license)).

## Phases

Phase 1 proves the two hard paths separately and puts a usable UI on the Core2. Phase 2 makes one product out of them and adds the remaining listener-facing behaviour. Work from phase 2 does not land in phase 1 ([P-0012](pitfalls.md#p-0012-phase-creep)).

## Non-goals for the skeleton

The firmware framework is ESP-IDF ([D-0028](decisions.md#d-0028-firmware-framework)). The Core2 application is `firmware/` and is not linked ([D-0043](decisions.md#d-0043-firmware-tree)).
