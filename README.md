# Porta

Porta is a portable audio appliance built on M5Stack Core2 v1.3. It plays a Spotify stream, bridges a 3.5 mm input to Bluetooth headphones, and, in a later phase, receives AirPlay and DLNA. Wi-Fi stays on the Core2. Bluetooth transmission stays on a separate M5Stamp Pico.

**Status:** Phase 0. This tree holds the specification, the development skeleton, and the [Postern](postern/README.md) submodule. There is no Core2 application yet. The current phase and the open choices are in [docs/roadmap.md](docs/roadmap.md) and [docs/decisions.md](docs/decisions.md).

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
firmware/              Core2 application, built with idf.py
postern/               I2S-to-Bluetooth component submodule
flake.nix              development shell
.devcontainer/         optional Ubuntu LTS + Nix
.vscode/               terminal profile and required extensions
.cursor/               local Cursor rules and hook (not tracked)
tmp/                   local design scratch (not tracked)
```

The firmware framework is ESP-IDF v6.1 ([D-0028](docs/decisions.md#d-0028-firmware-framework)). The Core2 application is [firmware/](firmware/README.md) and builds with `idf.py` from this shell ([D-0043](docs/decisions.md#d-0043-firmware-tree)). Postern is the bridge component ([D-0039](docs/decisions.md#d-0039-firmware-repos)). The license is Apache 2.0.
