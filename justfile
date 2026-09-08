default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

check:
    cargo check --workspace --all-targets --all-features

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo test --workspace --all-features

deny:
    cargo deny check

deps:
    cargo shear

structure:
    python scripts/check_source_shape.py

verify: fmt-check check lint test deny deps structure
    @echo "Nous Wave verification passed."
