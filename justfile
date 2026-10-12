set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    taplo fmt
    rumdl fmt .

quality:
    cargo sort-derives --check
    taplo fmt --check
    rumdl check .
    typos
    actionlint
    git ls-files -z '*.sh' '*.bash' | xargs -0 --no-run-if-empty shellcheck

clippy:
    cargo clippy --workspace --all-features --all-targets --locked -- -D warnings

check:
    cargo check --workspace --all-features --all-targets --locked

test:
    cargo test --workspace --all-features --all-targets --locked
    cargo test --workspace --all-features --doc --locked

cov:
    cargo llvm-cov --workspace --all-features --all-targets --locked

test-publish:
    cargo publish --dry-run --locked --allow-dirty

test-docs:
    cargo clean --doc
    cargo doc --workspace --all-features --no-deps --locked --open

book:
    mdbook serve book

web-build:
    cargo run --package xtask --locked -- build book
    cargo run --package xtask --locked -- build llms-txt
    cargo run --package xtask --locked -- build web

web: web-build
    dx serve --package web

web-preview: web-build
    cargo xtask preview web

ci: fmt quality check clippy test cov test-publish
    cargo machete
