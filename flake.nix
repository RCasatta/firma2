{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;
        crateName = craneLib.crateNameFromCargoToml {
          cargoToml = ./cli/Cargo.toml;
        };
      in
      {
        packages.default = craneLib.buildPackage {
          inherit (crateName) pname version;
          src = craneLib.cleanCargoSource ./.;

          # Add extra inputs here or any other derivation settings
          # doCheck = true;
          # buildInputs = [];
          # nativeBuildInputs = [];
        };
      });
}
