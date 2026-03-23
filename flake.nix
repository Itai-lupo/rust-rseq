{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, flake-utils, rust-overlay, crane, fenix, nixpkgs, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;

          overlays = [ (import rust-overlay) ];

        };

        projectName = ''rseq_on_rust'';
        target = ''x86_64-linux'';

        toolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          targets = [ "x86_64-unknown-linux-musl" ];
          extensions = [ "rust-src" ];
        });
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        src = craneLib.cleanCargoSource ./rseq_main/src + craneLib.cleanCargoSource ./rseq_payload/src;

        rseq-package = craneLib.buildPackage {
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

          nativeBuildInputs = [
            pkgs.musl
            pkgs.llvmPackages.bintools
          ];
        };


      in
      {
        formatter = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
        packages.default = rseq-package;

        devShells.default = craneLib.devShell {
          RUST_BACKTRACE = "1";
          NIX_CFLAGS_LINK = "-fuse-ld=lld";
          packages = with pkgs; [
            just
            rustfmt
            rust-analyzer
            llvmPackages.bintools
          ];
        };
      }
    );
}
