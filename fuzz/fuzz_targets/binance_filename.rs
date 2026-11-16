#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &str| {
    let _ = binance_core::Period::Daily.parse_file_date(input);
    let _ = binance_core::Period::Monthly.parse_file_date(input);
    let _ = input.parse::<binance_core::Market>();
    let _ = input.parse::<binance_core::Period>();
    let _ = input.parse::<binance_core::Dataset>();
    let _ = input.parse::<binance_core::Interval>();
    let _ = input.parse::<binance_core::BadRowPolicy>();
    let _ = binance_core::Symbol::parse(input);
});
