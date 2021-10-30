{ pkgs ? import <nixpkgs> {} }:
with pkgs;

mkShell {
  nativeBuildInputs = [ cargo rustc ];
  RUST_SRC_PATH = "${rust.packages.stable.rustPlatform.rustLibSrc}";
}
