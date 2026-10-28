# Operations

Status: baseline · 2026-10-28

Phase 0 has a development shell and an editor setup. The framework is ESP-IDF ([D-0028](decisions.md#d-0028-firmware-framework)). The shell provides `esptool` and does not yet provide `idf.py` or the Xtensa toolchain. It does not flash a board.

## Host

Development hosts are Linux ([D-0022](decisions.md#d-0022-development-host)). Nix with flakes enabled is required. The shell is `devShells.default` in `flake.nix`, pinned to nixpkgs `nixos-26.05` ([D-0015](decisions.md#d-0015-nixpkgs)).

```sh
nix develop
```

Nix evaluates a Git checkout from the files Git tracks. After `git init`, stage `flake.nix` and `flake.lock` (or commit them) before the first `nix develop`. An unstaged edit of `flake.nix` is invisible to the next shell. See [P-0022](pitfalls.md#p-0022-untracked-flake-files).

The shell replaces `PATH`. Git, Python 3, zsh, bash, CMake, Ninja, and `esptool` are in the shell because agent commands and the terminal run inside it ([P-0013](pitfalls.md#p-0013-devshell-path)). There is no login banner. Python is the flake's `python3` and the standard library. There is no `.venv` yet ([D-0040](decisions.md#d-0040-python), [P-0029](pitfalls.md#p-0029-pip)).

The first run downloads the nixpkgs pin. Commit `flake.lock` when the repository starts taking commits, so later shells hit the same pin.

## Postern

`postern/` is a submodule of `https://github.com/oemaix/postern.git`, branch `main` ([D-0039](decisions.md#d-0039-firmware-repos)).

## Devcontainer

The container is optional ([D-0013](decisions.md#d-0013-build-entry)). `.devcontainer/Dockerfile` uses Ubuntu 26.04 LTS and installs official single-user Nix 2.35.4, with flakes enabled, for the `vscode` user ([P-0036](pitfalls.md#p-0036-floating-nix-installer)). That user is the image's existing uid 1000 account, renamed ([P-0031](pitfalls.md#p-0031-ubuntu-uid-1000)). On start, `.devcontainer/entrypoint.sh` moves that account onto the workspace owner as the owner appears inside the container. The path comes from `DEVCONTAINER_WORKSPACE` ([P-0037](pitfalls.md#p-0037-checkout-directory-name)), and `updateRemoteUserUID` stays off ([P-0032](pitfalls.md#p-0032-container-uid)). Rootless Docker shows the host user as uid 0. `nix-command` and `flakes` are one line of `NIX_CONFIG`. The image file `/etc/nix/nix.conf` carries the same features, `accept-flake-config = false`, and `sandbox = false` ([P-0033](pitfalls.md#p-0033-host-nix-store)). The Nix sandbox is off inside that container only ([P-0019](pitfalls.md#p-0019-container-sandbox)).

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
| `ms-python.python` | The agent hook and host scripts |

The Espressif VS Code extension is not required. It installs a second ESP-IDF, and this shell does not provide `idf.py` yet.

## Agent shell hook

`.cursor/` is gitignored ([D-0017](decisions.md#d-0017-gitignore)). Cursor users on a fresh clone do not get the hook until it is copied back. The copy to install is `.cursor/hooks/nix-develop.py` and `.cursor/hooks/nix-develop-postern.py`, plus `.cursor/hooks.json`. The Porta script started from `nix-develop.py` in the `bootstrap` skill. The Postern script is local to this checkout.

The hooks are `preToolUse` filters on the Shell tool. `.cursor/hooks/nix-develop-postern.py` rewrites a command when its working directory is inside `postern/`, and runs it in that flake. `.cursor/hooks/nix-develop.py` does the same selection for every command. A command is left unchanged when `IN_NIX_SHELL` is already set or it already contains `nix develop -c` ([D-0018](decisions.md#d-0018-agent-shell), [P-0014](pitfalls.md#p-0014-hook-reentry)). `beforeShellExecution` cannot change the command, so the hooks are not attached there.

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
- ESP-IDF is the framework. Do not add Arduino or PlatformIO. Do not add an application tree until that work starts.

The rules are reminders. The tracked text is this suite.

## What this phase does not do

- It does not select a serial port, a baud rate, or a flash offset.
- It does not build an artifact.
- It does not create an application directory.
