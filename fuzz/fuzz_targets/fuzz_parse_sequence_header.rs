#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // 任意のバイト列でシーケンスヘッダーのパースがパニックしないことを検証する
    let _ = shiguredo_dav1d::parse_sequence_header(data);
});
