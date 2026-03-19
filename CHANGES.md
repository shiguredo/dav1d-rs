# 変更履歴

- UPDATE
  - 後方互換がある変更
- ADD
  - 後方互換がある追加
- CHANGE
  - 後方互換のない変更
- FIX
  - バグ修正

## develop

- [UPDATE] dav1d を 1.5.1 から 1.5.3 に更新する
  - @voluntas
- [ADD] `PixelLayout` enum を追加する (I400, I420, I422, I444)
  - @voluntas
- [ADD] `DecodedFrame::pixel_layout()` でフレームのピクセルレイアウトを取得できるようにする
  - @voluntas
- [ADD] `DecodedFrame::bit_depth()` でフレームのビット深度を取得できるようにする
  - @voluntas
- [ADD] `DecodedFrame::is_high_depth()` でハイビット深度 (10-bit 以上) の判定ができるようにする
  - @voluntas
- [ADD] シンボル名書き換え機能を追加する
  - 他のライブラリとのシンボル衝突を回避するため、静的ライブラリ内の全シンボルに
    `shiguredo_dav1d` プレフィックスを付与する
  - llvm-nm / llvm-objcopy を使用してビルド時に自動書き換えを行う
  - @voluntas
- [ADD] prebuilt バイナリダウンロード機能を追加する
  - `source-build` feature でソースからのビルドに切り替え可能にする
  - デフォルトでは prebuilt バイナリを使用する
  - @voluntas
- [ADD] `ColorPrimaries` enum を追加する (H.273 準拠の色域)
  - @voluntas
- [ADD] `TransferCharacteristics` enum を追加する (H.273 準拠の伝達特性)
  - @voluntas
- [ADD] `MatrixCoefficients` enum を追加する (H.273 準拠の行列係数)
  - @voluntas
- [ADD] `ChromaSamplePosition` enum を追加する
  - @voluntas
- [ADD] `ColorRange` enum を追加する (Studio / Full)
  - @voluntas
- [ADD] `FrameType` enum を追加する (Key, Inter, Intra, Switch)
  - @voluntas
- [ADD] `DecodedFrame::frame_type()` でフレーム種別を取得できるようにする
  - @voluntas
- [ADD] `DecodedFrame::temporal_id()` / `spatial_id()` で SVC 用 ID を取得できるようにする
  - @voluntas
- [ADD] `DecodedFrame::show_frame()` で表示フレーム判定ができるようにする
  - @voluntas
- [ADD] `DecodedFrame` に色空間メタデータのアクセサを追加する
  - `color_primaries()`, `transfer_characteristics()`, `matrix_coefficients()`,
    `chroma_sample_position()`, `color_range()`, `profile()`
  - @voluntas
- [ADD] `ContentLightLevel` 構造体を追加する (HDR メタデータ)
  - @voluntas
- [ADD] `MasteringDisplay` 構造体を追加する (HDR メタデータ)
  - @voluntas
- [ADD] `DecodedFrame::content_light_level()` で HDR コンテンツライトレベルを取得できるようにする
  - @voluntas
- [ADD] `DecodedFrame::mastering_display()` で HDR マスタリングディスプレイ情報を取得できるようにする
  - @voluntas
- [ADD] `SequenceHeader` 構造体を追加する
  - @voluntas
- [ADD] `parse_sequence_header()` でデコーダーなしにシーケンスヘッダーを解析できるようにする
  - @voluntas
- [ADD] `version()` / `version_api()` でリンクされている dav1d のランタイムバージョンを取得できるようにする
  - @voluntas
- [ADD] `DecodedFrame::y_plane_u16()` / `u_plane_u16()` / `v_plane_u16()` でハイビット深度のプレーンデータを `&[u16]` として取得できるようにする
  - @voluntas
- [ADD] `DataProps` 構造体を追加する
  - @voluntas
- [ADD] `Decoder::get_decode_error_data_props()` でデコードエラーの詳細を取得できるようにする
  - @voluntas
- [ADD] `DecoderConfig::frame_delay()` でデコーダーのフレーム遅延を取得できるようにする
  - @voluntas
- [ADD] `EventFlags` ビットフラグ型を追加する
  - @voluntas
- [ADD] `Decoder::get_event_flags()` でデコードイベントを取得できるようにする
  - @voluntas
- [ADD] `Decoder::apply_grain()` でデコード済みフレームにフィルムグレインを適用できるようにする
  - @voluntas
- [ADD] `Decoder::flush()` でデコーダーの内部状態をリセットできるようにする
  - @voluntas
- [ADD] `InloopFilterType` ビットフラグ型を追加する
  - @voluntas
- [ADD] `DecodeFrameType` enum を追加する
  - @voluntas
- [ADD] `DecoderConfig` に dav1d の全設定項目を追加する
  - `max_frame_delay`, `apply_grain`, `operating_point`, `all_layers`,
    `frame_size_limit`, `strict_std_compliance`, `output_invisible_frames`,
    `inloop_filters`, `decode_frame_type`
  - @voluntas
- [CHANGE] `Decoder::new()` を `Decoder::new(DecoderConfig)` に変更する
  - `DecoderConfig` でスレッド数などのデコーダー設定を指定できるようにする
  - @voluntas
- [CHANGE] `DecodedFrame` の I420 固定の制約を除去する
  - I400, I420, I422, I444 の全ピクセルレイアウトに対応する
  - `u_plane()` / `v_plane()` は I400 の場合に空のスライスを返す
  - @voluntas

### misc

- hisui のサブクレートから独立リポジトリに分離する
  - @voluntas
- build.rs で利用する toml crate を `toml` から `shiguredo_toml` に変更する
  - @voluntas

## 2025.1.0

**リリース日**: 2025-09-26
