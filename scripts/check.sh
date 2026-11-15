#!/usr/bin/env bash
# OS: CachyOS
# TIME: 2026-11-15
# DOC: Offline quality gate for the data-reader workspace
# LOGIC:
#   STEP 1: check formatting
#   STEP 2: check default, no-default and all-feature builds
#   STEP 3: run clippy with warnings denied for every target
#   STEP 4: run default-feature tests and doctests
#   STEP 5: build rustdoc with warnings denied
#   STEP 6: run Python-feature tests only when NumPy is importable
#   STEP 7: run cargo-deny and cargo-audit (CI mode requires both)
#   STEP 8: compile benchmarks when a benches/ directory exists
# USAGE:
#   ./scripts/check.sh [--ci]
#
# Q:
#   Q1: This gate is offline and does not read real archives. Run the smoke
#       requests in each crate's config/00.smoke.toml separately.
#   Q2: CI mode fails when cargo-deny or cargo-audit is missing; local runs
#       print a skip instead.

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
    set -Eeuo pipefail
fi

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
cd -- "$root"

ci=0
case ${1:-} in
    --ci) ci=1 ;;
    "") ;;
    *)
        printf 'unknown argument: %s\n' "$1" >&2
        exit 2
        ;;
esac

info() { printf '==> %s\n' "$*" >&2; }

info "cargo fmt"
cargo fmt --all -- --check

info "cargo check (default features)"
cargo check --workspace --all-targets --locked

info "cargo check (no default features)"
cargo check --workspace --all-targets --no-default-features --locked

info "cargo check (all features)"
cargo check --workspace --all-targets --all-features --locked

info "cargo clippy (default features)"
cargo clippy --workspace --all-targets --locked -- -D warnings

info "cargo clippy (no default features)"
cargo clippy --workspace --all-targets --no-default-features --locked -- -D warnings

info "cargo clippy (all features)"
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

info "tests"
if command -v cargo-nextest >/dev/null; then
    cargo nextest run --workspace --locked
else
    cargo test --workspace --locked
fi

info "doctests"
cargo test --workspace --doc --locked

info "rustdoc"
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --all-features --locked

if python3 -c 'import numpy' >/dev/null 2>&1; then
    info "Python-feature tests"
    if command -v cargo-nextest >/dev/null; then
        cargo nextest run --workspace --all-features --locked
    else
        cargo test --workspace --all-features --locked
    fi
else
    info "skip Python-feature tests (NumPy is not importable by python3)"
fi

if ((ci)); then
    command -v cargo-deny >/dev/null || {
        printf 'cargo-deny is required in CI mode\n' >&2
        exit 1
    }
    command -v cargo-audit >/dev/null || {
        printf 'cargo-audit is required in CI mode\n' >&2
        exit 1
    }
fi
if command -v cargo-deny >/dev/null; then
    info "cargo deny"
    cargo deny check
else
    info "skip cargo deny (not installed)"
fi
if command -v cargo-audit >/dev/null; then
    info "cargo audit"
    cargo audit
else
    info "skip cargo audit (not installed)"
fi

if [[ -d benches ]]; then
    info "compile benchmarks"
    cargo bench --workspace --no-run --locked
fi

info "offline gate passed"
