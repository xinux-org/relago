{
  self,
  pkgs,
  craneLib,
  ...
}: let
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

      rust-analyzer
    ];
  }
