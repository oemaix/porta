# Pitfalls

Status: baseline · 2026-10-28

Open entries are binding. Do not delete an entry. Mark it resolved, with the reason, when it stops applying. New entries take the next id and are added in the same change that discovered them.

Agents read this file before design or implementation work, and they add to it when a new trap shows up. The Cursor rule is only a reminder. This file is the copy that stays in git.

## Open

### P-0001 Spotify on the ESP32

- **Status:** Open
- **Avoid:** Do not implement the Spotify player as librespot, as a browser inside the Core2, as the Web Playback SDK, as the partner embedded SDK, or as the unmodified upstream cspot snapshot.
- **Why:** The Web Playback SDK plays in a browser and does not return PCM. librespot does not run on the ESP32. The embedded SDK is not a public download. The Connect path that does run here is [D-0029](decisions.md#d-0029-spotify-transport). A tone or a plain HTTP stream may stand in until that library is linked, and it must be labelled as a stand-in.

### P-0002 One radio, two jobs

- **Status:** Open
- **Avoid:** Do not run Wi-Fi station and A2DP source on the Core2.
- **Why:** Classic Bluetooth and Wi-Fi share the radio and coexist poorly at audio rates. [D-0008](decisions.md#d-0008-radio-split) puts transmit on the Stamp Pico for that reason.

### P-0003 No PSRAM

- **Status:** Open
- **Avoid:** Do not treat AAC as the first Bluetooth codec to bring up on the Stamp Pico.
- **Why:** The Stamp Pico is an ESP32-PICO-D4 without PSRAM. An AAC encoder plus A2DP can exhaust RAM that SBC does not. [D-0007](decisions.md#d-0007-bluetooth-transmitter) requires SBC as the fallback that actually ships. The 2026-10-28 check of the codec path is [P-0024](#p-0024-aac-source). Measure free heap on this board after SBC is stable. If AAC does not fit, record the numbers here and supersede the codec goal with a new decision.

### P-0004 Stack mechanics

- **Status:** Open
- **Avoid:** Do not mount Core2 v1.3 on a Base-series base. Do not leave the original battery bottom attached when modules are stacked.
- **Why:** M5 documents interference between the Core2 v1.3 vibration motor and Base-series bases, and tells you to remove the battery bottom when stacking modules. [D-0006](decisions.md#d-0006-base) selects M5GO Bottom2 v1.3 so a battery can remain.

### P-0005 Parallel cells

- **Status:** Open
- **Avoid:** Do not tie the HMI module cell, the M5GO Bottom2 cell, and the Core2 pack together as one battery.
- **Why:** Each has its own charger. Paralleling them cooks a cell. [D-0033](decisions.md#d-0033-power-architecture) has to name the one pack before the case is closed.

### P-0006 Internal I2S

- **Status:** Open
- **Avoid:** Do not drive the Core2 NS4168 speaker or SPM1423 microphone at the same time as Module Audio.
- **Why:** They use the M5-Bus I2S pins the codec needs. Disable the internal devices when Module Audio owns the bus.

### P-0007 I2C map

- **Status:** Open
- **Avoid:** Do not add an I2C device without scanning the assembled stack.
- **Why:** Known addresses already include HMI `0x41`, audio MCU `0x33`, ES8388 `0x10`, AXP192 `0x34`, FT6336U `0x38`, and BM8563 `0x51`. A module that "should" stack can still collide.

### P-0008 TRS jack

- **Status:** Open
- **Avoid:** Do not treat the two 3.5 mm jacks as a symmetric line-in and line-out pair.
- **Why:** On Module Audio the TRS jack is microphone only. Headphones use the TRRS jack, whose mic wiring is either CTIA or OMTP. Line-in is the TRS jack. Confirm the ES8388 input mux against the jack you intend.

### P-0009 Unbound HMI button

- **Status:** Open
- **Avoid:** Do not give HMI button index 3 a command.
- **Why:** [D-0026](decisions.md#d-0026-second-hmi-button) is open. A convenient binding becomes a behaviour the listener learns, and then a compatibility problem.

### P-0010 microSD

- **Status:** Open
- **Avoid:** Do not store credentials on the card. Do not copy Spotify's official offline files onto it and expect them to play. Do not mount the card in phase 1.
- **Why:** Offline Spotify files are encrypted and bound to Spotify's apps. The Web API does not supply full tracks. A config store for sources and playback status is only a candidate in [D-0027](decisions.md#d-0027-microsd-purpose). [D-0025](decisions.md#d-0025-microsd-in-phase-1) still keeps the slot out of phase 1.

### P-0011 One navigation model

- **Status:** Open
- **Avoid:** Do not add touch gestures, screen-button shortcuts, or encoder acceleration that bypass the shared actions.
- **Why:** [D-0010](decisions.md#d-0010-interaction-model). Two navigation models drift apart the moment a screen is added.

### P-0012 Phase creep

- **Status:** Open
- **Avoid:** Do not implement AirPlay, DLNA, auxiliary mode, the easier login, battery UI, or the elaborated layout in phase 1.
- **Why:** [D-0012](decisions.md#d-0012-phase-split). Those items have open decisions of their own. Landing them early freezes a guess.

### P-0013 Devshell PATH

- **Status:** Open
- **Avoid:** Do not call a host tool from an agent command and assume it is still on `PATH`.
- **Why:** `nix develop` replaces `PATH`. If a command needs a tool, add it to `flake.nix`. Absolute host paths are a last resort and should be called out in the change.

### P-0014 Hook reentry

- **Status:** Open
- **Avoid:** Do not wrap a command that already starts with `nix develop`, and do not enter a second devshell when `IN_NIX_SHELL` is set.
- **Why:** The agent hooks run on every Shell command ([D-0018](decisions.md#d-0018-agent-shell)). Nesting two devshells hides the command. `.cursor/hooks/nix-develop-postern.py` rewrites a command only when its working directory is inside `postern/`. `.cursor/hooks/nix-develop.py` selects that same flake. Both leave a command alone when it already contains `nix develop -c` or `IN_NIX_SHELL` is set.

### P-0015 Cursor files are local

- **Status:** Open
- **Avoid:** Do not put the only copy of a decision, a pitfall, or an operational step under `.cursor/`.
- **Why:** [D-0017](decisions.md#d-0017-gitignore). Other participants may not use Cursor. A fresh clone has no rules and no hook until those files are restored from the bootstrap skill.

### P-0016 Touch edges

- **Status:** Open
- **Avoid:** Do not place the only target for an action on the extreme edge of the Core2 panel.
- **Why:** M5 notes that some Core2 panels are non-linear at the edges and points at a screen-firmware update. Inset the blocks.

### P-0017 One clock master

- **Status:** Open
- **Avoid:** Do not let both the Core2 and the Stamp Pico drive the I2S bit clock and word clock.
- **Why:** Two masters on one wire is not a link. [D-0031](decisions.md#d-0031-board-link) and [D-0032](decisions.md#d-0032-i2s-format) have to name the master. Share a ground on that cable.

### P-0018 Brownout

- **Status:** Open
- **Avoid:** Do not start Wi-Fi transmit and I2S playback on the Core2 v1.3 without setting the AXP192 current limit on purpose and measuring the rail.
- **Why:** The PMU will brown out under the combined load and the failure looks like a random reboot. NFR-04.

### P-0019 Container sandbox

- **Status:** Open
- **Avoid:** Do not copy `sandbox = false` from the devcontainer Nix config onto a host Nix install.
- **Why:** It is there because a single-user Nix inside Docker often cannot use the user-namespace sandbox ([D-0014](decisions.md#d-0014-devcontainer-base)). The host keeps its own sandbox.

### P-0020 No toolchain yet

- **Status:** Resolved
- **Resolution:** [D-0028](decisions.md#d-0028-firmware-framework) selected ESP-IDF. Arduino and PlatformIO stay out. The application tree stays out until a firmware change ([D-0021](decisions.md#d-0021-no-firmware-tree)). Do not add an ESP-IDF Nix flake that is pinned to an older nixpkgs.

### P-0021 Blackberry pixels

- **Status:** Open
- **Avoid:** Do not skin the UI as a Blackberry 7290 or 8100.
- **Why:** Those phones are the reference for the wheel grammar in `ui.txt`, not for the pixels. [D-0011](decisions.md#d-0011-visual-language) is color blocks.

### P-0022 Untracked flake files

- **Status:** Open
- **Avoid:** Do not expect `nix develop` to see a file that Git does not track. Do not edit `flake.nix` and leave the edit unstaged.
- **Why:** In a Git checkout, Nix copies the files Git knows about, not the untracked worktree. After `git init`, `flake.nix` is invisible until `git add`. The agent hook then fails closed in practice: every Shell command tries to enter a devshell that cannot be evaluated. Stage or commit flake changes before the next agent command.

### P-0023 Core2 pin map

- **Status:** Open
- **Avoid:** Do not copy a generic M5-Bus I2S map onto Core2 v1.3. Do not treat Grove wire colors as the contract if they have not been buzzed on the HMI connector.
- **Why:** On this board, speaker BCLK is GPIO12 and that pin is not on the M5-Bus. LRCK is GPIO0, speaker data is GPIO2, mic data is GPIO34. GPIO15 is LCD DC. GPIO13 is UART2, which this design reuses as the external LRCK. The assignment that firmware must use is [D-0031](decisions.md#d-0031-board-link).

### P-0024 AAC source

- **Status:** Open
- **Avoid:** Do not treat an AAC encoder, or an AAC A2DP sink, as proof that the Stamp Pico can open an AAC A2DP source session.
- **Why:** SBC and AAC both encode 44.1 kHz and 48 kHz ([D-0032](decisions.md#d-0032-i2s-format)). Checked 2026-10-28: ESP-IDF 6.1 documents `a2dp_source_aac` for the ESP32 target. The application encodes MPEG-2 AAC LC with `esp_audio_codec` and registers that stream endpoint beside a mandatory SBC endpoint. The example does not ask for PSRAM. Espressif's codec table lists about 51 KB of heap for 48 kHz stereo AAC-LC, and about 40 KB of stack to run the encoder set. The Stamp Pico runs Classic Bluetooth and not Wi-Fi, so that heap is internal RAM. A minimal ESP32 image leaves on the order of 290 KB of data RAM; Classic Bluetooth is on the order of 70 KB for the controller in dual mode, and one measured Bluedroid enable took about 120 KB. Releasing the unused BLE controller is part of Espressif's own A2DP source example. Those figures leave room for 51 KB, the frame buffers, and the encoder stack. This tree has not measured the free heap after Bluedroid is up. Postern now links `esp_audio_codec` and registers an AAC endpoint beside SBC. A linked encoder is not a passed source session on this module. SBC is still the session that has to pass. Record the heap numbers here when Postern is first flashed.

### P-0025 Grove 5 V

- **Status:** Open
- **Avoid:** Do not connect the red Grove wire to a GPIO or to the Stamp Pico's 3.3 V pin.
- **Why:** Grove red is 5 V. ESP32 GPIOs are not 5 V tolerant. Signal wires in [D-0031](decisions.md#d-0031-board-link) are 3.3 V. A 5 V feed belongs only on a pin the Stamp Pico regulates, if that pin exists on the carrier.

### P-0026 cspot fork

- **Status:** Open
- **Avoid:** Do not vendor [feelfreelinux/cspot](https://github.com/feelfreelinux/cspot) at its last upstream snapshot. Do not flash [squeezelite-esp32](https://github.com/sle118/squeezelite-esp32) as the Porta firmware. Do not put cspot on the Stamp Pico.
- **Why:** Upstream cspot does not build cleanly and its password login is declined. The fixes people are still playing through live in the [philippe44/cspot](https://github.com/philippe44/cspot) fork used by squeezelite-esp32. That firmware is a Squeezebox player. Porta takes the library and the command pattern only. cspot needs PSRAM. Integrators mark the fork GPL-3.0. Confirm the license file at the pinned commit before linking ([D-0029](decisions.md#d-0029-spotify-transport), [D-0037](decisions.md#d-0037-license)).

### P-0027 Receiver codecs

- **Status:** Open
- **Avoid:** Do not add AirPlay 2. Do not advertise MP3, AAC, FLAC, Opus, Vorbis, WMA, or DSD on the DLNA renderer.
- **Why:** [D-0036](decisions.md#d-0036-airplay-and-dlna) is AirPlay 1 plus LPCM, WAV PCM, and ALAC, all 16-bit stereo at 44.1 or 48 kHz. A longer codec list is a new decision.

### P-0028 Postern file URL

- **Status:** Resolved
- **Avoid:** Do not give the Postern submodule a network URL until a remote exists, and do not replace `../postern` with an absolute path.
- **Why:** The first pin was a local sibling. Cloning it uses the file transport, which this Git allows only when the command sets `protocol.file.allow=always`.
- **Resolution:** The remote `https://github.com/oemaix/postern.git` exists. The submodule tracks branch `main` there ([D-0039](decisions.md#d-0039-firmware-repos)).

### P-0029 pip

- **Status:** Open
- **Avoid:** Do not `pip install` into the devshell, a user site, or a project `.venv`.
- **Why:** The interpreter is the flake's `python3` until [D-0040](decisions.md#d-0040-python) closes. A virtualenv with no packages is a second interpreter. ESP-IDF will bring its own Python later, and that tree stays separate.

### P-0030 No Classic Bluetooth

- **Status:** Open
- **Avoid:** Do not replace the Stamp Pico with an ESP32-S3, C2, C3, C5, C6, C61, H2, or S2 module.
- **Why:** ESP-IDF 6.1 lists Bluetooth Classic only on the original ESP32. The S3 and the C/H parts are Bluetooth LE. A2DP needs Classic Bluetooth. Checked 2026-10-28. A module that keeps Classic Bluetooth and adds PSRAM is an ESP32-WROVER, not an S3 stamp. [D-0007](decisions.md#d-0007-bluetooth-transmitter) stays the Stamp Pico until a measurement says AAC does not fit.

## Resolved

[P-0020](#p-0020-no-toolchain-yet) and [P-0028](#p-0028-postern-file-url) are resolved. The text stays above so the ids are not reused.
