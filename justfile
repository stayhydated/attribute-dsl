set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    taplo fmt
    rumdl fmt .

clippy:
    cargo clippy --workspace --all-features --all-targets -- -D warnings

check:
    cargo check --workspace --all-features --all-targets

test:
    cargo test --workspace --all-features --all-targets

cov:
    cargo llvm-cov --workspace --all-features --all-targets

test-publish:
    cargo publish --dry-run --locked --allow-dirty

test-docs:
    cargo clean --doc
    cargo doc --workspace --all-features --no-deps --open

ci: fmt check clippy test cov test-publish
    cargo machete
