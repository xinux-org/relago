{
  description = "Relago — bug reporter for Xinux";

  inputs = {
    # Too old to work with most libraries
    # nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";

    # Perfect!
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    # The flake-parts library
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  } @ inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} ({...}: {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      # pkgs = nixpkgs.legacyPackages.${system};
      flake = {
      	nixosModules.relago = import ./module.nix self;
      };
      
      perSystem = {
        system,
        ...
      }: let 
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = inputs.crane.mkLib pkgs;
        # slf = inputs.self;
       in rec {

        # Nix script formatter
        formatter = pkgs.nixfmt-rfc-style;

        # Development environment
        devShells.default = import ./shell.nix  {inherit self pkgs craneLib;};

        # Output package
        # packages.default = pkgs.callPackage ./. {inherit pkgs;};
        packages = {
          default = self.packages.${system}.relago;
          relago = pkgs.callPackage ./. {inherit pkgs craneLib;};
          relago-dev = self.packages.${system}.relago.overrideAttrs {
            dontCheck = true;
          };
      };
		
      };
    });
}
