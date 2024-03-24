{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  packages = with pkgs; [
    rustup
    graphviz
    imagemagick
    protobuf
    time
  ];
}
