# esp-rs rustc 1.95.0.0, the same Rust release as nixpkgs rustc 1.95.0.
# The nixpkgs compiler lists xtensa-esp32-espidf and cannot emit it (P-0049).
# This tarball is the rustup installer, not a sysroot. rust-src is separate
# because the Xtensa standard library is not prebuilt (P-0049).
{
  stdenv,
  fetchurl,
  autoPatchelfHook,
  zlib,
  openssl,
  libxml2,
  ncurses,
}:
let
  version = "1.95.0.0";
  toolchain = {
    x86_64-linux = fetchurl {
      url = "https://github.com/esp-rs/rust-build/releases/download/v${version}/rust-${version}-x86_64-unknown-linux-gnu.tar.xz";
      hash = "sha256-qtL7JLrqtq1hxB8ALxNv4LQW7zmlAdKXUKtmwYpplDM=";
    };
    aarch64-linux = fetchurl {
      url = "https://github.com/esp-rs/rust-build/releases/download/v${version}/rust-${version}-aarch64-unknown-linux-gnu.tar.xz";
      hash = "sha256-DH2I5oBfm3egSPMH/LHfDGWGOjBsJ4Ej3HcanLbShEw=";
    };
  }.${stdenv.hostPlatform.system};
  rustSrc = fetchurl {
    url = "https://github.com/esp-rs/rust-build/releases/download/v${version}/rust-src-${version}.tar.xz";
    hash = "sha256-cIvuM3rC1BwOhhr5MEe//skaBS+8tJV5JdhApB8ydxc=";
  };
in
stdenv.mkDerivation {
  pname = "esp-rs-rustc";
  inherit version;
  src = toolchain;
  dontUnpack = true;
  dontConfigure = true;
  dontBuild = true;
  dontStrip = true;

  nativeBuildInputs = [ autoPatchelfHook ];
  buildInputs = [
    stdenv.cc.cc.lib
    zlib
    openssl
    libxml2
    ncurses
  ];

  installPhase = ''
    runHook preInstall
    mkdir unpack
    tar -xJf ${toolchain} -C unpack
    tar -xJf ${rustSrc} -C unpack
    cd unpack
    patchShebangs rust-nightly-*/install.sh rust-src-nightly/install.sh
    ./rust-nightly-*/install.sh \
      --prefix="$out" \
      --disable-ldconfig \
      --without=rust-docs,rust-docs-json-preview
    ./rust-src-nightly/install.sh \
      --prefix="$out" \
      --disable-ldconfig
    runHook postInstall
  '';
}
