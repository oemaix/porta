{
  description = "Porta development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      eachSystem = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = eachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            # The Core2 crate is Rust on ESP-IDF v5.5.6 (D-0046). This shell
            # provides cargo for the host test. It does not provide idf.py.
            # The Stamp Pico image is the Postern submodule and has its own
            # shell. shell/.zshenv skips the host global zshrc, and
            # shell/.zshrc is the interactive prompt (P-0034).
            shellHook = ''
              export SHELL=${pkgs.zsh}/bin/zsh
              zdotdir=${./shell}
              if [ -n "''${ZDOTDIR:-}" ] && [ -f "''${ZDOTDIR}/.zshrc" ] && [ "''${ZDOTDIR}" != "$zdotdir" ]; then
                export DEV_SAVED_ZDOTDIR="''${ZDOTDIR}"
              fi
              export ZDOTDIR="$zdotdir"
            '';
            packages = with pkgs; [
              bashInteractive
              zsh
              git
              curl
              cacert
              coreutils
              findutils
              gnugrep
              gnused
              gawk
              which
              cmake
              ninja
              gcc
              rustc
              cargo
              rustfmt
              clippy
            ];
          };
        }
      );
    };
}
