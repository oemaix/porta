# Pitfalls

Status: baseline · 2026-10-30

Open entries are binding. Do not delete an entry. Mark it resolved, with the reason, when it stops applying. New entries take the next id and are added in the same change that discovered them.

Agents read this file before design or implementation work, and they add to it when a new trap shows up. The Cursor rule is only a reminder. This file is the copy that stays in git.

## Open

### P-0001 Spotify on the ESP32

- **Status:** Open
- **Avoid:** Do not implement the Spotify player as librespot, as a browser inside the Core2, as the Web Playback SDK, as the partner embedded SDK, or as the unmodified upstream cspot snapshot.
- **Why:** The Web Playback SDK plays in a browser and does not return PCM. librespot does not run on the ESP32. The embedded SDK is not a public download. The Connect path that does run here is [D-0029](decisions.md#d-0029-spotify-transport). A tone or a plain HTTP stream may stand in until that library is linked. Until then the status title is "Tone stand-in". Do not label the stand-in as Spotify, Connect, or a decoded track.

### P-0002 One radio, two jobs

- **Status:** Open
- **Avoid:** Do not run Wi-Fi station and A2DP source on the Core2.
- **Why:** Classic Bluetooth and Wi-Fi share the radio and coexist poorly at audio rates. [D-0008](decisions.md#d-0008-radio-split) puts transmit on the Stamp Pico for that reason.

### P-0003 No PSRAM

- **Status:** Open
- **Avoid:** Do not treat AAC as the first Bluetooth codec to bring up on the Stamp Pico.
- **Why:** The Stamp Pico is an ESP32-PICO-D4 without PSRAM. An AAC encoder plus A2DP can exhaust RAM that SBC does not. [D-0007](decisions.md#d-0007-bluetooth-transmitter) requires SBC as the fallback that actually ships. Postern records the same ban so that repository stands alone ([Postern P-0002](../postern/docs/pitfalls.md#p-0002-aac-first)). A linked encoder is not a passed session ([P-0024](#p-0024-aac-source)). Measure free heap on this board after SBC is stable. If AAC does not fit, record the numbers here and supersede the codec goal with a new decision.

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
- **Why:** Two masters on one wire is not a link. [D-0031](decisions.md#d-0031-board-link) names the Core2 as I2S master and the Stamp Pico as slave. Share a ground on that cable.

### P-0018 Brownout

- **Status:** Open
- **Avoid:** Do not treat a brownout or a random reboot as unexplained while Wi-Fi transmit and I2S are both running and the AXP192 current limit is still unmeasured. Do not refuse to start those two for that reason.
- **Why:** The combined load can brown out the PMU. The failure looks like a random reboot. It does not damage the Core2. Starting both before the limit is set on purpose and the rail is measured is allowed, and it is not the stable path. When that instability shows up, this is the first cause to check. NFR-04.

### P-0019 Container sandbox

- **Status:** Open
- **Avoid:** Do not copy `sandbox = false` from the devcontainer Nix config onto a host Nix install.
- **Why:** It is there because a single-user Nix inside Docker often cannot use the user-namespace sandbox ([D-0014](decisions.md#d-0014-devcontainer-base)). The host keeps its own sandbox.

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
- **Why:** SBC and AAC both encode 44.1 kHz and 48 kHz ([D-0032](decisions.md#d-0032-i2s-format)). ESP-IDF can register an AAC source endpoint beside a mandatory SBC endpoint. Postern links that encoder. A linked encoder is not a passed source session on this module, and the free heap after the Bluetooth stack is up has not been measured here. SBC is the session that has to pass ([P-0003](#p-0003-no-psram), [D-0007](decisions.md#d-0007-bluetooth-transmitter)). Postern records the same ban ([Postern P-0002](../postern/docs/pitfalls.md#p-0002-aac-first)). Record the heap numbers here when that image is first flashed.

### P-0025 Grove 5 V

- **Status:** Open
- **Avoid:** Do not connect the red Grove wire to a GPIO or to the Stamp Pico's 3.3 V pin.
- **Why:** Grove red is 5 V. ESP32 GPIOs are not 5 V tolerant. Signal wires in [D-0031](decisions.md#d-0031-board-link) are 3.3 V. A 5 V feed belongs only on a pin the Stamp Pico regulates, if that pin exists on the carrier. I2C pull-ups are the Core2 pair to 3.3 V, not this red wire ([Postern cable](../postern/docs/cable.md)).

### P-0026 cspot fork

- **Status:** Open
- **Avoid:** Do not vendor [feelfreelinux/cspot](https://github.com/feelfreelinux/cspot) at its last upstream snapshot. Do not flash [squeezelite-esp32](https://github.com/sle118/squeezelite-esp32) as the Porta firmware. Do not put cspot on the Stamp Pico.
- **Why:** Upstream cspot does not build cleanly and its password login is declined. The fixes people are still playing through live in the [philippe44/cspot](https://github.com/philippe44/cspot) fork used by squeezelite-esp32. That firmware is a Squeezebox player. Porta takes the library and the command pattern only. cspot needs PSRAM. Integrators mark the fork GPL-3.0. Confirm the license file at the pinned commit before linking ([D-0029](decisions.md#d-0029-spotify-transport), [D-0037](decisions.md#d-0037-license)).

### P-0029 pip

- **Status:** Open
- **Avoid:** Do not `pip install` into the devshell, a user site, or a project `.venv`.
- **Why:** [D-0044](decisions.md#d-0044-python-out-of-the-flake). A `.venv`, when one is required, is created with `uv`. That directory must not be the Installation Manager's tree under `~/.espressif`. The flake does not carry `uv` until that day ([P-0044](#p-0044-python-in-the-flake)).

### P-0030 No Classic Bluetooth

- **Status:** Open
- **Avoid:** Do not replace the Stamp Pico with an ESP32-S3, C2, C3, C5, C6, C61, H2, or S2 module.
- **Why:** ESP-IDF 6.1 lists Bluetooth Classic only on the original ESP32. The S3 and the C/H parts are Bluetooth LE. A2DP needs Classic Bluetooth. Checked 2026-10-28. A module that keeps Classic Bluetooth and adds PSRAM is an ESP32-WROVER, not an S3 stamp. [D-0007](decisions.md#d-0007-bluetooth-transmitter) stays the Stamp Pico until a measurement says AAC does not fit. Postern records the same target ban ([Postern P-0010](../postern/docs/pitfalls.md#p-0010-not-an-s3)).

### P-0031 Ubuntu uid 1000

- **Status:** Open
- **Avoid:** Do not create the `vscode` user with `groupadd` or `useradd` at uid 1000.
- **Why:** `ubuntu:26.04` already has user and group `ubuntu` at 1000:1000. `groupadd --gid 1000` exits 4, and the devcontainer image build stops before Nix is installed. `.devcontainer/Dockerfile` renames that account to `vscode` and keeps its groups, including `dialout`.

### P-0032 Container uid

- **Status:** Open
- **Avoid:** Do not set `safe.directory` so `nix develop` will open this checkout. Do not turn `updateRemoteUserUID` on. Do not leave the running `vscode` user at the image uid when the workspace owner inside the container is different.
- **Why:** Rootless Docker maps the host user to container uid 0. The numeric host uid 1000 is a subordinate id inside the container. `updateRemoteUserUID` copies that host number into the image, so `vscode` does not own the workspace, and `postCreateCommand` stops with libgit2 error 7. Files written as that uid show up on the host as an unknown user. `.devcontainer/entrypoint.sh` reads the owner inside the container and moves `vscode` onto it. When that owner is uid 0, `vscode` shares the uid with `root`. `getpwuid(0)` returns `root`, and the session home is `/root`. Do not put the Nix feature flags only in a home `nix.conf`; that file is not what this session reads ([P-0033](#p-0033-host-nix-store)). A `safe.directory` line in `~/.gitconfig` is lost on rebuild and does not fix the written uid.

### P-0033 Host Nix store

- **Status:** Open
- **Avoid:** Do not `chown` `/nix` when it is a bind mount of the host store. Do not mount the host store read-write. Do not turn on `nix-command` or `flakes` by writing `nix.conf` only into the container home or into `/root`. Do not put a newline in `NIX_CONFIG`. Do not take the first profile under `/nix/var/nix/profiles/per-user/`. Do not leave `accept-flake-config` on.
- **Why:** A home `nix.conf` is gone on rebuild. After the uid move the session home is `/root`, so `/home/vscode/.config/nix/nix.conf` is not the file Nix reads. `.devcontainer/nixos-host/devcontainer.json` bind-mounts the host `/nix` read-only and sets `NIX_CONFIG` to one line, `experimental-features = nix-command flakes`. A newline in `containerEnv` arrives as the two characters `\` and `n`, Nix leaves flakes disabled, and `.devcontainer/nix.sh` rewrites that sequence. The image file `/etc/nix/nix.conf` carries the same features, `accept-flake-config = false`, and `sandbox = false` for every home. `accept-flake-config = false` stops a flake from adding substituters. `.devcontainer/nix.sh` uses the host daemon when its socket is present, so this container does not write store paths itself. It selects the system profile, then the default profile, then the current user's profile. `.devcontainer/post-create.sh` preserves `NIX_CONFIG` across its sudo re-exec. The entrypoint leaves a mounted `/nix` alone.

### P-0034 Zsh exit status

- **Status:** Open
- **Avoid:** Do not start the interactive devshell zsh with no startup file.
- **Why:** Nix's zsh loads `zsh/newuser` when `.zshrc` is missing. As uid 0 that installer returns 1, so a following `exit` closes the terminal with status 1. The default prompt is `%m%#`, the container hostname and `#`. Each devshell sets `ZDOTDIR` to `shell/`, whose `.zshrc` shows the working directory and ends successfully. A NixOS host still sources `/etc/zshrc` after that. That file installs `command_not_found_handler`, and the handler returns 127. The devcontainer image does not have this rc. `shell/.zshenv` turns global rc files off, so a following `exit` is not that status. `/etc/zshenv` still runs. The devshell also sets `SHELL` to its own zsh. Nix's setup would otherwise leave `SHELL` as the non-interactive bash from the build environment.

### P-0035 One devcontainer for every host

- **Status:** Open
- **Avoid:** Do not bind-mount the host `/nix` from **Porta (Nix on Ubuntu)**. Do not send a host that has no `/nix` through **Porta (NixOS host store)**.
- **Why:** The mount hides the Nix store installed in the image. On a host with no `/nix`, Docker creates an empty directory and the image `nix` is gone. The Ubuntu config is `.devcontainer/devcontainer.json`. The NixOS config is `.devcontainer/nixos-host/devcontainer.json` ([D-0041](decisions.md#d-0041-devcontainer-host)).

### P-0036 Floating Nix installer

- **Status:** Open
- **Avoid:** Do not install Nix from `https://nixos.org/nix/install`.
- **Why:** That URL is the current installer. A later image rebuild can change the Nix version. `.devcontainer/Dockerfile` pins `https://releases.nixos.org/nix/nix-2.35.4/install` ([D-0014](decisions.md#d-0014-devcontainer-base)).

### P-0037 Checkout directory name

- **Status:** Open
- **Avoid:** Do not hardcode `/workspaces/porta` in the entrypoint or the image `WORKDIR`.
- **Why:** Dev Containers mounts the checkout at `/workspaces/` plus the directory name. A clone under another name leaves `vscode` on the image uid, and `postCreateCommand` stops with libgit2 error 7 ([P-0032](#p-0032-container-uid)). Both configs set `DEVCONTAINER_WORKSPACE` to `${containerWorkspaceFolder}`. The entrypoint reads that variable. The image `WORKDIR` is `/workspaces`.

### P-0038 Panel calls the player

- **Status:** Open
- **Avoid:** Do not call cspot, the ES8388, I2S, or the Wi-Fi driver from `input` or `ui`. Do not put PCM in the status snapshot. Do not add a second process, a socket, or a widget toolkit as the seam between the panel and the player.
- **Why:** [D-0042](decisions.md#d-0042-core2-components). The Core2 image is one ESP-IDF application. The seam is player commands and one snapshot ([software](software.md)). PCM stays on the path from `player` to `sink`. A toolkit or a second process spends the RAM the session needs and gives the screens another way to reach the codec.

### P-0040 Unmeasured rail

- **Status:** Open
- **Avoid:** Do not forbid Wi-Fi or Module Audio I2S because the AXP192 current limit is not measured yet. Do not write a guessed milliamp limit into the tree and call it the measurement. Do not enable the NS4168 from a generic ES8388 init.
- **Why:** Starting Wi-Fi and I2S together before that measurement is allowed and not recommended ([P-0018](#p-0018-brownout)). A brownout or a random reboot is suspected here first. The bench supply is not itself the measured limit ([D-0043](decisions.md#d-0043-firmware-tree)). `Board::limit_measured` stays false until the number is written down, and that flag does not block the start. The internal codec shares the Module Audio pins ([P-0006](#p-0006-internal-i2s)).

### P-0041 EIM home install

- **Status:** Open
- **Avoid:** Do not commit `~/.espressif` or `~/.cache/espidf-nix`. Do not expect the Nix store to contain the Xtensa toolchain. Do not point an ESP-IDF install at an unpinned tag.
- **Why:** Chip tools land under `~/.espressif`, not in the Nix store. The Porta devcontainer bind-mounts the login user's tree onto `/root/.espressif` ([D-0014](decisions.md#d-0014-devcontainer-base), [P-0032](#p-0032-container-uid)). Postern's shell is what currently fills that tree with v6.1, the first time its `idf.py` runs. Postern records that install on its own ([Postern P-0031](../postern/docs/pitfalls.md#p-0031-install-home)). A later Core2 image install of v5.5.6 uses the same home, and one tree must not be asked to satisfy two IDF versions without a recorded layout. In this image the path is `/root/.espressif`, because the session runs as root. On the host it is the login user's `~/.espressif`. `~/.cache/espidf-nix` is left in the container layer and is written again when an activation script is newer. Reloading the IDE keeps the container. Rebuilding it drops the writable layer. Copy `/root/.espressif` to the login user's home before the rebuild that adds the bind, or the new container hides the copy that exists only in the old layer.

### P-0042 FHS namespace

- **Status:** Open
- **Avoid:** Do not add [hnaderi/espidf-nix](https://github.com/hnaderi/espidf-nix) back to the Porta flake. Do not treat `SYS_ADMIN` and `seccomp=unconfined` on the Porta devcontainer as a Core2 build input. Do not remove those flags on the grounds that the Core2 crate calls `idf.py`. Do not replace a shell that still runs `idf.py` with an ESP-IDF flake pinned to an older nixpkgs.
- **Why:** [D-0046](decisions.md#d-0046-core2-language) removed `idf.py` from the Porta shell. `cargo test` does not need a mount namespace. The flags and the `~/.espressif` bind stay so this container can still run a shell that does, including Postern's `idf.py`. That command runs inside `buildFHSEnv` and needs a new mount namespace. A container that forbids it stops at `bwrap: Creating new namespace failed`. bubblewrap 0.12 uses `clone3`, which Docker's default seccomp profile rejects ([D-0014](decisions.md#d-0014-devcontainer-base)). A running container does not gain the flags until it is rebuilt. Postern records the same namespace trap for its own container ([Postern P-0032](../postern/docs/pitfalls.md#p-0032-fhs-namespace)). Dropping the Porta flags is a separate change.

### P-0043 Serial device node

- **Status:** Open
- **Avoid:** Do not `mknod` the serial node inside the container. Do not mount the host `/dev` on the container `/dev`. Do not bind one host device inode with `--device`. Do not make the host nodes world-writable. Do not name a baud rate or a flash offset in `devcontainer.json`. Do not treat a `uaccess` tag on the parent USB device as access to the tty node. Do not link only `ttyUSB0` and `ttyACM0`. Do not open the host tty from the entrypoint scan.
- **Why:** This bench uses `/dev/ttyUSB0` (major 188) and `/dev/ttyACM0` (major 166). The Core2 is `/dev/ttyACM0`. Those majors cover every `ttyUSBn` and `ttyACMn`. Rootless Docker maps container uid 0 to the host user. `mknod` in that user namespace returns EPERM, with or without `CAP_MKNOD`, so an entrypoint that ignores the error leaves the path missing. A `--device` bind is one host inode. Unplugging the cable deletes it, and the container path stays gone. Both configs bind the host `/dev` at `/host-dev` ([D-0014](decisions.md#d-0014-devcontainer-base)). That directory is the live host devtmpfs. The entrypoint links each `ttyUSB*` and `ttyACM*` node into `/dev` and repeats the scan while the container runs, so a cable plugged in after the IDE starts appears, and an unplug removes the link. Opening the node from that scan toggles modem lines and resets the board. `idf.py` bind-mounts `/dev` and `/host-dev` when it starts, and the link resolves in that sandbox. The device cgroup allows majors 188 and 166, which is what keeps the rest of `/host-dev` from being opened. Host group `dialout` is not a gid in this map, and the node stays owned by host root, so mode `0660` is not readable here. `.devcontainer/nixos-host/serial.rules` sets that mode, group `dialout`, and `TAG+="uaccess"` on every such node. Logind then grants the session user, which is container uid 0. A tag on the parent USB device does not do that. NixOS imports `.devcontainer/nixos-host/serial-udev.nix`. The entrypoint is copied into the image, and `/host-dev` is a container mount, so a running container stays on the old scan until it is rebuilt.

### P-0044 Python in the flake

- **Status:** Open
- **Avoid:** Do not add `python3`, `python314`, `uv`, or `pip` to the Porta flake merely because a hook exists. Do not `pip install` for the image build.
- **Why:** [D-0044](decisions.md#d-0044-python-out-of-the-flake). The hooks are standard-library scripts started by the host `python3`. The Python inside an ESP-IDF install, and a flake package added because the Core2 image target needs it, are not this ban. Postern records the same policy for its flake ([Postern P-0028](../postern/docs/pitfalls.md#p-0028-python-in-the-flake)). `pip` stays out ([P-0029](#p-0029-pip)). A project `.venv` is created with `uv` on the day a Porta script imports a third-party package.

### P-0045 ESP-IDF install home

- **Status:** Open
- **Avoid:** Do not expect one `~/.espressif` tree to work under a different home. Do not rewrite an activation script to one absolute path. Do not drop the host bind because a command already worked in the container.
- **Why:** An ESP-IDF installer writes absolute paths into its activation script and virtualenv shebangs. Postern's v6.1 install does this, and Postern records the procedure ([Postern P-0031](../postern/docs/pitfalls.md#p-0031-install-home)). In the Porta devcontainer that home is `/root` ([P-0032](#p-0032-container-uid)). The bind stores those files in the login user's `~/.espressif` ([P-0041](#p-0041-eim-home-install)). On the host the `/root` paths are not that tree, so a wrapper that expects them fails to set up the version. A snapshot taken while those paths are missing caches an empty version, and a later run keeps the empty value. When the user's tree exists and `/root/.espressif` does not, the wrapper bind-mounts the tree at `/root/.espressif` for that command. The container already has `/root/.espressif`, so the wrapper runs the real command there. A later v5.5.6 tree for the Core2 hits the same home split.

### P-0046 LCD rail and the microSD slot

- **Status:** Open
- **Avoid:** Do not mount the microSD card because the panel rail is on. Do not turn that rail off to keep the slot dark. Do not drive GPIO4 low while the panel is using the shared SPI bus.
- **Why:** ILI9342C logic power is AXP192 LDO2, and that rail also feeds the microSD slot. The backlight is DCDC3. LCD reset is AXP GPIO4. ESP32 GPIO4 is the card chip-select, a different pin ([architecture](architecture.md)). Phase 1 does not mount the card ([D-0025](decisions.md#d-0025-microsd-in-phase-1)). The panel code holds GPIO4 high so the slot ignores the bus.

### P-0048 Core2 image target

- **Status:** Open
- **Avoid:** Do not treat a passing `cargo test` as a flashed Core2 image.
- **Why:** [D-0046](decisions.md#d-0046-core2-language). `cargo test` covers the panel, the commands, and the tone. The framework ban — not `idf.py`, not ESP-IDF v6.1, not an esp-hal image — is that decision. The Xtensa Rust target and the v5.5.6 tree are not in this shell yet. That gap is [roadmap](roadmap.md) status. This entry does not forbid adding them.

## Resolved

### P-0009 Unbound HMI button

- **Status:** Resolved
- **Avoid:** Superseded by [D-0026](decisions.md#d-0026-second-hmi-button). The old ban, "do not give button index 3 a command", made an open recommendation binding. That no longer applies.
- **Resolution:** [D-0026](decisions.md#d-0026-second-hmi-button) stays open. The software reports the button and does not act on it. A prototype that binds it says in the title or the commit that the binding is not that decision.

### P-0020 No toolchain yet

- **Status:** Resolved
- **Avoid:** Superseded for the Core2 by [D-0046](decisions.md#d-0046-core2-language), and for the Stamp Pico by [Postern D-0022](../postern/docs/decisions.md#d-0022-idf-in-the-shell). The old note that the Porta shell installs v6.1 through EIM no longer applies.
- **Resolution:** [D-0028](decisions.md#d-0028-firmware-framework) selected ESP-IDF. Arduino and PlatformIO stay out. [D-0046](decisions.md#d-0046-core2-language) removed `idf.py` from the Porta shell. The Core2 image is Rust on ESP-IDF v5.5.6. An ESP-IDF Nix flake pinned to an older nixpkgs stays out of either shell. Postern's shell installs v6.1 through EIM and records that ban itself ([Postern P-0006](../postern/docs/pitfalls.md#p-0006-idf-flake)).

### P-0021 Blackberry pixels

- **Status:** Resolved
- **Avoid:** Superseded by [D-0011](decisions.md#d-0011-visual-language). This entry added no mechanism beyond that decision.
- **Resolution:** The screen is color blocks. It does not imitate a Blackberry 7290 or 8100. Those phones are the reference for the wheel grammar only.

### P-0027 Receiver codecs

- **Status:** Resolved
- **Avoid:** Superseded by [D-0036](decisions.md#d-0036-airplay-and-dlna).
- **Resolution:** [D-0036](decisions.md#d-0036-airplay-and-dlna) is AirPlay 1 plus LPCM, WAV PCM, and ALAC. It already excludes AirPlay 2, MP3, AAC, FLAC, Opus, Vorbis, WMA, and DSD. A longer codec list is a new decision.

### P-0028 Postern file URL

- **Status:** Resolved
- **Avoid:** Superseded by [D-0039](decisions.md#d-0039-firmware-repos). The old ban, "do not give the submodule a network URL until a remote exists", no longer applies.
- **Resolution:** The remote `https://github.com/oemaix/postern.git` exists. The submodule tracks branch `main` there. Do not replace that URL with an absolute local path.

### P-0039 Tone stand-in

- **Status:** Resolved
- **Avoid:** Superseded by [P-0001](#p-0001-spotify-on-the-esp32).
- **Resolution:** The status title stays "Tone stand-in" until cspot is linked. That sentence now lives in [P-0001](#p-0001-spotify-on-the-esp32).

### P-0047 Postern image

- **Status:** Resolved
- **Avoid:** Superseded by [D-0045](decisions.md#d-0045-postern-image). The old second sentence, "do not run `idf.py` inside `postern/components/postern`", is [Postern P-0030](../postern/docs/pitfalls.md#p-0030-reuse-directory) and stays in that repository.
- **Resolution:** Do not add a Stamp Pico application in this repository. [D-0045](decisions.md#d-0045-postern-image) is that rule, stated here so a Porta checkout does not depend on opening Postern to know it. Postern's one-image rule is [Postern P-0034](../postern/docs/pitfalls.md#p-0034-one-image).
