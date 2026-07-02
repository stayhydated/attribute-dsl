set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    taplo fmt
    rumdl fmt .

clippy:
    cargo clippy --workspace --all-features

check:
    cargo check --workspace --all-features

test:
    cargo test --workspace --all-features

cov:
    cargo llvm-cov --workspace --exclude xtask --exclude web --all-features --all-targets

test-publish:
    cargo xtask release plan

test-docs:
    cargo clean --doc
    cargo doc --workspace --all-features --no-deps --open

ci: fmt check clippy test cov
