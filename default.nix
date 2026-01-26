{
  pkgs,
  craneLib,
  ...
}: let
  # Helpful nix function
  lib = pkgs.lib;
  getLibFolder = pkg: "${pkg}/lib";

  # Manifest via Cargo.toml
  manifest = (pkgs.lib.importTOML ./Cargo.toml).workspace.package;

  # Compile time dependencies
  commonNativeBuildInputs = with pkgs; [
    # GCC toolchain
    gcc
    gnumake
    pkg-config

    # LLVM toolchain
    cmake
    llvmPackages.llvm
    llvmPackages.clang

    # Rust
    rustc
    cargo
    clippy

    # Other compile time dependencies
    pkg-config

    dbus.dev
    zlib
    # libssl
  ];

  # Runtime dependencies which will be shipped
  # with nix package
  commonBuildInputs = with pkgs; [
    openssl
    pkg-config
    # libressl

    dbus.dev
    systemd.dev

    zlib
    pkg-configUpstream
  ];

  src = craneLib.cleanCargoSource ./.;

  common = {
    # dbus = pkgs.dbus;
    DBUS_PATH = "${pkgs.dbus}";

    # fixupPhase = ''
    #   mkdir -p $out/mgrs
    #   cp -R ./crates/database/* $out/mgrs
    # '';

    # Set Environment Variables
    RUST_BACKTRACE = 1;
    RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

    # # Compiler LD variables
    NIX_LDFLAGS = "-L${(getLibFolder pkgs.libiconv)} -L${(getLibFolder pkgs.pkg-config)} -L${(getLibFolder pkgs.dbus.dev)}";
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
      gcc
      libiconv
      # postgresql
      llvmPackages.llvm
      dbus.dev
    ]);

    # PKG_CONFIG_PATH = "${pkgs.dbus.dev}/lib/pkgconfig";
  };

  cargoArtifacts =
    craneLib.buildDepsOnly {
      inherit src;
      strictDeps = true;

      nativeBuildInputs = commonNativeBuildInputs;
      buildInputs = commonBuildInputs;
    }
    // common;
in
  craneLib.buildPackage ({
      pname = manifest.name;
      version = manifest.version;

      inherit src cargoArtifacts;

      nativeBuildInputs = commonNativeBuildInputs;
      buildInputs = commonBuildInputs;
    }
    // common)
