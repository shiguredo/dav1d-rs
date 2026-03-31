Created: 2026-03-19
Completed: 2026-03-19

# DecodedFrame に色空間・フレーム種別のアクセサを追加する

Model: Opus 4.6

## 概要

dav1d の `Dav1dPicture` にはシーケンスヘッダー (`seq_hdr`) と
フレームヘッダー (`frame_hdr`) へのポインタが含まれており、
色空間情報やフレーム種別を取得できる。
現在の `DecodedFrame` はピクセルデータのアクセサしか持っていない。

## 追加すべきアクセサ

### フレームヘッダーから取得

- `frame_type() -> FrameType` — フレーム種別 (Key, Inter, Intra, Switch)
- `temporal_id() -> u8` — SVC 用テンポラル ID
- `spatial_id() -> u8` — SVC 用スパーシャル ID
- `show_frame() -> bool` — 表示フレームかどうか

### シーケンスヘッダーから取得

- `color_primaries() -> ColorPrimaries`
- `transfer_characteristics() -> TransferCharacteristics`
- `matrix_coefficients() -> MatrixCoefficients`
- `chroma_sample_position() -> ChromaSamplePosition`
- `color_range() -> ColorRange`
- `profile() -> u8` — AV1 プロファイル (0, 1, 2)

### FrameType enum

- `Key` (0)
- `Inter` (1)
- `Intra` (2)
- `Switch` (3)

## 前提 issue

- #0002 (色空間メタデータ用 enum)

## 根拠

フレーム種別の判定はキーフレーム検出やストリーム分析に必要。
色空間情報はデコード後の画像処理パイプラインで正しい色変換を行うために不可欠。
dav1d がデコード結果に含めている情報をラッパーが隠す理由がない。

## 解決方法

`FrameType` enum を定義し、`DecodedFrame` にフレームヘッダー (`frame_type`, `temporal_id`, `spatial_id`, `show_frame`) と
シーケンスヘッダー (`color_primaries`, `transfer_characteristics`, `matrix_coefficients`, `chroma_sample_position`, `color_range`, `profile`) のアクセサを追加した。
