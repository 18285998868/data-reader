# data-reader

[![CI](https://github.com/18285998868/data-reader/actions/workflows/ci.yml/badge.svg)](https://github.com/18285998868/data-reader/actions/workflows/ci.yml)

Rust readers for local Binance and Dukascopy public archives. Both crates load,
decode and validate archive data into typed columnar output; they do not
download, transform or compute indicators.

## Crates

| Crate | Layout | Entry point |
| --- | --- | --- |
| [`binance-core`](crates/binance-core/README.md) | `{root}/data/{market}/{period}/{dataset}/{symbol}[/{interval}]/*.zip` | `read`, CLI, Python |
| [`dukascopy-core`](crates/dukascopy-core/README.md) | `{root}/dukascopy/{symbol}/{YYYY}/[{MM}/[{DD}/]]{BID\|ASK}_candles_*.bi5` | `load`, `load_columns`, Python |

## Build and test

1. Use the latest stable toolchain pinned by `rust-toolchain.toml`.
2. Run the workspace gate.
3. Run the real-archive smoke requests separately.

```bash
cargo build --workspace --release
./scripts/check.sh
cargo run -p binance-core -- read -c crates/binance-core/config/00.smoke.toml
cargo run -p dukascopy-core --example load -- crates/dukascopy-core/config/00.smoke.toml
```

`./scripts/check.sh --ci` additionally requires `cargo-deny` and `cargo-audit`.
Real archives are expected at the roots configured in each crate's
`config/00.smoke.toml`.

Benchmarks run with `cargo bench --workspace`. Fuzzing uses nightly and
`cargo-fuzz`: `cd fuzz && cargo +nightly fuzz run binance_filename`.

## Python

Each crate has its own `pyproject.toml`; build it from its own directory:

```bash
cd crates/binance-core && maturin develop --features python
cd crates/dukascopy-core && maturin develop --features python
```

## Documentation

- Binance execution path: [`crates/binance-core/STEP.md`](crates/binance-core/STEP.md)
- Dukascopy execution path: [`crates/dukascopy-core/STEP.md`](crates/dukascopy-core/STEP.md)
- Design decisions: [`crates/binance-core/docs/adr/`](crates/binance-core/docs/adr/)
- Licenses: MIT (`LICENSE`)
