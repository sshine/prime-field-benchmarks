let
  pkgs = import <nixpkgs> {};
in
  pkgs.mkShell {
    packages = with pkgs; [
      rustup
      pkg-config
      cargo-criterion
    ];
  }
