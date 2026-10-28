# Decisions

Status: baseline · 2026-10-28

Ids are `D-xxxx`. They are not reused and not renumbered. A later change supersedes an id by pointing at a new one. Status is **Decided** or **Undecided**. A recommendation under an open id is not a decision.

Items marked as a reasonable completion were filled in during bootstrap because the notes were silent. The raw notes stay in local `tmp/design-notes/` and are not tracked.

| Id | Status | Summary |
| --- | --- | --- |
| [D-0001](#d-0001-product-name) | Decided | The product name is Porta |
| [D-0002](#d-0002-product-functions) | Decided | Three jobs: Spotify, line bridge, network bridge |
| [D-0003](#d-0003-host-board) | Decided | Host is Core2 v1.3 |
| [D-0004](#d-0004-analog-module) | Decided | Analog I/O is Module Audio, SKU M144 |
| [D-0005](#d-0005-hmi-module) | Decided | Wheel and side buttons are Module HMI |
| [D-0006](#d-0006-base) | Decided | Stacked base is M5GO Bottom2 v1.3 |
| [D-0007](#d-0007-bluetooth-transmitter) | Decided | Stamp Pico transmits Bluetooth, AAC then SBC |
| [D-0008](#d-0008-radio-split) | Decided | Wi-Fi on the Core2, A2DP on the Stamp Pico |
| [D-0009](#d-0009-hmi-mapping) | Decided | Scroll selects, press confirms, button 2 goes back |
| [D-0010](#d-0010-interaction-model) | Decided | One action model; the wheel is primary |
| [D-0011](#d-0011-visual-language) | Decided | Flat color blocks, not a retro skin |
| [D-0012](#d-0012-phase-split) | Decided | Phase contents follow the roadmap note |
| [D-0013](#d-0013-build-entry) | Decided | Nix flake required, devcontainer optional |
| [D-0014](#d-0014-devcontainer-base) | Decided | Ubuntu 26.04 LTS, single-user Nix |
| [D-0015](#d-0015-nixpkgs) | Decided | nixpkgs `nixos-26.05` |
| [D-0016](#d-0016-documentation-language) | Decided | English |
| [D-0017](#d-0017-gitignore) | Decided | Ignore `tmp/` and `.cursor/`; track `.vscode/` |
| [D-0018](#d-0018-agent-shell) | Decided | Agent shell enters Porta, or Postern inside that submodule |
| [D-0019](#d-0019-terminal-shell) | Decided | Default terminal is Porta; Postern is a second profile |
| [D-0020](#d-0020-pitfall-record) | Decided | Pitfalls live in `docs/pitfalls.md` |
| [D-0021](#d-0021-no-firmware-tree) | Decided | No application tree in phase 0 |
| [D-0022](#d-0022-development-host) | Decided | Linux is the development host |
| [D-0023](#d-0023-separate-phase-1-prototypes) | Decided | Phase 1 player and bridge stay separate |
| [D-0024](#d-0024-editor-extensions) | Decided | Required extension list |
| [D-0025](#d-0025-microsd-in-phase-1) | Decided | microSD is unused in phase 1 |
| [D-0026](#d-0026-second-hmi-button) | Undecided | Side button index 3 |
| [D-0027](#d-0027-microsd-purpose) | Undecided | Config store is a candidate; offline Spotify is not |
| [D-0028](#d-0028-firmware-framework) | Decided | ESP-IDF on both boards |
| [D-0029](#d-0029-spotify-transport) | Decided | Core2 is a Connect speaker via cspot |
| [D-0030](#d-0030-login) | Undecided | Phase 1 login and the later easier login |
| [D-0031](#d-0031-board-link) | Decided | I2C on GPIO32/33, I2S on GPIO14/13/26 |
| [D-0032](#d-0032-i2s-format) | Decided | 16-bit stereo; rate follows the source, 44.1 or 48 |
| [D-0033](#d-0033-power-architecture) | Undecided | Which cell supplies the stack |
| [D-0034](#d-0034-screen-buttons) | Undecided | Idea: the on-screen label is the function |
| [D-0035](#d-0035-palette) | Undecided | Colors, with an interim prototype set |
| [D-0036](#d-0036-airplay-and-dlna) | Decided | AirPlay 1 and a short DLNA codec list |
| [D-0037](#d-0037-license) | Decided | Apache License 2.0 |
| [D-0038](#d-0038-enclosure-and-approvals) | Decided | The stack is the enclosure; no approvals |
| [D-0039](#d-0039-firmware-repos) | Decided | Core2 in this repo; Postern is the bridge component |
| [D-0040](#d-0040-python) | Undecided | The flake's Python is enough until a package is needed |

## Decided

### D-0001 Product name

- **Status:** Decided
- **Choice:** The product and the repository are named Porta.
- **Source:** Reasonable completion. The notes never name the product. The workspace directory is `porta`.
- **Consequence:** The flake, the devcontainer, and these docs use that name.

### D-0002 Product functions

- **Status:** Decided
- **Choice:** Porta has three jobs. A Spotify player sends Wi-Fi audio to the 3.5 mm output and to Bluetooth. A line bridge sends the 3.5 mm input to Bluetooth. A network bridge sends AirPlay and DLNA to the 3.5 mm output and to Bluetooth.
- **Source:** `what.txt`.
- **Consequence:** Use cases and requirements are built from these three jobs. Scope changes supersede this id.

### D-0003 Host board

- **Status:** Decided
- **Choice:** The host is M5Stack Core2 v1.3.
- **Source:** `hardware.txt` names Core2. `readmap.txt` names Core2 v1.3 for the player.
- **Consequence:** Bring-up targets that revision's PMU, IMU, and panel. A different Core2 revision is a new decision.

### D-0004 Analog module

- **Status:** Decided
- **Choice:** The 3.5 mm jacks are M5Stack Module Audio, SKU M144, ES8388 codec.
- **Source:** Reasonable completion. `hardware.txt` says "m5stack audio". Combined with the jack requirement in `what.txt`, M144 is the M5Stack module that provides those jacks. Vendor page: [Module Audio](https://docs.m5stack.com/en/module/Module_Audio).
- **Consequence:** The TRS jack is the line input. The TRRS jack is the headphone output. A different board supersedes this id.

### D-0005 HMI module

- **Status:** Decided
- **Choice:** The wheel and two side buttons are M5Stack Module HMI (STM32F030, I2C `0x41`).
- **Source:** `hardware.txt` and `ui.txt`. Vendor page: [Module HMI](https://docs.m5stack.com/en/module/Module_HMI).
- **Consequence:** UI code talks to that module. Button 1 is the encoder press. Buttons 2 and 3 are the side buttons.

### D-0006 Base

- **Status:** Decided
- **Choice:** When modules are stacked on Core2 v1.3, the base is M5GO Bottom2 v1.3, SKU A014-C-V13.
- **Source:** `hardware.txt` says "m5stack m5go bottom 2". Reasonable completion of the revision: Core2 v1.3's matching base is Bottom2 v1.3, and M5's Core2 v1.3 notes say to use M5GO Bottom2 instead of a Base-series base. Vendor page: [M5GO Bottom2 v1.3](https://docs.m5stack.com/en/base/Base_M5GO_Bottom2_v1.3).
- **Consequence:** The original Core2 battery bottom is removed for the stack. An older Bottom2 (MPU6886) is not the chosen base.

### D-0007 Bluetooth transmitter

- **Status:** Decided
- **Choice:** Bluetooth transmit is an M5Stamp Pico on an M5Stack Module Proto. The codec goal is AAC. SBC is the fallback.
- **Source:** `hardware.txt`.
- **Consequence:** The Stamp Pico firmware is a transmitter. SBC must pass before AAC is treated as done.

### D-0008 Radio split

- **Status:** Decided
- **Choice:** The Core2 uses Wi-Fi. The Stamp Pico is the Bluetooth transmitter. One radio does not do both jobs.
- **Source:** Reasonable completion of the hardware split in `hardware.txt`. The Stamp Pico is described as dedicated to Bluetooth transmit, and the player is described on the Core2.
- **Consequence:** Do not enable the Core2 as an A2DP source to "simplify" phase 1.

### D-0009 HMI mapping

- **Status:** Decided
- **Choice:** Wheel rotation moves the selection. Wheel press confirms. HMI button index 2 is a single Back action. The note's words cancel, back, and up are that one action: leave the current screen for its parent. On the root screen it does not open a hidden command.
- **Source:** `ui.txt`, plus a reasonable completion that "one button" is button index 2 (the first side button in the vendor API). Button index 3 stays open as [D-0026](#d-0026-second-hmi-button).
- **Consequence:** Firmware may swap index 2 and 3 only by superseding this id.

### D-0010 Interaction model

- **Status:** Decided
- **Choice:** Wheel, touch, and the three screen buttons emit one action set: focus next, focus previous, activate, back. The wheel is the primary control. A touch on an unfocused block moves focus. A touch on the focused block activates it.
- **Source:** Reasonable completion of the Blackberry wheel grammar in `ui.txt` together with a touchscreen that must stay intuitive.
- **Consequence:** Screens are not touch-only and not wheel-only. See [P-0011](pitfalls.md#p-0011-one-navigation-model).

### D-0011 Visual language

- **Status:** Decided
- **Choice:** The screen is a modern, simple layout of color blocks with large targets on 320×240. It does not imitate a Blackberry, a phone home screen, or a desktop window.
- **Source:** `ui.txt`.
- **Consequence:** Phase 2 may add density. It does not replace the block model. Colors themselves are [D-0035](#d-0035-palette).

### D-0012 Phase split

- **Status:** Decided
- **Choice:** Phase 1 is the Stamp Pico I2S-to-Bluetooth bridge, the Core2 Wi-Fi-to-I2S player, and a simple complete UI with a real UI API and a working HMI. Phase 2 is the elaborated UI, auxiliary bridge modes, easier login, and battery level plus low-battery power-off.
- **Source:** `readmap.txt`.
- **Consequence:** AirPlay and DLNA belong to phase 2 because `what.txt` requires them and `readmap.txt` places bridge features after phase 1. See [D-0023](#d-0023-separate-phase-1-prototypes).

### D-0013 Build entry

- **Status:** Decided
- **Choice:** The supported build entry is a Nix flake. A devcontainer is optional and must end in the same `nix develop` shell.
- **Source:** Bootstrap request.
- **Consequence:** Instructions that only work inside the container are not the supported path.

### D-0014 Devcontainer base

- **Status:** Decided
- **Choice:** The container image is Ubuntu 26.04 LTS. Nix is installed with the official single-user installer (`--no-daemon`), flakes enabled, for the `vscode` user.
- **Source:** Reasonable completion. "Ubuntu LTS" at the time of bootstrap is 26.04, not the interim 26.10. Single-user Nix matches a one-user container.
- **Consequence:** Host Nix settings are not copied into the image. The container sandbox exception is [P-0019](pitfalls.md#p-0019-container-sandbox).

### D-0015 Nixpkgs

- **Status:** Decided
- **Choice:** `nixpkgs` follows `github:NixOS/nixpkgs/nixos-26.05`. Devshells exist for `x86_64-linux` and `aarch64-linux`.
- **Source:** Reasonable completion. 26.05 is the stable channel at bootstrap. The host is Linux.
- **Consequence:** Moving to `nixos-unstable` or adding Darwin is a new decision.

### D-0016 Documentation language

- **Status:** Decided
- **Choice:** The suite and the README are English.
- **Source:** Reasonable completion. The design notes are English.
- **Consequence:** A translation would be a second document, not a silent replacement.

### D-0017 Gitignore

- **Status:** Decided
- **Choice:** Git ignores `tmp/`, `.cursor/`, Nix `result` links, and `.direnv/`. Git tracks `.vscode/`.
- **Source:** Bootstrap request, plus a reasonable completion for Nix outputs and for keeping the editor setup available to people who do not use Cursor.
- **Consequence:** Pitfalls and decisions must live under `docs/`. Cursor rules can disappear on a fresh clone. [operations](operations.md) says how to restore the hook. Inside this Git checkout, Nix only evaluates staged or committed files ([P-0022](pitfalls.md#p-0022-untracked-flake-files)).

### D-0018 Agent shell

- **Status:** Decided
- **Choice:** Every agent Shell command runs inside `nix develop`, using zsh if that binary is visible to the hook, otherwise bash. A command whose working directory is inside `postern/` uses that submodule's flake. If `IN_NIX_SHELL` is already set, or the command already starts with `nix develop`, it is not wrapped again.
- **Source:** Bootstrap request. The Postern hook was added when that submodule became a place to build.
- **Consequence:** Two `preToolUse` entries share the Shell tool. `.cursor/hooks/nix-develop-postern.py` rewrites only `postern/` commands. `.cursor/hooks/nix-develop.py` selects the same flake, so either entry is enough and they do not nest ([P-0014](pitfalls.md#p-0014-hook-reentry)). Tools the agent needs are packages in the devshell it enters.

### D-0019 Terminal shell

- **Status:** Decided
- **Choice:** The default integrated terminal runs `.vscode/nix-develop-terminal.sh`, which enters this repo's `nix develop` with zsh when the host has zsh, otherwise bash. The profile **Postern** runs `.vscode/nix-develop-postern-terminal.sh` and enters the submodule's devshell.
- **Source:** Bootstrap request. The Postern profile was added with the submodule hook.
- **Consequence:** Both shells are packages in each devshell so `nix develop -c` can find the one the script selects. The default profile stays this repo.

### D-0020 Pitfall record

- **Status:** Decided
- **Choice:** `docs/pitfalls.md` is the record. Cursor rules tell the agent to read it, to append new pitfalls, and to maintain the rules themselves.
- **Source:** Bootstrap request.
- **Consequence:** A pitfall that exists only in a chat is not recorded.

### D-0021 No firmware tree

- **Status:** Decided
- **Choice:** Phase 0 does not add application source or a `firmware/` tree. The framework itself is [D-0028](#d-0028-firmware-framework).
- **Source:** Reasonable completion. An empty tree would imply a layout the firmware has not earned yet.
- **Consequence:** The first firmware change includes the layout and supersedes this id's "no tree" clause. It does not reuse the id.

### D-0022 Development host

- **Status:** Decided
- **Choice:** The supported host is Linux, either Nix on the host or the Ubuntu devcontainer.
- **Source:** Reasonable completion of the flake systems and the Ubuntu container.
- **Consequence:** No Windows terminal profile is configured.

### D-0023 Separate phase 1 prototypes

- **Status:** Decided
- **Choice:** In phase 1 the Core2 player and the Stamp Pico bridge are separate programs. The product behaviour that feeds the player into Bluetooth, and the line-in bridge, is phase 2 auxiliary mode.
- **Source:** `readmap.txt` lists the bridge and the player as separate phase 1 bullets, and lists auxiliary mode under phase 2.
- **Consequence:** Phase 1 can pass UC-02 without the Core2 UI controlling the Stamp Pico.

### D-0024 Editor extensions

- **Status:** Decided
- **Choice:** The required extensions are `jnoortheen.nix-ide`, `editorconfig.editorconfig`, `DavidAnson.vscode-markdownlint`, `yzhang.markdown-all-in-one`, `ms-vscode.cpptools`, `ms-vscode.vscode-serial-monitor`, and `ms-python.python`.
- **Source:** Reasonable completion from the skeleton: Nix, Markdown, EditorConfig, C or C++, serial logs, and Python.
- **Consequence:** The Espressif VS Code extension stays off this list until the devshell provides `idf.py`. That extension installs its own IDF copy and would fight the flake.

### D-0025 microSD in phase 1

- **Status:** Decided
- **Choice:** Phase 1 does not read or write the microSD slot.
- **Source:** Reasonable completion. `hardware.txt` leaves the slot's purpose open, and phase 1 does not need it.
- **Consequence:** Credentials, caches, and music files do not go on the card in phase 1. The later purpose is [D-0027](#d-0027-microsd-purpose).

### D-0028 Firmware framework

- **Status:** Decided
- **Choice:** Both the Core2 and the Stamp Pico are ESP-IDF applications. One IDF generation covers both. Arduino and PlatformIO are not used.
- **Source:** Project decision, 2026-10-28.
- **Consequence:** `flake.nix` installs `esptool`, CMake, and Ninja from nixpkgs 26.05. That pin has no ESP-IDF package and no Xtensa toolchain. The usual Nix ESP-IDF flakes are pinned to older nixpkgs and currently break on 26.05 because Python 3.10 was removed. They are not inputs of this flake. `idf.py` arrives in a later change that can build against this pin. There is still no `firmware/` tree ([D-0021](#d-0021-no-firmware-tree)).

### D-0029 Spotify transport

- **Status:** Decided
- **Choice:** With a Spotify Premium account, the Core2 is a Connect speaker. It loads and decodes the audio itself. The phone is the remote that chooses the device and the music. Play, pause, next, previous, and volume on the Core2 call that same session. The Web API is only for later catalog actions such as search and starting a chosen playlist.
- **Component:** [cspot](https://github.com/feelfreelinux/cspot), the C++ Connect library aimed at the ESP32. Vendor the fork that [squeezelite-esp32](https://github.com/sle118/squeezelite-esp32) still patches, [philippe44/cspot](https://github.com/philippe44/cspot), not the last upstream snapshot. Upstream fails to build and its username login is declined. squeezelite-esp32 is the reference for commands and track metadata (`components/spotify`). It is not the Porta application. Porta's own I2S sink feeds Module Audio. The PCM5102 sink in cspot is the shape of that adapter.
- **Source:** Project decision, 2026-10-28. Connect speakers pull the track from Spotify. ESP Muse boards running this library were still playing in 2026, including after a December 2025 break that a squeezelite-esp32 update repaired.
- **Consequence:** cspot needs PSRAM, so it runs on the Core2 only. The first login is Zeroconf from a Spotify app on the same network. Credentials are stored after that ([D-0030](#d-0030-login)). A tone is still allowed as a bring-up stand-in before cspot is linked, and it must be labelled as one. Integrators mark this fork GPL-3.0. Read the license file at the pinned commit before linking. If it is GPL-3.0, the firmware binary is GPL-3.0 and does not stay an Apache-2.0-only distribution ([D-0037](#d-0037-license), [P-0026](pitfalls.md#p-0026-cspot-fork)).

### D-0031 Board link

- **Status:** Decided
- **Choice:** The cable is I2C plus I2S. The Core2 is I2S master on I2S1 and I2C controller on a bus of its own. The Stamp Pico is I2S slave and I2C peripheral. Audio on the cable goes only from the Core2 to the Stamp Pico.

  | Signal | Core2 GPIO | Reachable on | Stamp Pico |
  | --- | --- | --- | --- |
  | I2C SDA | GPIO32 | PORT.A yellow, M-Bus pin 19 | GPIO21 |
  | I2C SCL | GPIO33 | PORT.A white, M-Bus pin 20 | GPIO22 |
  | I2S BCLK | GPIO14 | PORT.C yellow, M-Bus pin 16 | GPIO18 |
  | I2S LRCK | GPIO13 | PORT.C white, M-Bus pin 15 | GPIO19 |
  | I2S DOUT | GPIO26 | PORT.B yellow, M-Bus pin 10 | GPIO36 DIN |
  | GND | — | Grove black | GND |

- **Source:** The link type is a project decision, 2026-10-28. The pins are taken from the [Core2 v1.3 pin map](https://docs.m5stack.com/en/core/Core2_v1.3) and from the Grove colors on ExtPort For Core2. Module HMI brings PORT.B and PORT.C out of the stack. PORT.A passes through on M-Bus pins 19 and 20.
- **Consequence:** GPIO21 and GPIO22 stay the internal I2C bus (AXP192, touch, RTC, IMU, HMI, Module Audio). GPIO0, GPIO2, GPIO12, and GPIO34 stay the onboard codec and are not wired to the Stamp Pico. Core2 GPIO36 is the other PORT.B wire and is input-only, so it is left unconnected. Stamp Pico GPIO36 is a different pin and is the cable's data input. Stamp Pico GPIO26 stays free for a later return path and is not wired. Grove red is 5 V and must not land on a GPIO ([P-0025](pitfalls.md#p-0025-grove-5v)). Wire colors follow ExtPort For Core2. Confirm them on the HMI connector before soldering. The GPIO numbers are the contract ([P-0023](pitfalls.md#p-0023-core2-pin-map)). The I2C address on this second bus is `0x1D` (Postern [D-0016](../postern/docs/decisions.md#d-0016-i2c-address)). Pins the ESP32 GPIO matrix may use for a further I2S are listed in [architecture](architecture.md). The Stamp Pico side is Postern [D-0017](../postern/docs/decisions.md#d-0017-reference-pins).

### D-0032 I2S format

- **Status:** Decided
- **Choice:** The cable is 16-bit stereo. SBC and AAC both support 44.1 kHz and 48 kHz, so the Bluetooth rate follows the music source when the source is one of those two. The I2S clock uses that same rate. The Core2 is the only resampler. The Stamp Pico does not resample. If the headphones do not offer the source rate, A2DP negotiation picks the other of the two, and the Core2 converts to the negotiated rate. Any other source rate is converted to 44.1 or 48 before the cable.
- **Source:** Project rule, 2026-10-28, after checking the A2DP SBC capability bits in ESP-IDF (`ESP_A2D_SBC_CIE_SF_44K` and `ESP_A2D_SBC_CIE_SF_48K`) and the sample rates of the Espressif SBC and AAC encoders. Spotify's own hardware documentation treats 44.1 kHz stereo as the normal track format and 48 kHz as rare.
- **Consequence:** Phase 1 Spotify audio runs at 44.1 kHz in practice. AAC encoding support is not the same thing as an A2DP source endpoint. See [P-0024](pitfalls.md#p-0024-aac-source).

### D-0036 AirPlay and DLNA

- **Status:** Decided
- **Choice:** Phase 2 receives AirPlay 1 (RAOP) and acts as a DLNA audio renderer. AirPlay 2 stays out. It needs an Apple license. Porta plays a URI a controller sets. It does not browse a media server.
- **Codecs:** Every row is 16-bit stereo at 44.1 kHz or 48 kHz, the same contract as the I2S cable ([D-0032](#d-0032-i2s-format)). The Core2 resamples any other rate inside these codecs to one of those two.

  | Path | Codec | Notes |
  | --- | --- | --- |
  | AirPlay 1 | ALAC | The codec iOS and macOS send on RAOP. |
  | DLNA | LPCM (`audio/L16`) | 44100 and 48000 only. |
  | DLNA | WAV PCM (`audio/wav`) | The same PCM shape as LPCM. |
  | DLNA | ALAC | The AirPlay decoder, under the same limits. |

- **Source:** The recommendation on this id, accepted 2026-10-28. `what.txt` names AirPlay and DLNA without versions. `readmap.txt` places them in phase 2. RAOP on ESP32 decodes ALAC to 16-bit PCM.
- **Consequence:** The renderer advertises only the DLNA rows. MP3, AAC, FLAC, Opus, Vorbis, WMA, and DSD are not receive codecs here. Vorbis stays inside cspot ([D-0029](#d-0029-spotify-transport)). Bluetooth stays SBC, then AAC ([D-0007](#d-0007-bluetooth-transmitter)). See [P-0027](pitfalls.md#p-0027-receiver-codecs).

### D-0037 License

- **Status:** Decided
- **Choice:** The software is Apache License 2.0. The text is `LICENSE`. The copyright line is `Copyright 2026 oemaix`.
- **Source:** Project decision, 2026-10-28.
- **Consequence:** New source files carry the Apache 2.0 header when they are added. This id does not grant a Spotify or Bluetooth license.

### D-0038 Enclosure and approvals

- **Status:** Decided
- **Choice:** The stacked modules are the enclosure. There is no separate shell. Radio approval and safety approval are out of scope.
- **Source:** Project decision, 2026-10-28.
- **Consequence:** Do not draw a case. Do not plan certification. The stack order of Module HMI and Module Audio is still free, except that the base is M5GO Bottom2 v1.3 and the original Core2 battery bottom comes off ([D-0006](#d-0006-base)). Cables use the Grove ports that remain reachable on that stack ([D-0031](#d-0031-board-link)).

### D-0039 Firmware repos

- **Status:** Decided
- **Choice:** The Core2 application lives in this repo when firmware work starts. The Stamp Pico bridge is the ESP-IDF component [Postern](../postern/README.md), pinned as the submodule `postern`. Its URL is `https://github.com/oemaix/postern.git`, branch `main`. Another project builds that component into its own application and flashes that image.
- **Shared surface:** The cable contract lives in Postern. I2S is 16-bit stereo at 44.1 or 48 kHz, Core2 master, audio only toward the Stamp Pico ([D-0031](#d-0031-board-link), [D-0032](#d-0032-i2s-format)). I2C is Postern's register file (`include/postern_regs.h` there): rate, start, stop, and link status. The 7-bit address is still open. GPIOs on the Stamp Pico are Postern Kconfig, chosen at compile time, and they are not the Core2 numbers in [D-0031](#d-0031-board-link). UI, cspot, the ES8388, the HMI, and the resampler stay on the Core2. A2DP, SBC, and AAC stay in Postern.
- **Source:** Project decision, 2026-10-28, accepting the recommendation on this id. The name followed the same day. The submodule URL became the GitHub remote on `main` once that repository existed.
- **Consequence:** Phase 0 still has no Core2 `firmware/` tree ([D-0021](#d-0021-no-firmware-tree)). A shared-components repository waits until the cable header outgrows Postern. [P-0028](pitfalls.md#p-0028-postern-file-url) is resolved.

## Undecided

### D-0026 Second HMI button

- **Status:** Undecided
- **Options:** Leave it unused. Bind it to a mode switch. Bind it to play/pause.
- **Recommendation:** Leave it unused through phase 1. Play/pause is Activate on the transport block, which keeps one action model.
- **Source:** `ui.txt` says the other button is undecided.
- **Consequence:** Software reports the button and does not act on it.

### D-0027 microSD purpose

- **Status:** Undecided
- **Intent:** The card stores music downloaded from Spotify for offline playback.
- **Finding:** That job is not available to this device through a supported API. Official offline files are encrypted and bound to Spotify's own apps. The Web API does not return full tracks. Track previews are short clips, not a library. Copying an official offline cache onto the card will not play.
- **Options still open:** Leave the slot unused. Use it later if a supported offline mechanism appears. Store audio that is not Spotify. Use the card as a config store.
- **Config-store candidate:** Remembered AirPlay senders and DLNA sources, plus saved status such as the last source and whether playback was running. Added 2026-10-28. This is the use that fits the hardware now that offline Spotify does not.
- **Source:** Project note, 2026-10-28. `hardware.txt` originally left the slot unnamed.
- **Consequence:** No filesystem layout is specified, and this id stays open. Phase 1 still does not mount the card ([D-0025](#d-0025-microsd-in-phase-1)). Credentials stay off the card. Do not write a downloader that assumes the official offline feature can be reused ([P-0010](pitfalls.md#p-0010-microsd)).

### D-0030 Login

- **Status:** Undecided
- **Options:** Phase 1 uses Zeroconf from the Spotify app, then stored credentials. A setup screen types the account. The Web API gets its own OAuth token later.
- **Recommendation:** Zeroconf once, then the stored blob. The Core2 does not ask for a Spotify password. "Easier login" in phase 2 means fewer steps around that first discovery, not a keyboard.
- **Source:** `readmap.txt` puts an easier login in phase 2, which implies phase 1 may be awkward, and does not say what either method is.
- **Consequence:** The Connect screen has a placeholder and no fields yet.

### D-0033 Power architecture

- **Status:** Undecided
- **Options:** M5GO Bottom2 cell is the only pack, charged by its TP4057, measured by an added gauge or by inference. The Core2 AXP192 still measures a pack on its battery pins. The HMI cell is isolated and unused. A bench supply powers phase 1 and the product decision waits.
- **Recommendation:** Use a bench supply for the two phase 1 prototypes. Before the stack is closed up, draw one pack, one charger, and one measurement point. Do not parallel the HMI cell, the Bottom2 cell, and the AXP192 pack.
- **Source:** Reasonable completion of the mechanical notes on Core2 v1.3. The notes themselves do not describe power.
- **Consequence:** FR-11 cannot be implemented against a guessed gauge. [P-0004](pitfalls.md#p-0004-stack-mechanics) and [P-0005](pitfalls.md#p-0005-parallel-cells) still apply.

### D-0034 Screen buttons

- **Status:** Undecided
- **Current idea:** The function of each screen button is the label drawn at the bottom of the screen. The label may change with the screen. The button still emits one of the shared actions. The screen decides which action the label means.
- **Not chosen yet:** Firmware does not bind the three buttons until a screen actually draws those labels.
- **Source:** Project note, 2026-10-28. `ui.txt` names the buttons and does not assign them.
- **Consequence:** The phase 1 API accepts the events. Until labels exist, it drops them. A fixed bezel legend would supersede this idea.

### D-0035 Palette

- **Status:** Undecided
- **Options:** A neutral dark UI. A light UI. A brand palette still to be designed.
- **Recommendation:** Prototypes may use background `#1C1C1E`, surface `#2C2C2E`, text `#F5F5F7`, accent `#3D7EFF`, confirm `#2F9E62`, and danger `#D64545`. These values are not a brand decision.
- **Source:** `ui.txt` says color blocks and does not give colors.
- **Consequence:** No asset pipeline for a theme.

### D-0040 Python

- **Status:** Undecided
- **Options:** The flake's `python3` only. `uv` plus a gitignored `.venv` as soon as the shell exists. `pip` into that environment.
- **Recommendation:** The flake is enough today. The hook and the shell scripts use the standard library and nothing from PyPI. Add `uv` to `flake.nix`, and create a gitignored `.venv` with it, on the day a third-party package is imported. Do not use `pip`. Keep that environment separate from the Python tree ESP-IDF will install for `idf.py`.
- **Source:** Project question, 2026-10-28.
- **Consequence:** No `.venv`, no `uv`, and no `requirements.txt` until this id closes. Postern's shell matches this one. [P-0029](pitfalls.md#p-0029-pip).

