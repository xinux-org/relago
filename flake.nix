{
  description = "Relago — bug reporter for Xinux";

  inputs = {
    # Perfect!
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # The flake-parts library
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    git-hooks-nix = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane.url = "github:ipetkov/crane";
  };

  outputs =
    { self, ... }@inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = with inputs; [
        treefmt-nix.flakeModule
        git-hooks-nix.flakeModule
      ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      flake = {
        nixosModules.relago = import ./module.nix self;
        nixosModules.default = import ./module.nix self;
        hydraJobs.x86_64-linux.relago = self.packages.x86_64-linux.relago;
      };

      perSystem =
        {
          pkgs,
          system,
          config,
          ...
        }:
        let
          craneLib = inputs.crane.mkLib pkgs;

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

            # Rust
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

          src = lib.cleanSourceWith {
            src = ./.;
            name = "source";
            filter =
              path: type: (builtins.match ".*ftl$" path != null || craneLib.filterCargoSources path type);
          };

          common = {
            inherit src;
            strictDeps = true;

            pname = manifest.name;
            version = manifest.version;

            nativeBuildInputs = commonNativeBuildInputs;
            buildInputs = commonBuildInputs;

            # dbus = pkgs.dbus;
            DBUS_PATH = "${pkgs.dbus}";

            # fixupPhase = ''
            #   mkdir -p $out/mgrs
            #   cp -R ./crates/database/* $out/mgrs
            # '';

            # Set Environment Variables
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            RUST_MIN_STACK = 16777216;

            WEBKIT_DISABLE_DMABUF_RENDERER = 0;

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

          cargoArtifacts = craneLib.buildDepsOnly common;
        in
        {
          # Development environment
          devShells.default = import ./shell.nix {
            inherit self pkgs craneLib;
            shellHook = config.pre-commit.installationScript;
          };

          # Output package
          packages = {
            default = self.packages.${system}.relago;
            relago = craneLib.buildPackage (
              {
                inherit cargoArtifacts;
              }
              // common
            );
            relago-dev = self.packages.${system}.relago.overrideAttrs {
              dontCheck = true;
            };
          };

          checks = {
            relago-clippy = craneLib.cargoClippy (
              common
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              }
            );

            relago-doc = craneLib.cargoDoc (
              common
              // {
                inherit cargoArtifacts;
                env.RUSTDOCFLAGS = "--deny warnings";
              }
            );

            relago-test = craneLib.cargoTest (
              common
              // {
                inherit cargoArtifacts;
              }
            );
          };

          treefmt = import ./treefmt.nix;

          pre-commit.settings.hooks = {
            treefmt.enable = true;
          };
        };
    };
}
