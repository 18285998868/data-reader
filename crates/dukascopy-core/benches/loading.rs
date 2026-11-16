//! Benchmarks for planning, reading and decoding a real `.bi5` fixture.
#![expect(clippy::expect_used, reason = "benchmark fixture setup")]

use std::hint::black_box;

use chrono::NaiveDate;
use criterion::{Criterion, criterion_group, criterion_main};
use dukascopy_core::{DataConfig, PriceType, Timeframe, load_columns};

fn fixture_config() -> DataConfig {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
    DataConfig::new(root, "EURUSD", Timeframe::D1)
        .expect("fixture config is valid")
        .with_price_type(PriceType::Bid)
        .with_date_range(
            Some(NaiveDate::from_ymd_opt(2025, 1, 1).expect("valid date")),
            Some(NaiveDate::from_ymd_opt(2025, 12, 31).expect("valid date")),
        )
        .expect("fixture date range is valid")
}

fn loading(c: &mut Criterion) {
    let config = fixture_config();
    c.bench_function("load_columns_daily_fixture", |b| {
        b.iter(|| black_box(load_columns(&config).expect("fixture decodes")));
    });
}

criterion_group!(benches, loading);
criterion_main!(benches);
