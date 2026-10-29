{
  description = "Porta development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
  # Official EIM binaries, built with the pkgs passed in. Follow this pin
  # so the wrapper does not pull nixos-unstable (D-0028).
  inputs.espidf.url = "github:hnaderi/espidf-nix";
  inputs.espidf.inputs.nixpkgs.follows = "nixpkgs";

  outputs =
    { nixpkgs, espidf, ... }:
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
            # ESP-IDF v6.1 comes from hnaderi/espidf-nix (D-0028).
            # That wrapper uses Python 3.13 from this nixpkgs pin. EIM
            # installs the chip toolchain into ~/.espressif on the first
            # idf.py run. This flake does not carry a project Python (D-0044).
            # shell/.zshrc is the interactive prompt and keeps `exit` at 0 (P-0034).
            shellHook = ''
              zdotdir=${./shell}
              if [ -n "''${ZDOTDIR:-}" ] && [ -f "''${ZDOTDIR}/.zshrc" ] && [ "''${ZDOTDIR}" != "$zdotdir" ]; then
                export DEV_SAVED_ZDOTDIR="''${ZDOTDIR}"
              fi
              export ZDOTDIR="$zdotdir"
            '';
            packages =
              (with pkgs; [
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
              ])
              ++ [
                (espidf.lib.mkEspIdf {
                  inherit pkgs;
                  idfVersion = "v6.1";
                  idfTargets = "esp32";
                })
              ];
          };
        }
      );
    };
}
