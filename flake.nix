{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, flake-utils, crane, fenix, nixpkgs, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;

          overlays = [
            fenix.overlays.default
          ];

        };

        projectName = "rseq_on_rust";
        target = "x86_64-linux";

        toolchain = fenix.packages.${system}.complete.withComponents [ "rustc" "cargo" "rust-src" ];


        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        src = craneLib.cleanCargoSource ./rseq_main/src +  craneLib.cleanCargoSource ./rseq_payload/src;

        arduino-package = craneLib.buildPackage {
          inherit src;
          strictDeps = true;

          doCheck = false;
          cargoVendorDir = craneLib.vendorMultipleCargoDeps {
            inherit (craneLib.findCargoFiles src) cargoConfigs;
            cargoLockList = [
              ./Cargo.lock
              "${toolchain}/lib/rustlib/src/rust/library/Cargo.lock"
            ];
          };

          buildInputs = [
           pkgs.musl 
          ];
        };


      in
      {
        formatter = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
        packages.default = arduino-package;
 
        # apps.${system}.default = self.apps.flash;

        devShells.default = craneLib.devShell {
          RUST_BACKTRACE = "1";

          packages = with pkgs; [
            just
            rustfmt
            rust-analyzer
          ];
        };
      }
    );
}
