{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  packages = with pkgs; [
    rustup

    ocamlformat
    ocamlPackages.ocaml
    ocamlPackages.menhir
    ocamlPackages.dune_3
    ocamlPackages.findlib
    ocamlPackages.bisect_ppx
    ocamlPackages.ppxlib
    ocamlPackages.ppx_inline_test
    ocamlPackages.pprint
    ocamlPackages.z3
  ];
}
