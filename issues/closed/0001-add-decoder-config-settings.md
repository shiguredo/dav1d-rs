Created: 2026-03-19
Completed: 2026-03-19

# DecoderConfig に dav1d の全設定項目を追加する

Model: Opus 4.6

## 概要

現在の `DecoderConfig` は `n_threads` のみ公開しているが、
dav1d の `Dav1dSettings` には他にも多数の設定項目がある。
デコーダーの動作を細かく制御するために、これらを公開する必要がある。

## 追加すべきフィールド

- `max_frame_delay: usize` — 低遅延デコード用 (デフォルト: 0、dav1d が自動決定)
- `apply_grain: bool` — フィルムグレインの適用有無 (デフォルト: true)
- `operating_point: usize` — スケーラブル AV1 ビットストリームのオペレーティングポイント選択 (0-31、デフォルト: 0)
- `all_layers: bool` — スケーラブル AV1 の全空間レイヤーを出力するか (デフォルト: true)
- `frame_size_limit: Option<u32>` — 最大フレームサイズ制限 (ピクセル単位、None で無制限)
- `strict_std_compliance: bool` — ビットストリーム規格違反時にデコードを中断するか (デフォルト: false)
- `output_invisible_frames: bool` — 非表示フレームも出力するか (デフォルト: false)
- `inloop_filters: InloopFilterType` — 有効にするポストフィルター (デフォルト: All)
- `decode_frame_type: DecodeFrameType` — デコードするフレーム種別 (デフォルト: All)

## 必要な enum

### InloopFilterType

dav1d のインループフィルターはビットフラグで制御される。

- `None` — フィルターなし (0)
- `Deblock` — デブロッキングフィルター (1)
- `Cdef` — CDEF フィルター (2)
- `Restoration` — ループリストレーションフィルター (4)
- `All` — 全フィルター有効 (7)

### DecodeFrameType

- `All` — 全フレームをデコード (0)
- `Reference` — 参照フレームのみ (1)
- `Intra` — イントラフレームのみ (2)
- `Key` — キーフレームのみ (3)

## 根拠

Hisui では現在シングルスレッドで使用しているが、
`apply_grain` や `inloop_filters` は性能チューニングに直結する。
また `max_frame_delay` は低遅延ストリーミングで重要な設定であり、
`decode_frame_type` はサムネイル生成のようなユースケースで有用。
dav1d が提供する設定を隠す理由がない。

## 解決方法

`DecoderConfig` に全フィールドを追加し、`InloopFilterType` と `DecodeFrameType` enum を定義した。
`Decoder::new()` で各フィールドを `Dav1dSettings` に反映するようにした。
