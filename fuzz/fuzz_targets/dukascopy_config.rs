#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &str| {
    let _ = toml::from_str::<dukascopy_core::DataConfig>(input);
});
