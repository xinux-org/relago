{ pkgs, ... }:
{
  projectRootFile = "flake.nix";
  programs.rustfmt = {
    enable = true;
    edition = "2021";
  };
  programs.nixfmt.enable = true;
  programs.toml-sort.enable = true;
}
