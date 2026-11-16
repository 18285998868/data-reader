//! Microbenchmarks for the public parsing primitives.

use std::hint::black_box;

use binance_core::{Interval, Market, Period, Symbol};
use criterion::{Criterion, criterion_group, criterion_main};

fn parsing(c: &mut Criterion) {
    c.bench_function("period_parse_file_date", |b| {
        b.iter(|| black_box(Period::Daily.parse_file_date(black_box("BTCUSDT-1m-2024-01-02"))));
    });
    c.bench_function("interval_from_str", |b| {
        b.iter(|| black_box("1h".parse::<Interval>()));
    });
    c.bench_function("market_from_str", |b| {
        b.iter(|| black_box("futures/um".parse::<Market>()));
    });
    c.bench_function("symbol_parse", |b| {
        b.iter(|| black_box(Symbol::parse(black_box("BTCUSDT"))));
    });
}

criterion_group!(benches, parsing);
criterion_main!(benches);
