{
  pkgs ? let
    lock = (builtins.fromJSON (builtins.readFile ./flake.lock)).nodes.nixpkgs.locked;
    nixpkgs = fetchTarball {
      url = "https://github.com/nixos/nixpkgs/archive/${lock.rev}.tar.gz";
      sha256 = lock.narHash;
    };
  in
    import nixpkgs {overlays = [];},
  ...
}: let
  # Helpful nix function
  getLibFolder = pkg: "${pkg}/lib";

  # Manifest via Cargo.toml
  manifest = (pkgs.lib.importTOML ./Cargo.toml).workspace.package;
in
  pkgs.stdenv.mkDerivation {
    name = "${manifest.name}-dev";

    # Compile time dependencies
    nativeBuildInputs = with pkgs; [
      # GCC toolchain
      gcc
      gnumake
      pkg-config

      # LLVM toolchain
      cmake
      llvmPackages.llvm
      llvmPackages.clang

      # Hail the Nix
      nixd
      statix
      deadnix
      alejandra

      # Rust
      rustc
      cargo
      rustfmt
      clippy
      rust-analyzer
      cargo-watch

      # A few utils
      just

      systemd
      pkg-config
    ];

    # Runtime dependencies which will be shipped
    # with nix package
    buildInputs = with pkgs; [
      openssl
      dbus
      systemd
      # pkg-config
    ];

    # Set Environment Variables
    RUST_BACKTRACE = "full";
    RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

    DBUS_PATH = "${pkgs.dbus}";

    # Compiler LD variables
    # > Make sure packages have /lib or /include path'es
    NIX_LDFLAGS = "-L${(getLibFolder pkgs.libiconv)} -L${getLibFolder pkgs.postgresql} -L${(getLibFolder pkgs.dbus)}";
    
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
      pkgs.gcc
      pkgs.libiconv
      pkgs.llvmPackages.llvm
      pkgs.dbus
    ];

    shellHook = ''
    '';

    ####################################################################
    # Without  this, almost  everything  fails with  locale issues  when
    # using `nix-shell --pure` (at least on NixOS).
    # See
    # + https://github.com/NixOS/nix/issues/318#issuecomment-52986702
    # + http://lists.linuxfromscratch.org/pipermail/lfs-support/2004-June/023900.html
    ####################################################################

    LOCALE_ARCHIVE =
      if pkgs.stdenv.isLinux
      then "${pkgs.glibcLocales}/lib/locale/locale-archive"
      else "";
  }
