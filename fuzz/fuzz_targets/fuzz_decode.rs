#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // 任意のバイト列のデコードがパニックしないことを検証する
    let config = shiguredo_dav1d::DecoderConfig::new();
    let mut decoder = match shiguredo_dav1d::Decoder::new(config) {
        Ok(decoder) => decoder,
        Err(_) => return,
    };

    if decoder.decode(data).is_ok() {
        // デコードできたフレームのプレーンアクセサを呼び出して、
        // 不正な入力に対する panic 耐性を検証する
        while let Ok(Some(frame)) = decoder.next_frame() {
            let _ = frame.y_plane();
            let _ = frame.u_plane();
            let _ = frame.v_plane();
            let _ = frame.y_plane_u16();
            let _ = frame.u_plane_u16();
            let _ = frame.v_plane_u16();
            let _ = frame.y_stride();
            let _ = frame.u_stride();
            let _ = frame.v_stride();
        }
    }
});
