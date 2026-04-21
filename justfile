set shell := ["bash", "-cu"]

default:
    @just --list

fmt:
    cargo fmt --all

lint:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings

build:
    cargo build

build-release:
    cargo auditable build --release

test:
    cargo test --all-features

deny:
    cargo deny check

audit:
    cargo audit

check: lint test

ci: lint deny test audit
    @echo "CI checks passed locally."

install-tools:
    command -v cargo-deny >/dev/null || cargo install cargo-deny --locked
    command -v cargo-audit >/dev/null || cargo install cargo-audit --locked
    command -v cargo-auditable >/dev/null || cargo install cargo-auditable --locked
