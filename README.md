# Porta

Porta is a portable audio appliance built on M5Stack Core2 v1.3. It plays a Spotify stream, bridges a 3.5 mm input to Bluetooth headphones, and, in a later phase, receives AirPlay and DLNA. Wi-Fi stays on the Core2. Bluetooth transmission stays on a separate M5Stamp Pico.

**Status:** Phase 1 is stalled. The Core2 image announces porta on Spotify Connect. A phone can select it, the session authenticates, and the track name arrives. The access point then refuses the legacy audio key (`audio key error 0 1`, packet `0x0E`). No samples are decoded, so the TRRS jack stays silent. The Connect implementation is incomplete. It resumes when an upstream client can obtain that key. There is no Spotify Connect device here to reverse engineer the newer license handshake. The phase list and the open choices are in [docs/roadmap.md](docs/roadmap.md) and [docs/decisions.md](docs/decisions.md).

## Spotify playback

On 2026-11-01 the bench log showed a correct key request (`file 20 track 16`) and the refusal `0 1`. Login, metadata, and the phone's play and pause state still succeed. Those later packets are the Connect remote, not the audio file. cspot, including the [philippe44/cspot](https://github.com/philippe44/cspot) pin in `firmware/third_party/cspot`, sends this same legacy request. The reports of that refusal:

- [librespot #1649](https://github.com/librespot-org/librespot/issues/1649), the pinned thread for `error audio key 0 1`.
- [librespot #1664](https://github.com/librespot-org/librespot/issues/1664) (2025-12-23): every track is skipped, on Home Assistant and on Windows.
- [librespot #1735](https://github.com/librespot-org/librespot/issues/1735) (2026-08): one long-standing Premium account still receives a key, a Premium account created in June 2026 does not, on the same program and the same tracks.
- [Volumio, 2026-10-15](https://community.volumio.com/t/2026-spotify-plugin/77605/17): newer Premium accounts in one family plan fail, older accounts in that plan play. The log shows `LicenseVersion`, then `aes key with code 1`.
- [go-librespot #317](https://github.com/devgianlu/go-librespot/issues/317): the same code `1`. The thread treats the newer license handshake as the remaining path, and records that these projects do not implement it.

The rollout described there is per account. A Premium subscription several years old can already be on the refusing side. Account age does not change the reply. This tree has one such account and no second account to compare.

## Documentation

| Document | Contents |
| --- | --- |
| [Overview](docs/overview.md) | What the product is |
| [Vision and scope](docs/vision-and-scope.md) | Aims, boundaries, phases |
| [Glossary](docs/glossary.md) | Terms used in the suite |
| [Use cases](docs/use-cases.md) | What a listener can do |
| [Requirements](docs/requirements.md) | Functional and non-functional requirements |
| [UI design](docs/ui-design.md) | Inputs, screens, visual rules |
| [Architecture](docs/architecture.md) | Boards, audio paths, buses |
| [Schematics](docs/hardware/schematics.md) | Vendor circuit prints for the boards |
| [Cable](https://github.com/oemaix/postern/blob/main/docs/cable.md) | Application note for the I2C and I2S link. It lives in Postern. |
| [Software](docs/software.md) | Core2 components and how they meet |
| [Operations](docs/operations.md) | Devshell, container, editor, agent hook |
| [Roadmap](docs/roadmap.md) | Phases and current status |
| [Decisions](docs/decisions.md) | Decided and undecided items (`D-xxxx`) |
| [Pitfalls](docs/pitfalls.md) | Traps to avoid (`P-xxxx`) |

The notes under `tmp/` are local scratch and are not part of the clone. This suite is the specification.

## Development

The supported entry is the Nix flake:

```sh
nix develop
```

The default terminal profile **Nix Develop** runs the same shell, using zsh when the host has zsh and bash otherwise. An optional Ubuntu 26.04 devcontainer installs Nix and opens the same shell. Details, required extensions, and the agent hook are in [docs/operations.md](docs/operations.md).

## Layout

```text
README.md
docs/                  specification
firmware/              Core2 application, Rust on ESP-IDF v5.5.6
postern/               Stamp Pico firmware submodule, built with its own idf.py
flake.nix              development shell
.devcontainer/         optional Ubuntu LTS + Nix
.vscode/               terminal profile and required extensions
.cursor/               local Cursor rules and hook (not tracked)
tmp/                   local design scratch (not tracked)
```

The Core2 application is [firmware/](firmware/README.md), Rust on ESP-IDF v5.5.6 ([D-0046](docs/decisions.md#d-0046-core2-language)). Postern is the Stamp Pico image, C on ESP-IDF v6.1, in the submodule, with its own shell ([D-0045](docs/decisions.md#d-0045-postern-image)). The license is Apache 2.0.
