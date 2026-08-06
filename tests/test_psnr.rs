use shiguredo_dav1d::{Decoder, DecoderConfig, PixelLayout};

// ============================================================================
// フレーム生成ヘルパー
// ============================================================================

/// ダミー I420 フレームを生成する
///
/// Y プレーンはフレーム番号に応じたグラデーション、UV プレーンは 128 固定。
fn generate_dummy_i420(
    width: usize,
    height: usize,
    frame_index: usize,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let y_size = width * height;
    let uv_width = width.div_ceil(2);
    let uv_height = height.div_ceil(2);
    let uv_size = uv_width * uv_height;

    let mut y = vec![0u8; y_size];
    for row in 0..height {
        for col in 0..width {
            y[row * width + col] = ((col + row + frame_index * 7) % 256) as u8;
        }
    }

    let u = vec![128u8; uv_size];
    let v = vec![128u8; uv_size];

    (y, u, v)
}

/// SMPTE カラーバー風の I420 フレームを生成する
///
/// 7 色の縦ストライプ（白/黄/シアン/緑/マゼンタ/赤/青）を
/// BT.601 で YUV に変換し I420 形式で返す。
fn generate_colorbar_i420(width: usize, height: usize) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    // SMPTE カラーバーの RGB 値（白/黄/シアン/緑/マゼンタ/赤/青）
    let bars: [(u8, u8, u8); 7] = [
        (235, 235, 235), // 白
        (235, 235, 16),  // 黄
        (16, 235, 235),  // シアン
        (16, 235, 16),   // 緑
        (235, 16, 235),  // マゼンタ
        (235, 16, 16),   // 赤
        (16, 16, 235),   // 青
    ];

    let y_size = width * height;
    let uv_width = width.div_ceil(2);
    let uv_height = height.div_ceil(2);
    let uv_size = uv_width * uv_height;

    let mut y_plane = vec![0u8; y_size];
    let mut u_plane = vec![128u8; uv_size];
    let mut v_plane = vec![128u8; uv_size];

    for row in 0..height {
        for col in 0..width {
            let bar_index = col * 7 / width;
            let (r, g, b) = bars[bar_index];

            // BT.601 RGB -> YCbCr
            let rf = r as f64;
            let gf = g as f64;
            let bf = b as f64;
            let yv = (0.257 * rf + 0.504 * gf + 0.098 * bf + 16.0).clamp(16.0, 235.0) as u8;
            y_plane[row * width + col] = yv;

            // UV は 2x2 ブロック単位（左上ピクセルで代表する）
            if row % 2 == 0 && col % 2 == 0 {
                let u = (-0.148 * rf - 0.291 * gf + 0.439 * bf + 128.0).clamp(16.0, 240.0) as u8;
                let v = (0.439 * rf - 0.368 * gf - 0.071 * bf + 128.0).clamp(16.0, 240.0) as u8;
                let uv_row = row / 2;
                let uv_col = col / 2;
                u_plane[uv_row * uv_width + uv_col] = u;
                v_plane[uv_row * uv_width + uv_col] = v;
            }
        }
    }

    (y_plane, u_plane, v_plane)
}

/// SMPTE カラーバー風の 10-bit I420 フレームを生成する
///
/// `generate_colorbar_i420` と同じ BT.601 変換を 10-bit レンジ (0..1023) にスケールする。
/// 8-bit のスタジオレンジ (Y: 16..235、UV: 16..240) を 4 倍した
/// 10-bit のスタジオレンジ (Y: 64..940、UV: 64..960) で表現する。
fn generate_colorbar_i420_16bit(width: usize, height: usize) -> (Vec<u16>, Vec<u16>, Vec<u16>) {
    // SMPTE カラーバーの RGB 値 (白/黄/シアン/緑/マゼンタ/赤/青)
    let bars: [(u8, u8, u8); 7] = [
        (235, 235, 235), // 白
        (235, 235, 16),  // 黄
        (16, 235, 235),  // シアン
        (16, 235, 16),   // 緑
        (235, 16, 235),  // マゼンタ
        (235, 16, 16),   // 赤
        (16, 16, 235),   // 青
    ];

    let y_size = width * height;
    let uv_width = width.div_ceil(2);
    let uv_height = height.div_ceil(2);
    let uv_size = uv_width * uv_height;

    let mut y_plane = vec![0u16; y_size];
    let mut u_plane = vec![512u16; uv_size];
    let mut v_plane = vec![512u16; uv_size];

    for row in 0..height {
        for col in 0..width {
            let bar_index = col * 7 / width;
            let (r, g, b) = bars[bar_index];

            // BT.601 RGB -> YCbCr (8-bit の結果を 4 倍して 10-bit レンジにする)
            let rf = r as f64;
            let gf = g as f64;
            let bf = b as f64;
            let yv =
                (4.0 * (0.257 * rf + 0.504 * gf + 0.098 * bf + 16.0)).clamp(64.0, 940.0) as u16;
            y_plane[row * width + col] = yv;

            // UV は 2x2 ブロック単位（左上ピクセルで代表する）
            if row % 2 == 0 && col % 2 == 0 {
                let u = (4.0 * (-0.148 * rf - 0.291 * gf + 0.439 * bf + 128.0)).clamp(64.0, 960.0)
                    as u16;
                let v = (4.0 * (0.439 * rf - 0.368 * gf - 0.071 * bf + 128.0)).clamp(64.0, 960.0)
                    as u16;
                let uv_row = row / 2;
                let uv_col = col / 2;
                u_plane[uv_row * uv_width + uv_col] = u;
                v_plane[uv_row * uv_width + uv_col] = v;
            }
        }
    }

    (y_plane, u_plane, v_plane)
}

// ============================================================================
// 品質計測ヘルパー
// ============================================================================

/// Y プレーン同士の PSNR を計算する（dB）
///
/// 値が大きいほど入力と出力が近い。一般に 30dB 以上あれば視覚的に良好。
fn psnr_y(original: &[u8], decoded: &[u8], width: usize, height: usize) -> f64 {
    let y_size = width * height;
    assert!(original.len() >= y_size);
    assert!(decoded.len() >= y_size);

    let mut mse_sum: f64 = 0.0;
    for i in 0..y_size {
        let diff = original[i] as f64 - decoded[i] as f64;
        mse_sum += diff * diff;
    }
    let mse = mse_sum / y_size as f64;
    if mse == 0.0 {
        return f64::INFINITY;
    }
    10.0 * (255.0_f64 * 255.0 / mse).log10()
}

/// Y プレーン同士の PSNR を計算する（dB、16-bit 入力版）
///
/// 10-bit レンジ (0..1023) の画素値同士を比較するため、最大値は 1023 を使う。
fn psnr_y_u16(original: &[u16], decoded: &[u16], width: usize, height: usize) -> f64 {
    let y_size = width * height;
    assert!(original.len() >= y_size);
    assert!(decoded.len() >= y_size);

    let mut mse_sum: f64 = 0.0;
    for i in 0..y_size {
        let diff = original[i] as f64 - decoded[i] as f64;
        mse_sum += diff * diff;
    }
    let mse = mse_sum / y_size as f64;
    if mse == 0.0 {
        return f64::INFINITY;
    }
    10.0 * (1023.0_f64 * 1023.0 / mse).log10()
}

// ============================================================================
// dav1d デコードヘルパー
// ============================================================================

/// デコード結果の Y プレーンをストライド無しで抽出する
///
/// dav1d のデコード結果はストライドが幅と一致するとは限らないため、
/// 行ごとに width 分だけコピーして詰める。
fn extract_y_plane(frame: &shiguredo_dav1d::DecodedFrame) -> Vec<u8> {
    let width = frame.width();
    let height = frame.height();
    let stride = frame.y_stride();
    let y_data = frame.y_plane();
    let mut y = Vec::with_capacity(width * height);
    for row in 0..height {
        y.extend_from_slice(&y_data[row * stride..row * stride + width]);
    }
    y
}

/// デコード結果の Y プレーンを u16 で抽出する (10-bit 用)
///
/// `y_plane_u16()` のスライスは `height * (stride / 2)` 要素で各行末にパディングを含むため、
/// 行ごとに width 分だけコピーして詰める。ストライドはバイト単位で、10-bit では
/// 1 ピクセル 2 バイトになるため行幅は `stride / 2` 要素になる。
fn extract_y_plane_u16(frame: &shiguredo_dav1d::DecodedFrame) -> Vec<u16> {
    let width = frame.width();
    let height = frame.height();
    let stride = frame.y_stride();
    let y_data = frame
        .y_plane_u16()
        .expect("10-bit デコード結果に u16 プレーンが必要");
    let mut y = Vec::with_capacity(width * height);
    for row in 0..height {
        let row_start = row * (stride / 2);
        y.extend_from_slice(&y_data[row_start..row_start + width]);
    }
    y
}

/// dav1d でデコードして (Y プレーン, 幅, 高さ) の一覧を返す
fn decode_with_dav1d(packets: &[Vec<u8>]) -> Vec<(Vec<u8>, usize, usize)> {
    let config = DecoderConfig::new();
    let mut decoder = Decoder::new(config).expect("dav1d デコーダーの生成に失敗");
    let mut decoded = Vec::new();

    for packet in packets {
        decoder.decode(packet).expect("デコードに失敗");
        while let Ok(Some(frame)) = decoder.next_frame() {
            decoded.push((extract_y_plane(&frame), frame.width(), frame.height()));
        }
    }

    decoder.finish().expect("finish に失敗");
    while let Ok(Some(frame)) = decoder.next_frame() {
        decoded.push((extract_y_plane(&frame), frame.width(), frame.height()));
    }

    decoded
}

// ============================================================================
// AOM エンコードヘルパー
// ============================================================================

/// AOM でエンコードしてフレーム単位のビットストリームを返す
fn encode_with_aom(
    config: shiguredo_aom::EncoderConfig,
    frames: &[(Vec<u8>, Vec<u8>, Vec<u8>)],
) -> Vec<Vec<u8>> {
    let mut encoder = shiguredo_aom::Encoder::new(config).expect("aom エンコーダーの生成に失敗");
    let options = shiguredo_aom::EncodeOptions {
        force_keyframe: false,
    };
    let mut packets = Vec::new();

    for (y, u, v) in frames {
        let image = shiguredo_aom::ImageData::I420 { y, u, v };
        encoder.encode(&image, &options).expect("エンコードに失敗");
        while let Some(encoded) = encoder.next_frame() {
            packets.push(
                encoded
                    .data()
                    .expect("エンコード済みデータの取得に失敗")
                    .to_vec(),
            );
        }
    }

    encoder.finish().expect("finish に失敗");
    while let Some(encoded) = encoder.next_frame() {
        packets.push(
            encoded
                .data()
                .expect("エンコード済みデータの取得に失敗")
                .to_vec(),
        );
    }

    packets
}

/// SVT-AV1 で 16-bit (ハイビット深度) 入力のエンコードをしてフレーム単位のビットストリームを返す
///
/// 10-bit エンコード時は `ColorFormat::I42010` でエンコーダーを生成し、
/// `FrameData::I42010` で入力する。プレーンはバイト列 (u16 のリトルエンディアン)
/// で渡すため、ここで変換する。
fn encode_with_svt_av1_16bit(
    config: shiguredo_svt_av1::EncoderConfig,
    frames: &[(Vec<u16>, Vec<u16>, Vec<u16>)],
) -> Vec<Vec<u8>> {
    let mut encoder =
        shiguredo_svt_av1::Encoder::new(config).expect("svt-av1 エンコーダーの生成に失敗");
    let options = shiguredo_svt_av1::EncodeOptions {
        force_keyframe: false,
    };
    let mut packets = Vec::new();

    for (y, u, v) in frames {
        let to_le_bytes = |plane: &[u16]| -> Vec<u8> {
            let mut bytes = Vec::with_capacity(plane.len() * 2);
            for value in plane {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            bytes
        };
        let y_bytes = to_le_bytes(y);
        let u_bytes = to_le_bytes(u);
        let v_bytes = to_le_bytes(v);
        let frame = shiguredo_svt_av1::FrameData::I42010 {
            y: &y_bytes,
            u: &u_bytes,
            v: &v_bytes,
        };
        encoder.encode(&frame, &options).expect("エンコードに失敗");
        while let Some(encoded) = encoder.next_frame() {
            packets.push(encoded.data().to_vec());
        }
    }

    encoder.finish().expect("finish に失敗");
    while let Some(encoded) = encoder.next_frame() {
        packets.push(encoded.data().to_vec());
    }

    packets
}

/// dav1d でデコードして (Y プレーン u16, 幅, 高さ) の一覧を返す (10-bit 用)
///
/// 各フレームが 10-bit (ハイビット深度) でデコードされ、Y/U/V すべての
/// u16 プレーンアクセサが有効であることを検証する。
fn decode_with_dav1d_16bit(packets: &[Vec<u8>]) -> Vec<(Vec<u16>, usize, usize)> {
    let config = DecoderConfig::new();
    let mut decoder = Decoder::new(config).expect("dav1d デコーダーの生成に失敗");
    let mut decoded = Vec::new();

    for packet in packets {
        decoder.decode(packet).expect("デコードに失敗");
        while let Ok(Some(frame)) = decoder.next_frame() {
            assert_eq!(frame.bit_depth(), 10, "10-bit でデコードされるべき");
            assert!(frame.is_high_depth(), "ハイビット深度であるべき");
            assert!(
                frame.u_plane_u16().is_some(),
                "u16 の U プレーンにアクセスできるべき"
            );
            assert!(
                frame.v_plane_u16().is_some(),
                "u16 の V プレーンにアクセスできるべき"
            );
            decoded.push((extract_y_plane_u16(&frame), frame.width(), frame.height()));
        }
    }

    decoder.finish().expect("finish に失敗");
    while let Ok(Some(frame)) = decoder.next_frame() {
        assert_eq!(frame.bit_depth(), 10, "10-bit でデコードされるべき");
        assert!(frame.is_high_depth(), "ハイビット深度であるべき");
        assert!(
            frame.u_plane_u16().is_some(),
            "u16 の U プレーンにアクセスできるべき"
        );
        assert!(
            frame.v_plane_u16().is_some(),
            "u16 の V プレーンにアクセスできるべき"
        );
        decoded.push((extract_y_plane_u16(&frame), frame.width(), frame.height()));
    }

    decoded
}

/// AOM エンコード → dav1d デコードのカラーバー PSNR 検証
fn roundtrip_colorbar_aom(
    config: shiguredo_aom::EncoderConfig,
    num_frames: usize,
    min_psnr_db: f64,
) {
    let width = config.g_w as usize;
    let height = config.g_h as usize;

    let (y, u, v) = generate_colorbar_i420(width, height);
    let input_frames: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> = (0..num_frames)
        .map(|_| (y.clone(), u.clone(), v.clone()))
        .collect();

    let packets = encode_with_aom(config, &input_frames);
    assert!(!packets.is_empty(), "エンコードされたパケットが空");

    let decoded_frames = decode_with_dav1d(&packets);
    assert_eq!(
        decoded_frames.len(),
        num_frames,
        "デコードされたフレーム数 {}, 期待値 {num_frames}",
        decoded_frames.len()
    );

    for (i, (decoded_y, w, h)) in decoded_frames.iter().enumerate() {
        assert_eq!(*w, width, "フレーム {i}: 幅が一致しない");
        assert_eq!(*h, height, "フレーム {i}: 高さが一致しない");
        let psnr = psnr_y(&y, decoded_y, width, height);
        assert!(
            psnr >= min_psnr_db,
            "フレーム {i}: PSNR {psnr:.1} dB が {min_psnr_db} dB 未満"
        );
    }
}

// ============================================================================
// SVT-AV1 エンコードヘルパー
// ============================================================================

/// SVT-AV1 でエンコードしてフレーム単位のビットストリームを返す
fn encode_with_svt_av1(
    config: shiguredo_svt_av1::EncoderConfig,
    frames: &[(Vec<u8>, Vec<u8>, Vec<u8>)],
) -> Vec<Vec<u8>> {
    let mut encoder =
        shiguredo_svt_av1::Encoder::new(config).expect("svt-av1 エンコーダーの生成に失敗");
    let options = shiguredo_svt_av1::EncodeOptions {
        force_keyframe: false,
    };
    let mut packets = Vec::new();

    for (y, u, v) in frames {
        let frame = shiguredo_svt_av1::FrameData::I420 { y, u, v };
        encoder.encode(&frame, &options).expect("エンコードに失敗");
        while let Some(encoded) = encoder.next_frame() {
            packets.push(encoded.data().to_vec());
        }
    }

    encoder.finish().expect("finish に失敗");
    while let Some(encoded) = encoder.next_frame() {
        packets.push(encoded.data().to_vec());
    }

    packets
}

/// SVT-AV1 エンコード → dav1d デコードのカラーバー PSNR 検証
fn roundtrip_colorbar_svt_av1(
    config: shiguredo_svt_av1::EncoderConfig,
    num_frames: usize,
    min_psnr_db: f64,
) {
    let width = config.width;
    let height = config.height;

    let (y, u, v) = generate_colorbar_i420(width, height);
    let input_frames: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> = (0..num_frames)
        .map(|_| (y.clone(), u.clone(), v.clone()))
        .collect();

    let packets = encode_with_svt_av1(config, &input_frames);
    assert!(!packets.is_empty(), "エンコードされたパケットが空");

    let decoded_frames = decode_with_dav1d(&packets);
    assert_eq!(
        decoded_frames.len(),
        num_frames,
        "デコードされたフレーム数 {}, 期待値 {num_frames}",
        decoded_frames.len()
    );

    for (i, (decoded_y, w, h)) in decoded_frames.iter().enumerate() {
        assert_eq!(*w, width, "フレーム {i}: 幅が一致しない");
        assert_eq!(*h, height, "フレーム {i}: 高さが一致しない");
        let psnr = psnr_y(&y, decoded_y, width, height);
        assert!(
            psnr >= min_psnr_db,
            "フレーム {i}: PSNR {psnr:.1} dB が {min_psnr_db} dB 未満"
        );
    }
}

// ============================================================================
// AOM エンコード → dav1d デコード: ラウンドトリップテスト
// ============================================================================

/// AOM Realtime CBR でダミーフレームのラウンドトリップ
#[test]
fn test_roundtrip_aom_dummy_frames() {
    let width: u32 = 320;
    let height: u32 = 240;
    let num_frames = 10;

    let mut config =
        shiguredo_aom::EncoderConfig::new(width, height, shiguredo_aom::ImageFormat::I420);
    config.g_usage = shiguredo_aom::Usage::Realtime;
    config.rc_end_usage = shiguredo_aom::RateControlMode::Cbr;
    config.rc_target_bitrate = 1000;
    config.cpu_used = Some(8);

    let input_frames: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> = (0..num_frames)
        .map(|i| generate_dummy_i420(width as usize, height as usize, i))
        .collect();

    let packets = encode_with_aom(config, &input_frames);
    assert!(!packets.is_empty(), "エンコードされたパケットが空");

    let decoded_frames = decode_with_dav1d(&packets);
    assert_eq!(decoded_frames.len(), num_frames);
    for (i, (y, w, h)) in decoded_frames.iter().enumerate() {
        assert_eq!(*w, width as usize, "フレーム {i}: 幅が一致しない");
        assert_eq!(*h, height as usize, "フレーム {i}: 高さが一致しない");
        assert!(!y.is_empty(), "フレーム {i}: Y プレーンが空");
    }
}

// ============================================================================
// AOM エンコード → dav1d デコード: PSNR テスト
// ============================================================================
//
// PSNR 閾値 (min_psnr_db = 50.0) の根拠:
// 実測ベースライン (macOS):
//   - aom Realtime CBR (320x240, 1000kbps): inf dB (MSE=0、完全再構成)
//   - aom GoodQuality VBR (320x240, 1000kbps): inf dB
//   - aom AllIntra Q (320x240, cq_level=30): 55.6 dB
//   - svt-av1 VBR (320x240, 1Mbps): 70.6 dB
//   - svt-av1 CRF (320x240, qp=35): 63.6 dB
// プラットフォーム間の SIMD 実装差による丸め順の違いを考慮して、
// ベースラインから 5 dB 以上の余裕を持つ 50.0 dB を閾値にしている。

/// AOM Realtime CBR カラーバーの PSNR 検証
#[test]
fn test_psnr_aom_realtime_cbr() {
    let mut config = shiguredo_aom::EncoderConfig::new(320, 240, shiguredo_aom::ImageFormat::I420);
    config.g_usage = shiguredo_aom::Usage::Realtime;
    config.rc_end_usage = shiguredo_aom::RateControlMode::Cbr;
    config.rc_target_bitrate = 1000;
    config.cpu_used = Some(8);

    roundtrip_colorbar_aom(config, 30, 50.0);
}

/// AOM GoodQuality VBR カラーバーの PSNR 検証
#[test]
fn test_psnr_aom_good_quality_vbr() {
    let mut config = shiguredo_aom::EncoderConfig::new(320, 240, shiguredo_aom::ImageFormat::I420);
    config.g_usage = shiguredo_aom::Usage::GoodQuality;
    config.rc_end_usage = shiguredo_aom::RateControlMode::Vbr;
    config.rc_target_bitrate = 1000;
    config.cpu_used = Some(8);
    config.g_lag_in_frames = Some(0);

    roundtrip_colorbar_aom(config, 10, 50.0);
}

/// AOM AllIntra Q カラーバーの PSNR 検証
#[test]
fn test_psnr_aom_all_intra_q() {
    let mut config = shiguredo_aom::EncoderConfig::new(320, 240, shiguredo_aom::ImageFormat::I420);
    config.g_usage = shiguredo_aom::Usage::AllIntra;
    config.rc_end_usage = shiguredo_aom::RateControlMode::Q;
    config.rc_target_bitrate = 1000;
    config.cpu_used = Some(8);
    config.cq_level = Some(30);

    roundtrip_colorbar_aom(config, 5, 50.0);
}

// ============================================================================
// SVT-AV1 エンコード → dav1d デコード: ラウンドトリップテスト
// ============================================================================

/// SVT-AV1 の共通エンコーダー設定を返す
///
/// SVT-AV1 は look_ahead_distance がデフォルトで大きいため、
/// テストでは fps_numerator=1 で低遅延にし、enc_mode=13 で最速にする。
fn svt_av1_encoder_config(width: usize, height: usize) -> shiguredo_svt_av1::EncoderConfig {
    let mut config =
        shiguredo_svt_av1::EncoderConfig::new(width, height, shiguredo_svt_av1::ColorFormat::I420);
    config.target_bit_rate = 1_000_000;
    config.fps_numerator = 1;
    config.fps_denominator = 1;
    config.enc_mode = 13;
    config
}

/// SVT-AV1 VBR でダミーフレームのラウンドトリップ
#[test]
fn test_roundtrip_svt_av1_dummy_frames() {
    let width = 320;
    let height = 240;
    let num_frames = 5;

    let config = svt_av1_encoder_config(width, height);

    let input_frames: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> = (0..num_frames)
        .map(|i| generate_dummy_i420(width, height, i))
        .collect();

    let packets = encode_with_svt_av1(config, &input_frames);
    assert!(!packets.is_empty(), "エンコードされたパケットが空");

    let decoded_frames = decode_with_dav1d(&packets);
    assert_eq!(decoded_frames.len(), num_frames);
    for (i, (y, w, h)) in decoded_frames.iter().enumerate() {
        assert_eq!(*w, width, "フレーム {i}: 幅が一致しない");
        assert_eq!(*h, height, "フレーム {i}: 高さが一致しない");
        assert!(!y.is_empty(), "フレーム {i}: Y プレーンが空");
    }
}

// ============================================================================
// SVT-AV1 エンコード → dav1d デコード: PSNR テスト
// ============================================================================

/// SVT-AV1 VBR カラーバーの PSNR 検証
#[test]
fn test_psnr_svt_av1_vbr() {
    let mut config = svt_av1_encoder_config(320, 240);
    config.rate_control_mode = shiguredo_svt_av1::RcMode::Vbr;

    roundtrip_colorbar_svt_av1(config, 5, 50.0);
}

/// SVT-AV1 CRF カラーバーの PSNR 検証
#[test]
fn test_psnr_svt_av1_crf() {
    let mut config = svt_av1_encoder_config(320, 240);
    config.rate_control_mode = shiguredo_svt_av1::RcMode::CqpOrCrf;
    config.target_bit_rate = 0;
    config.qp = Some(35);

    roundtrip_colorbar_svt_av1(config, 5, 50.0);
}

// ============================================================================
// EAGAIN 経路のテスト
// ============================================================================

/// decode() が EAGAIN を返したときにデータが破棄されることと、
/// フレームを取り出すと回復することを検証する
///
/// dav1d は前回送信したデータが入力バッファに残っている場合に EAGAIN を返す。
/// 複数フレームを連結したバッファを 1 回で渡すと 1 フレーム分しか消費されず、
/// 次の decode() が EAGAIN になる (n_threads=1 なら決定的)。
/// EAGAIN で渡したデータは破棄されるため、フレームを取り出してから
/// 同じデータを再度渡す必要がある
#[test]
fn test_decode_eagain_retry() {
    let width: u32 = 320;
    let height: u32 = 240;
    let num_frames = 2;

    let mut config =
        shiguredo_aom::EncoderConfig::new(width, height, shiguredo_aom::ImageFormat::I420);
    config.g_usage = shiguredo_aom::Usage::Realtime;
    config.rc_end_usage = shiguredo_aom::RateControlMode::Cbr;
    config.rc_target_bitrate = 1000;
    config.cpu_used = Some(8);

    let input_frames: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> = (0..num_frames)
        .map(|i| generate_dummy_i420(width as usize, height as usize, i))
        .collect();

    // 2 フレームを連結して 1 バッファにする
    let packets = encode_with_aom(config, &input_frames);
    let mut combined = Vec::new();
    for packet in &packets {
        combined.extend_from_slice(packet);
    }

    let config = DecoderConfig::new();
    let mut decoder = Decoder::new(config).expect("dav1d デコーダーの生成に失敗");

    // 1 フレーム分しか消費されず、残りが内部バッファに残る
    decoder.decode(&combined).expect("デコードに失敗");

    // 内部バッファにデータが残っているため EAGAIN が返り、データは破棄される
    let err = decoder.decode(&combined).expect_err("EAGAIN が返るべき");
    assert!(err.is_eagain(), "EAGAIN であるべき");

    // フレームを取り出すと内部バッファが空になり、再送できるようになる
    let mut decoded = Vec::new();
    while let Ok(Some(frame)) = decoder.next_frame() {
        decoded.push((extract_y_plane(&frame), frame.width(), frame.height()));
    }
    decoder.finish().expect("finish に失敗");
    while let Ok(Some(frame)) = decoder.next_frame() {
        decoded.push((extract_y_plane(&frame), frame.width(), frame.height()));
    }

    assert_eq!(
        decoded.len(),
        num_frames,
        "デコードされたフレーム数 {}, 期待値 {num_frames}",
        decoded.len()
    );
    // 2 フレームの内容が異なること (重複出力のバグを検出する)
    assert_ne!(
        decoded[0].0, decoded[1].0,
        "フレームが重複して出力されている"
    );
    for (y, w, h) in &decoded {
        assert_eq!(*w, width as usize, "フレームの幅が一致しない");
        assert_eq!(*h, height as usize, "フレームの高さが一致しない");
        assert!(!y.is_empty(), "Y プレーンが空");
    }
}

// ============================================================================
// ハイビット深度 (10-bit) デコードのテスト
// ============================================================================
//
// PSNR 閾値 (min_psnr_db = 50.0) の根拠:
// 実測ベースライン (macOS):
//   - aom AllIntra Q (320x240, cq_level=30, 10-bit): 66.0 dB
// 8-bit テストと同じく、プラットフォーム間の SIMD 実装差による丸め順の違いを
// 考慮して 50.0 dB を閾値にしている。

/// 10-bit エンコード (SVT-AV1) → dav1d デコードで u16 プレーンを検証する
///
/// `ColorFormat::I42010` でエンコードしたビットストリームをデコードし、
/// ビット深度 10・ハイビット深度フラグ・u16 プレーンの画素値 (PSNR) を検証する。
/// 8-bit のデコード結果と違い、u16 プレーンアクセサが `Some` を返すことを確認する。
///
/// エンコーダーの選定: AOM (shiguredo_aom) は `aom_codec_enc_init_ver` に
/// `AOM_CODEC_USE_HIGHBITDEPTH` フラグを渡さないため 10-bit エンコードに対応していない。
/// SVT-AV1 (shiguredo_svt_av1) は `ColorFormat::I42010` で 10-bit エンコードできるため、
/// こちらを使用する。
///
/// PSNR 閾値 (min_psnr_db = 50.0) の根拠:
/// 実測ベースライン (macOS):
///   - svt-av1 VBR (320x240, 1Mbps, 10-bit): 71.4 dB
///
/// 8-bit テストと同じく、プラットフォーム間の SIMD 実装差による丸め順の違いを
/// 考慮して 50.0 dB を閾値にしている。
#[test]
fn test_decode_10bit_high_depth() {
    let width = 320;
    let height = 240;

    let mut config = shiguredo_svt_av1::EncoderConfig::new(
        width,
        height,
        shiguredo_svt_av1::ColorFormat::I42010,
    );
    config.target_bit_rate = 1_000_000;
    config.fps_numerator = 1;
    config.fps_denominator = 1;
    config.enc_mode = 13;

    let (y, u, v) = generate_colorbar_i420_16bit(width, height);
    let input_frames = vec![(y.clone(), u.clone(), v.clone())];

    let packets = encode_with_svt_av1_16bit(config, &input_frames);
    assert!(!packets.is_empty(), "エンコードされたパケットが空");

    let decoded_frames = decode_with_dav1d_16bit(&packets);
    assert_eq!(
        decoded_frames.len(),
        1,
        "デコードされたフレーム数 {}, 期待値 1",
        decoded_frames.len()
    );

    let (decoded_y, w, h) = &decoded_frames[0];
    assert_eq!(*w, width, "幅が一致しない");
    assert_eq!(*h, height, "高さが一致しない");
    let psnr = psnr_y_u16(&y, decoded_y, width, height);
    assert!(psnr >= 50.0, "PSNR {psnr:.1} dB が 50.0 dB 未満");
}

// ============================================================================
// モノクロ (I400) デコードのテスト
// ============================================================================

/// AOM の monochrome 設定でエンコード → dav1d デコードで I400 を検証する
///
/// I400 ではクロマプレーンが存在しないため、`pixel_layout()` が `I400` になり、
/// `u_plane()` / `v_plane()` が空のスライスを返すことを確認する。
#[test]
fn test_decode_monochrome_i400() {
    let width: u32 = 320;
    let height: u32 = 240;

    let mut config =
        shiguredo_aom::EncoderConfig::new(width, height, shiguredo_aom::ImageFormat::I420);
    config.g_usage = shiguredo_aom::Usage::AllIntra;
    config.rc_end_usage = shiguredo_aom::RateControlMode::Q;
    config.rc_target_bitrate = 1000;
    config.cpu_used = Some(8);
    config.cq_level = Some(30);
    config.monochrome = Some(true);

    let (y, u, v) = generate_colorbar_i420(width as usize, height as usize);
    let input_frames = vec![(y.clone(), u, v)];

    let packets = encode_with_aom(config, &input_frames);
    assert!(!packets.is_empty(), "エンコードされたパケットが空");

    let config = DecoderConfig::new();
    let mut decoder = Decoder::new(config).expect("dav1d デコーダーの生成に失敗");
    let mut frames = Vec::new();
    for packet in &packets {
        decoder.decode(packet).expect("デコードに失敗");
        while let Ok(Some(frame)) = decoder.next_frame() {
            frames.push(frame);
        }
    }
    decoder.finish().expect("finish に失敗");
    while let Ok(Some(frame)) = decoder.next_frame() {
        frames.push(frame);
    }

    assert_eq!(
        frames.len(),
        1,
        "デコードされたフレーム数 {}, 期待値 1",
        frames.len()
    );
    let frame = &frames[0];
    assert_eq!(
        frame.pixel_layout(),
        PixelLayout::I400,
        "I400 レイアウトであるべき"
    );
    assert_eq!(frame.width(), width as usize, "幅が一致しない");
    assert_eq!(frame.height(), height as usize, "高さが一致しない");
    assert_eq!(frame.bit_depth(), 8, "8-bit であるべき");
    assert!(!frame.is_high_depth(), "ハイビット深度ではないべき");
    assert!(
        frame.u_plane().is_empty(),
        "I400 では U プレーンが空であるべき"
    );
    assert!(
        frame.v_plane().is_empty(),
        "I400 では V プレーンが空であるべき"
    );
    assert!(
        frame.u_plane_u16().is_none(),
        "I400 では u16 の U プレーンは None であるべき"
    );
    assert!(
        frame.v_plane_u16().is_none(),
        "I400 では u16 の V プレーンは None であるべき"
    );

    let decoded_y = extract_y_plane(frame);
    let psnr = psnr_y(&y, &decoded_y, width as usize, height as usize);
    assert!(psnr >= 50.0, "PSNR {psnr:.1} dB が 50.0 dB 未満");
}
