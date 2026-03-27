use shiguredo_dav1d::{Decoder, DecoderConfig};

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

/// dav1d でデコードして (Y プレーン, 幅, 高さ) の一覧を返す
fn decode_with_dav1d(packets: &[Vec<u8>]) -> Vec<(Vec<u8>, usize, usize)> {
    let config = DecoderConfig::new();
    let mut decoder = Decoder::new(config).expect("failed to create dav1d decoder");
    let mut decoded = Vec::new();

    for packet in packets {
        decoder.decode(packet).expect("failed to decode");
        while let Ok(Some(frame)) = decoder.next_frame() {
            decoded.push((extract_y_plane(&frame), frame.width(), frame.height()));
        }
    }

    decoder.finish().expect("failed to finish");
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
    let mut encoder = shiguredo_aom::Encoder::new(config).expect("failed to create aom encoder");
    let options = shiguredo_aom::EncodeOptions {
        force_keyframe: false,
    };
    let mut packets = Vec::new();

    for (y, u, v) in frames {
        let image = shiguredo_aom::ImageData::I420 { y, u, v };
        encoder.encode(&image, &options).expect("failed to encode");
        while let Some(encoded) = encoder.next_frame() {
            packets.push(encoded.data().to_vec());
        }
    }

    encoder.finish().expect("failed to finish");
    while let Some(encoded) = encoder.next_frame() {
        packets.push(encoded.data().to_vec());
    }

    packets
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
    assert!(!packets.is_empty(), "no encoded packets");

    let decoded_frames = decode_with_dav1d(&packets);
    assert_eq!(
        decoded_frames.len(),
        num_frames,
        "decoded {} frames, expected {num_frames}",
        decoded_frames.len()
    );

    for (i, (decoded_y, w, h)) in decoded_frames.iter().enumerate() {
        assert_eq!(*w, width, "frame {i}: width mismatch");
        assert_eq!(*h, height, "frame {i}: height mismatch");
        let psnr = psnr_y(&y, decoded_y, width, height);
        assert!(
            psnr >= min_psnr_db,
            "frame {i}: PSNR {psnr:.1} dB < {min_psnr_db} dB"
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
        shiguredo_svt_av1::Encoder::new(config).expect("failed to create svt-av1 encoder");
    let options = shiguredo_svt_av1::EncodeOptions {
        force_keyframe: false,
    };
    let mut packets = Vec::new();

    for (y, u, v) in frames {
        let frame = shiguredo_svt_av1::FrameData::I420 { y, u, v };
        encoder.encode(&frame, &options).expect("failed to encode");
        while let Some(encoded) = encoder.next_frame() {
            packets.push(encoded.data().to_vec());
        }
    }

    encoder.finish().expect("failed to finish");
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
    assert!(!packets.is_empty(), "no encoded packets");

    let decoded_frames = decode_with_dav1d(&packets);
    assert_eq!(
        decoded_frames.len(),
        num_frames,
        "decoded {} frames, expected {num_frames}",
        decoded_frames.len()
    );

    for (i, (decoded_y, w, h)) in decoded_frames.iter().enumerate() {
        assert_eq!(*w, width, "frame {i}: width mismatch");
        assert_eq!(*h, height, "frame {i}: height mismatch");
        let psnr = psnr_y(&y, decoded_y, width, height);
        assert!(
            psnr >= min_psnr_db,
            "frame {i}: PSNR {psnr:.1} dB < {min_psnr_db} dB"
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
    assert!(!packets.is_empty(), "no encoded packets");

    let decoded_frames = decode_with_dav1d(&packets);
    assert_eq!(decoded_frames.len(), num_frames);
    for (i, (y, w, h)) in decoded_frames.iter().enumerate() {
        assert_eq!(*w, width as usize, "frame {i}: width mismatch");
        assert_eq!(*h, height as usize, "frame {i}: height mismatch");
        assert!(!y.is_empty(), "frame {i}: empty Y plane");
    }
}

// ============================================================================
// AOM エンコード → dav1d デコード: PSNR テスト
// ============================================================================

/// AOM Realtime CBR カラーバーの PSNR 検証
#[test]
fn test_psnr_aom_realtime_cbr() {
    let mut config = shiguredo_aom::EncoderConfig::new(320, 240, shiguredo_aom::ImageFormat::I420);
    config.g_usage = shiguredo_aom::Usage::Realtime;
    config.rc_end_usage = shiguredo_aom::RateControlMode::Cbr;
    config.rc_target_bitrate = 1000;
    config.cpu_used = Some(8);

    roundtrip_colorbar_aom(config, 30, 25.0);
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

    roundtrip_colorbar_aom(config, 10, 25.0);
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

    roundtrip_colorbar_aom(config, 5, 25.0);
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
    assert!(!packets.is_empty(), "no encoded packets");

    let decoded_frames = decode_with_dav1d(&packets);
    assert_eq!(decoded_frames.len(), num_frames);
    for (i, (y, w, h)) in decoded_frames.iter().enumerate() {
        assert_eq!(*w, width, "frame {i}: width mismatch");
        assert_eq!(*h, height, "frame {i}: height mismatch");
        assert!(!y.is_empty(), "frame {i}: empty Y plane");
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

    roundtrip_colorbar_svt_av1(config, 5, 25.0);
}

/// SVT-AV1 CRF カラーバーの PSNR 検証
#[test]
fn test_psnr_svt_av1_crf() {
    let mut config = svt_av1_encoder_config(320, 240);
    config.rate_control_mode = shiguredo_svt_av1::RcMode::CqpOrCrf;
    config.target_bit_rate = 0;
    config.qp = Some(35);

    roundtrip_colorbar_svt_av1(config, 5, 25.0);
}
