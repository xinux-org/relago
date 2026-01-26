{
  description = "Relago";

  inputs = {
    # Stable for keeping thins clean
    # nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";

    # Fresh and new for testing
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    crane.url = "github:ipetkov/crane";

    # The flake-utils library
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      craneLib = crane.mkLib pkgs;
    in {
      # Nix script formatter
      formatter = pkgs.nixfmt-rfc-style;

      # Development environment
      devShells.default = import ./shell.nix {inherit self pkgs craneLib;};

      # Output package
      packages = {
        default = self.packages.${system}.relago;
        relago = pkgs.callPackage ./. {inherit pkgs craneLib;};
        relago-dev = self.packages.${system}.relago.overrideAttrs {
          dontCheck = true;
        };
      };
    })
    // {
      # NixOS module (deployment)
      nixosModules.relago = import ./module.nix self;
    };
}
