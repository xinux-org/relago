{
  self,
  pkgs,
  craneLib,
  ...
}:
let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).workspace.package;
in
craneLib.devShell {
  name = "${manifest.name}-dev";

  # Compile time dependencies
  packages = with pkgs; [
    nixd
    statix
    deadnix
    self.formatter.${pkgs.stdenv.hostPlatform.system}
    nixfmt-tree

    # Rust
    rustc
    cargo
    rustfmt
    clippy
    rust-analyzer
    cargo-watch
    cargo-expand

    pkg-config
    dbus.dev
    systemd.dev

    gtk4
    libadwaita
    cairo
    pango
    graphene
    openssl
    rustPlatform.bindgenHook
    pprof
  ];

  # Set Environment Variables
  RUST_BACKTRACE = "full";
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
