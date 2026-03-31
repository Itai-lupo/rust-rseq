build:
    nix build -L

clean-build:
    nix build --rebuild -L

run:
    nix run -L

test:
    cargo test --release

clean:
    cargo clean
    rm -f result

check:
    nix flags check
