{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };
#todo add cross compile for all systems that support rseq at least for checks
  outputs = { flake-utils, rust-overlay, crane, nixpkgs, advisory-db, ... }:
    flake-utils.lib.eachSystem [
  "x86_64-linux"
  "aarch64-linux"
  "riscv64-linux"
  "powerpc64le-linux"
  "s390x-linux"
] (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;

          overlays = [ (import rust-overlay) ];
        };

        inherit (nixpkgs) lib;
        projectName = ''rseq_on_rust'';
        target = ''x86_64-linux'';

        toolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          targets = [ "x86_64-unknown-linux-musl" ];
          extensions = [ "rust-src" ];
        });
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        src = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (craneLib.filterCargoSources path type) ||
            (lib.hasSuffix ".h" path) ||
            (lib.hasSuffix ".ld" path);
        };

        cargoArtifacts = craneLib.buildPackage {
          inherit src;
          strictDeps = true;
          doCheck = true;
          cargoVendorDir = craneLib.vendorMultipleCargoDeps {
            inherit (craneLib.findCargoFiles src) cargoConfigs;
            cargoLockList = [
              ./Cargo.lock
              ./rseq_utils/Cargo.lock
              ./rseq_payload/Cargo.lock
              "${toolchain}/lib/rustlib/src/rust/library/Cargo.lock"
            ];
          };

      

          RUST_BACKTRACE = "1";
          NIX_CFLAGS_LINK = "-fuse-ld=lld";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          GLIBC_TUNABLES = "glibc.pthread.rseq=0";


          nativeBuildInputs = [
            pkgs.clang
            pkgs.llvmPackages.libclang

            pkgs.llvmPackages.bintools
          ];
        };
        rseq-package = craneLib.buildPackage {
          inherit src cargoArtifacts;
        };

      in
      {
        formatter = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
        packages.default = rseq-package;
        checks = {
#todo get clippy to work on main and rseq payload
          # main-clippy = craneLib.cargoClippy { inherit src cargoArtifacts; };
          main-fmt = craneLib.cargoFmt { inherit src; };
          main-audit = craneLib.cargoAudit { inherit src advisory-db; };
          main-deny = craneLib.cargoDeny { inherit src; };
/* 
          submodule-clippy = craneLib.cargoClippy {
            inherit src cargoArtifacts;
            cargoClippyExtraArgs = "--manifest-path=${./rseq_payload}/Cargo.toml";
          }; */
            submodule-fmt = craneLib.cargoFmt {
            src = ./rseq_payload;
          };
          submodule-audit = craneLib.cargoAudit {
            src = ./rseq_payload;
            advisory-db = advisory-db;
          };
      /*    submodule-deny = craneLib.cargoDeny {
            src = ./rseq_payload;
          };   */
        };

        devShells.default = craneLib.devShell {
          RUST_BACKTRACE = "1";
          NIX_CFLAGS_LINK = "-fuse-ld=lld";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";


          packages = with pkgs; [
            clang
            just
            rustfmt
            rustPlatform.bindgenHook
            rust-analyzer
            llvmPackages.bintools
            llvmPackages.libclang
          ];
        };
      }
    );
}
