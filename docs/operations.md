# Operations

Status: baseline · 2026-10-29

The framework is ESP-IDF v6.1 ([D-0028](decisions.md#d-0028-firmware-framework)). The shell provides `idf.py` through [hnaderi/espidf-nix](https://github.com/hnaderi/espidf-nix), plus `esptool` and a host `gcc`. The chip toolchain is downloaded into `~/.espressif` on the first `idf.py` run ([P-0041](pitfalls.md#p-0041-eim-home-install)). The shell does not choose a baud rate or a flash offset. The devcontainer links every host `ttyUSB*` and `ttyACM*` node into `/dev` while it runs, including a cable plugged in after the IDE starts ([P-0043](pitfalls.md#p-0043-serial-device-node)). The Core2 on this bench is `/dev/ttyACM0`. `~/.espressif` in this image is `/root/.espressif`. The devcontainer bind-mounts the login user's `~/.espressif` on the host there ([P-0041](pitfalls.md#p-0041-eim-home-install)). The Core2 tree is `firmware/` ([D-0043](decisions.md#d-0043-firmware-tree)).

## Host

Development hosts are Linux ([D-0022](decisions.md#d-0022-development-host)). Nix with flakes enabled is required. The shell is `devShells.default` in `flake.nix`, pinned to nixpkgs `nixos-26.05` ([D-0015](decisions.md#d-0015-nixpkgs)).

```sh
nix develop
```

Nix evaluates a Git checkout from the files Git tracks. After `git init`, stage `flake.nix` and `flake.lock` (or commit them) before the first `nix develop`. An unstaged edit of `flake.nix` is invisible to the next shell. See [P-0022](pitfalls.md#p-0022-untracked-flake-files).

The shell replaces `PATH`. Git, zsh, bash, CMake, Ninja, host `gcc`, `esptool`, and `idf.py` are in the shell because agent commands and the terminal run inside it ([P-0013](pitfalls.md#p-0013-devshell-path)). There is no login banner. The flake does not provide Python ([D-0044](decisions.md#d-0044-python-out-of-the-flake), [P-0044](pitfalls.md#p-0044-python-in-the-flake)). There is no `.venv`. A later third-party import gets a gitignored `.venv` from `uv`, and `pip` stays unused ([P-0029](pitfalls.md#p-0029-pip)). `gcc` is the host compiler for the firmware logic test. `idf.py` is the ESP-IDF v6.1 wrapper. Its Python is 3.13 from this nixpkgs pin, inside the Installation Manager's own environment.

The first run downloads the nixpkgs pin. Commit `flake.lock` when the repository starts taking commits, so later shells hit the same pin.

## Core2 host test

`firmware/` is the Core2 application. This check compiles the panel, the player commands, and the tone PCM with host `gcc`. It does not call `idf.py`. The binary goes under `tmp/`, which is scratch.

```sh
gcc -std=c11 -Wall -Wextra -Werror \
  -I firmware/components/seam/include \
  -I firmware/components/input/include \
  -I firmware/components/ui/include \
  -I firmware/components/player/include \
  -I firmware/components/sink/include \
  -I firmware/components/net/include \
  -I firmware/components/board/include \
  -I firmware/components/app/include \
  firmware/components/seam/porta_seam.c \
  firmware/components/input/porta_input.c \
  firmware/components/ui/porta_ui.c \
  firmware/components/player/porta_player.c \
  firmware/components/sink/porta_sink.c \
  firmware/components/net/porta_net.c \
  firmware/components/board/porta_board.c \
  firmware/components/app/porta_app.c \
  firmware/tests/porta_phase1_test.c \
  -o tmp/porta-phase1-test
tmp/porta-phase1-test
```

Stage `flake.nix` and `flake.lock` before the next `nix develop` if either file changed ([P-0022](pitfalls.md#p-0022-untracked-flake-files)).

## Core2 image

From this shell, in `firmware/`:

```sh
idf.py set-target esp32
idf.py build
```

The first command installs ESP-IDF v6.1 for `esp32` under `~/.espressif`. Later commands reuse that install. `idf.py` enters that install through bubblewrap and needs a mount namespace ([P-0042](pitfalls.md#p-0042-fhs-namespace)). Flash with `idf.py -p` and the port chosen at that moment. This page does not name a port, a baud rate, or an offset.

## Postern

`postern/` is a submodule of `https://github.com/oemaix/postern.git`, branch `main` ([D-0039](decisions.md#d-0039-firmware-repos)).

## Devcontainer

The container is optional ([D-0013](decisions.md#d-0013-build-entry)). `.devcontainer/Dockerfile` uses Ubuntu 26.04 LTS and installs official single-user Nix 2.35.4, with flakes enabled, for the `vscode` user ([P-0036](pitfalls.md#p-0036-floating-nix-installer)). That user is the image's existing uid 1000 account, renamed ([P-0031](pitfalls.md#p-0031-ubuntu-uid-1000)). On start, `.devcontainer/entrypoint.sh` moves that account onto the workspace owner as the owner appears inside the container. The path comes from `DEVCONTAINER_WORKSPACE` ([P-0037](pitfalls.md#p-0037-checkout-directory-name)), and `updateRemoteUserUID` stays off ([P-0032](pitfalls.md#p-0032-container-uid)). Rootless Docker shows the host user as uid 0. `nix-command` and `flakes` are one line of `NIX_CONFIG`. The image file `/etc/nix/nix.conf` carries the same features, `accept-flake-config = false`, and `sandbox = false` ([P-0033](pitfalls.md#p-0033-host-nix-store)). The Nix sandbox is off inside that container only ([P-0019](pitfalls.md#p-0019-container-sandbox)). Both configs add `SYS_ADMIN` and `seccomp=unconfined` so `idf.py` can create its mount namespace ([P-0042](pitfalls.md#p-0042-fhs-namespace)). Both bind the host `/dev` at `/host-dev`. The entrypoint links every `ttyUSB*` and `ttyACM*` node there, including a cable plugged in after the IDE starts ([P-0043](pitfalls.md#p-0043-serial-device-node)). The host tty rule is `.devcontainer/nixos-host/serial.rules`: mode `0660`, group `dialout`, and tag `uaccess`. NixOS imports `.devcontainer/nixos-host/serial-udev.nix` from the host configuration. After that system rebuild, replug the cable so logind grants the session user. That user is uid 0 in this container. A non-NixOS host installs the same rule under `/etc/udev/rules.d/`. Both configs bind the login user's `~/.espressif` on the host onto `/root/.espressif`. Reloading the IDE keeps the running container and its writable layer. Rebuilding the container starts a new layer. Copy `/root/.espressif` to that user's home before that rebuild, or the bind hides the copy that exists only in the old layer. A rebuild is required before a running container sees `SYS_ADMIN`, `/host-dev`, and the entrypoint link. The Dockerfile is unchanged.

Reopen in Container asks which config to use ([D-0041](decisions.md#d-0041-devcontainer-host)).

| Config | Who |
| --- | --- |
| **Porta (Nix on Ubuntu)** | A host without NixOS. The Nix store stays in the image. |
| **Porta (NixOS host store)** | A NixOS host. The container bind-mounts that host's `/nix` read-only and uses its daemon, so store paths survive a rebuild ([P-0033](pitfalls.md#p-0033-host-nix-store)). |

`postCreateCommand` runs `.devcontainer/post-create.sh`, which waits for the uid change, keeps `NIX_CONFIG` if it must re-exec as `vscode`, and then evaluates `nix develop --command true` once. That fills whichever store the config selected. After that, the terminal profile below is the shell people use. A rebuild is required after an entrypoint or mount change. The entrypoint and the Nix client are copied into the image.

## Terminal

`.vscode/settings.json` defines the profile **Nix Develop** as `nix develop` for this repo with `--command zsh`, and selects it as the default on Linux and macOS ([D-0019](decisions.md#d-0019-terminal-shell)). The profile **Postern** is the same command for the Postern flake. zsh is a devshell package. The image does not install it. The prompt is the working directory, then `#` as root and `%` otherwise ([P-0034](pitfalls.md#p-0034-zsh-exit-status)). Those profiles are the terminal menu on the host and in the container. The image puts `/usr/local/bin` first on `PATH`, so `nix` there is the client copied into the image. The two `devcontainer.json` files do not repeat the profiles. Open them from the terminal profile menu. They are not top-level commands. The extension list in both configs is the same list as `.vscode/extensions.json`.

## Required extensions

These are the extensions a workspace should install ([D-0024](decisions.md#d-0024-editor-extensions)). The same list is in `.vscode/extensions.json` and in the devcontainer.

| Extension | Why it is required |
| --- | --- |
| `jnoortheen.nix-ide` | Edit `flake.nix` |
| `editorconfig.editorconfig` | Apply `.editorconfig` |
| `DavidAnson.vscode-markdownlint` | Keep the doc suite consistent |
| `yzhang.markdown-all-in-one` | Edit the doc suite |
| `ms-vscode.cpptools` | ESP-IDF firmware is C |
| `ms-vscode.vscode-serial-monitor` | Board bring-up logs |
| `ms-python.python` | The agent hook. The flake does not provide its interpreter ([D-0044](decisions.md#d-0044-python-out-of-the-flake)) |

The Espressif VS Code extension is not required. The shell provides `idf.py`. That extension would install a second copy ([D-0024](decisions.md#d-0024-editor-extensions)).

## Agent shell hook

`.cursor/` is gitignored ([D-0017](decisions.md#d-0017-gitignore)). Cursor users on a fresh clone do not get the hook until it is copied back. The copy to install is `.cursor/hooks/nix-develop.py` and `.cursor/hooks/nix-develop-postern.py`, plus `.cursor/hooks.json`. The Porta script started from `nix-develop.py` in the `bootstrap` skill. The Postern script is local to this checkout.

The hooks are `preToolUse` filters on the Shell tool. They are standard-library Python. The process that launches them supplies `python3`. That interpreter is not a flake package ([D-0044](decisions.md#d-0044-python-out-of-the-flake)). `.cursor/hooks/nix-develop-postern.py` rewrites a command when its working directory is inside `postern/`, and runs it in that flake. `.cursor/hooks/nix-develop.py` does the same selection for every command. A command is left unchanged when `IN_NIX_SHELL` is already set or it already contains `nix develop -c` ([D-0018](decisions.md#d-0018-agent-shell), [P-0014](pitfalls.md#p-0014-hook-reentry)). `beforeShellExecution` cannot change the command, so the hooks are not attached there.

Restore check:

```sh
printf '%s' '{"tool_name":"Shell","tool_input":{"command":"echo hi","working_directory":"."}}' \
  | .cursor/hooks/nix-develop.py
printf '%s' '{"tool_name":"Shell","tool_input":{"command":"echo hi","working_directory":"postern"}}' \
  | .cursor/hooks/nix-develop-postern.py
```

The first printed command contains `IN_NIX_SHELL` and `nix develop -c`. The second names the `postern` flake. A command that already contains `nix develop -c` is left unchanged. The Postern script leaves a Porta working directory unchanged.

## Cursor rules

Three local rules apply in every session:

- Read and extend [pitfalls](pitfalls.md).
- Update `.cursor/rules/` when a convention will be needed again.
- ESP-IDF is the framework. Do not add Arduino or PlatformIO. The Core2 application is `firmware/` ([D-0043](decisions.md#d-0043-firmware-tree)).

The rules are reminders. The tracked text is this suite.

## What this phase does not do

- It does not select a baud rate or a flash offset. The container nodes follow the host `ttyUSB*` and `ttyACM*` names. The Core2 on this bench is `/dev/ttyACM0`.
- It does not flash a board by itself. `idf.py build` in `firmware/` is the image build, where a mount namespace is allowed.
- The host test does not need the chip toolchain.
