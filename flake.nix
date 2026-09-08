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
        "aarch64-darwin"
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
        in
        rec {
          # Development environment
          devShells.default = import ./shell.nix {
            inherit self pkgs craneLib;
            shellHook = config.pre-commit.installationScript;
          };

          # Output package
          packages = {
            default = self.packages.${system}.relago;
            relago = pkgs.callPackage ./. { inherit pkgs craneLib; };
            relago-dev = self.packages.${system}.relago.overrideAttrs {
              dontCheck = true;
            };
          };

          treefmt = import ./treefmt.nix;
        };
    };
}
