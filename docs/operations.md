# Operations

Status: baseline · 2026-10-28

Phase 0 has a development shell and an editor setup. The framework is ESP-IDF ([D-0028](decisions.md#d-0028-firmware-framework)). The shell provides `esptool` and does not yet provide `idf.py` or the Xtensa toolchain. It does not flash a board.

## Host

Development hosts are Linux ([D-0022](decisions.md#d-0022-development-host)). Nix with flakes enabled is required. The shell is `devShells.default` in `flake.nix`, pinned to nixpkgs `nixos-26.05` ([D-0015](decisions.md#d-0015-nixpkgs)).

```sh
nix develop
```

Nix evaluates a Git checkout from the files Git tracks. After `git init`, stage `flake.nix` and `flake.lock` (or commit them) before the first `nix develop`. An unstaged edit of `flake.nix` is invisible to the next shell. See [P-0022](pitfalls.md#p-0022-untracked-flake-files).

The shell replaces `PATH`. Git, Python 3, zsh, bash, CMake, Ninja, and `esptool` are in the shell because agent commands and the terminal run inside it ([P-0013](pitfalls.md#p-0013-devshell-path)). There is no login banner.

The first run downloads the nixpkgs pin. Commit `flake.lock` when the repository starts taking commits, so later shells hit the same pin.

## Postern

`postern/` is a submodule of `https://github.com/oemaix/postern.git`, branch `main` ([D-0039](decisions.md#d-0039-firmware-repos)).

## Devcontainer

The container is optional ([D-0013](decisions.md#d-0013-build-entry)). `.devcontainer/Dockerfile` uses Ubuntu 26.04 LTS, creates a `vscode` user, and installs official single-user Nix with flakes enabled. The Nix sandbox is off inside that container only ([P-0019](pitfalls.md#p-0019-container-sandbox)).

Reopen the folder in the devcontainer. `postCreateCommand` evaluates `nix develop --command true` once, which fills the Nix store. After that, the terminal profile below is the shell people use.

## Terminal

`.vscode/nix-develop-terminal.sh` changes to the repository root and runs `nix develop -c zsh` when the host has zsh, otherwise `nix develop -c bash` ([D-0019](decisions.md#d-0019-terminal-shell)). `.vscode/settings.json` selects that profile as the default on Linux and macOS.

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

`.cursor/` is gitignored ([D-0017](decisions.md#d-0017-gitignore)). Cursor users on a fresh clone do not get the hook until it is copied back. The copy to install is the script `nix-develop.py` from the `bootstrap` skill, at `.cursor/hooks/nix-develop.py`, plus `.cursor/hooks.json`.

The hook is a `preToolUse` filter on the Shell tool. It rewrites a command so that it runs in `nix develop` unless `IN_NIX_SHELL` is already set or the command is already `nix develop` ([D-0018](decisions.md#d-0018-agent-shell), [P-0014](pitfalls.md#p-0014-hook-reentry)). `beforeShellExecution` cannot change the command, so the hook is not attached there.

Restore check:

```sh
printf '%s' '{"tool_name":"Shell","tool_input":{"command":"echo hi","working_directory":"."}}' \
  | .cursor/hooks/nix-develop.py
```

The printed command contains `IN_NIX_SHELL` and `nix develop -c`. A command that already starts with `nix develop` is left unchanged.

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
