{
  description = "Relago — bug reporter for Xinux";

  inputs = {
    # Perfect!
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    treefmt-nix.url = "github:numtide/treefmt-nix";

    # The flake-parts library
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";

    git-hooks-nix = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      treefmt-nix,
      ...
    }@inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } (
      { ... }:
      let
        systems = [
          "x86_64-linux"
          "aarch64-linux"
          "aarch64-darwin"
        ];
      in
      {
        inherit systems;
        flake = {
          nixosModules.relago = import ./module.nix self;
          nixosModules.default = import ./module.nix self;
          hydraJobs.x86_64-linux.relago = self.packages.x86_64-linux.relago;
        };

        imports = [
          inputs.treefmt-nix.flakeModule
          inputs.git-hooks-nix.flakeModule
        ];

        perSystem =
          {
            system,
            config,
            ...
          }:
          let
            pkgs = nixpkgs.legacyPackages.${system};
            craneLib = inputs.crane.mkLib pkgs;
          in
          rec {
            # Development environment
            devShells.default = import ./shell.nix {
              inherit self pkgs craneLib;
              shellHook = config.pre-commit.installationScript;
            };

            # Output package
            # packages.default = pkgs.callPackage ./. {inherit pkgs;};
            packages = {
              default = self.packages.${system}.relago;
              relago = pkgs.callPackage ./. { inherit pkgs craneLib; };
              relago-dev = self.packages.${system}.relago.overrideAttrs {
                dontCheck = true;
              };
            };

            treefmt = import ./treefmt.nix;

            pre-commit.settings.hooks.treefmt = {
              enable = true;
              package = config.treefmt.build.wrapper;
            };

            checks.pre-commit-check = config.pre-commit.settings.hooks;
          };
      }
    );
}
