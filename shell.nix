{
  self,
  pkgs,
  craneLib,
  ...
}:
let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).workspace.package;
  isLinux = pkgs.stdenv.hostPlatform.isLinux;
  linuxPackages = pkgs.lib.optionals isLinux (
    with pkgs;
    [
      dbus.dev
      systemd.dev
      dbus
      gtk4
      libadwaita
      cairo
      pango
      graphene
      gdk-pixbuf
      pprof
    ]
  );
in
craneLib.devShell {
  name = "${manifest.name}-dev";

  # Compile time dependencies
  packages =
    with pkgs;
    [
      nixd
      statix
      deadnix
      self.formatter.${pkgs.stdenv.hostPlatform.system}
      nixfmt-tree

      # Rust
      rustc
      cargo
      rustfmt
      clippy
      rust-analyzer
      cargo-watch
      cargo-expand

      pkg-config
      xz
      openssl
      rustPlatform.bindgenHook
    ]
    ++ linuxPackages;

  # Set Environment Variables
  RUST_BACKTRACE = "full";
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
