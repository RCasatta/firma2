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
        defaultPackage = craneLib.buildPackage {
          inherit (crateName) pname version;
          # src = craneLib.cleanCargoSource ./.;
          src = ./.;

          BITCOIND_EXE = "${pkgs.bitcoind}/bin/bitcoind";

          # Add extra inputs here or any other derivation settings
          doCheck = true;
          # buildInputs = [];
          # nativeBuildInputs = [];
        };
      in
      {
        packages.default = defaultPackage;

        devShells.default = pkgs.mkShell {
          buildInputs = [ defaultPackage pkgs.jq pkgs.age ];
        };

      });
}
