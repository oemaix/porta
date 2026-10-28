#!/bin/sh
# Installed path: .vscode/nix-develop-postern-terminal.sh
# Interactive terminal inside the Postern flake.
# Prefer zsh when the host has it; otherwise bash.
set -eu
root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$root/postern"
if command -v zsh >/dev/null 2>&1 || [ -x /usr/bin/zsh ] || [ -x /bin/zsh ]; then
  exec nix develop -c zsh
fi
exec nix develop -c bash
