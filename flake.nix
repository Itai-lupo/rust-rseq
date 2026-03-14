{
  description = "Rust rseq example";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [ rustc rustfmt cargo ];
        };
        packages.default = pkgs.rustPlatform.buildRustPackage {
          name = "rseq-example";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
      });
}   
