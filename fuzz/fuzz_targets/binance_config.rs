#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &str| {
    let _ = binance_core::Config::from_toml_str(input);
});
