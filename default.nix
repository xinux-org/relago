{
  pkgs,
  craneLib,
  ...
}:
let
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
    systemd.dev
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

    gtk4
    libadwaita
    glib
    cairo
    pango
    gdk-pixbuf
    graphene
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
    RUST_MIN_STACK = 16777216;

    # # Compiler LD variables
    NIX_LDFLAGS = "-L${(getLibFolder pkgs.libiconv)} -L${(getLibFolder pkgs.pkg-config)} -L${(getLibFolder pkgs.dbus.dev)}";
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
      with pkgs;
      [
        gcc
        libiconv
        # postgresql
        llvmPackages.llvm
        dbus.dev
      ]
    );

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
craneLib.buildPackage (
  {
    pname = manifest.name;
    version = manifest.version;

    inherit src cargoArtifacts;

    nativeBuildInputs = commonNativeBuildInputs;
    buildInputs = commonBuildInputs;

    postInstall = ''
            install -D -m 644 /dev/stdin $out/share/dbus-1/system.d/org.freedesktop.problems.daemon.conf <<EOF
      <!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
       "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
      <busconfig>
        <policy user="relago">
          <allow own="org.freedesktop.problems.daemon"/>
          <allow send_destination="org.freedesktop.problems.daemon"/>
          <allow receive_sender="org.freedesktop.problems.daemon"/>
        </policy>

        <policy context="default">
          <allow send_destination="org.freedesktop.problems.daemon"/>
        </policy>
      </busconfig>
      EOF
    '';
  }
  // common
)
