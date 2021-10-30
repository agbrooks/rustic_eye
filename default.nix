{ pkgs ? import <nixpkgs> {} }:
with pkgs;

rustPlatform.buildRustPackage rec {
  pname = "rustic_eye";
  version = "0.1.0";

  src = lib.sourceByRegex ./. [
    "src"
    "Cargo\\.((lock)|(toml))"
    ".+\\.rs$"
  ];

  cargoSha256 = "1vmg106b96462l2zmcnl9qvcgkjyra8f56p27lk6i410ibk5965q";

  meta = with lib; {
    description = "Add height map-based stereoscopy to images";
    homepage = "https://github.com/agbrooks/rustic_eye";
    license = licenses.mit;
  };
}
