# 公開 API のテストを追加し、EAGAIN 経路とハイビット深度を検証する

- Created: 2026-08-02
- Completed: {YYYY-MM-DD}
- Branch: feature/add-missing-tests
- Polished: {YYYY-MM-DD}
- Reporter: @voluntas

## 目的

FFI バインディングの中核経路（EAGAIN・ハイビット深度・メタデータアクセサ）と任意入力の panic 耐性をテストで担保し、デコーダの検出力が弱い現行テストを改善する。

## 現状

- 公開 API の大半が一度も呼び出されていない（`version()` / `version_api()` / `Error::is_eagain()` / `Error` の Display / `DecoderConfig::frame_delay()` / `Decoder::apply_grain()` / `Decoder::flush()` / `Decoder::get_event_flags()` / `Decoder::get_decode_error_data_props()` / `y_plane_u16()` / `u_plane_u16()` / `v_plane_u16()` / `u_stride()` / `v_stride()` / SVC ID / 色空間メタデータ 5 種 / HDR 2 種 / シーケンスヘッダーのエラーパス）
- EAGAIN 経路（`Decoder::decode()` が EAGAIN を返すパスと、`next_frame()` の EAGAIN → `Ok(None)` 変換）がテストで一度も実行されない。デフォルト設定（`n_threads = 1`, `max_frame_delay = 0`）では `decode_with_dav1d` の「毎パケット drain」パターンのため EAGAIN が構造的に発生しない
- ハイビット深度（10-bit / 12-bit）と I400 / I422 / I444 レイアウトのデコードテストがなく、u16 プレーンアクセサ群が検証不可能な状態
- `parse_sequence_header()` のエラーパス（空入力・シーケンスヘッダー不在・未知 hbd）が未テスト
- `tests/test_psnr.rs` の PSNR 閾値 25.0 dB が根拠なく緩い。25 dB は平均画素誤差約 14 レベル相当で、全画素を ±15 レベルずらすデコーダのバグを検出できない。各設定の期待品質帯は 33 〜 45 dB 程度
- `tests/test_psnr.rs` のラウンドトリップテストは全フレーム同一画像のため、フレーム順序・重複・スタックのバグを検出できない（`generate_dummy_i420` はフレームごとに内容が変わるのに、検証は「非空」のみ）
- `decode_black` は「black」と名乗るのに画素値（Y=16, UV=128）を検証していない
- pbt/（proptest）と fuzz/（cargo-fuzz）が存在しない（時雨堂 Rust 規約違反。PBT は `pbt/tests/prop_<module>.rs`、Fuzzing は `fuzz/`）

## 設計方針

- 時雨堂 Rust 規約に従い、単体テスト（`tests/` と `src/` 内 `#[cfg(test)]`）+ PBT（proptest）+ Fuzzing（cargo-fuzz）の役割分担で補う
- 単体テストは意図的なエラーパス・境界値、PBT は代数的不変条件、Fuzzing は任意入力の panic 耐性を担当する

## 完了条件

- 公開 API の主要経路（`version()` / `Error::is_eagain()` / `Display` / `frame_delay()` / `apply_grain()` / `flush()` / `get_event_flags()` / `get_decode_error_data_props()` / u16 プレーン 3 種 / 10-bit デコード / I400 / EAGAIN 再試行 / `parse_sequence_header()` エラーパス）がテストされること
- EAGAIN を意図的に発生させるテスト（`max_frame_delay` を 1 にして 2 フレームを連続 `decode()` する等）が追加されること
- PSNR 閾値が実測ベースラインをコメントで示した上で見直されること
- `decode_black` が画素値（Y=16, UV=128）と寸法を検証すること
- pbt/ と fuzz/ が導入され、`parse_sequence_header()` と `decode()` の任意バイト列に対する panic 耐性が検証されること

## 解決方法

- `src/lib.rs` 内の `#[cfg(test)]` モジュールに、エラーパスと境界値の単体テストを追加する（`parse_sequence_header(&[])` の Err、`decode(&[])` の `Ok(())`、`Error::is_eagain()`、Display、`DecoderConfig::frame_delay()` の不変条件）
- `tests/test_psnr.rs` に EAGAIN 再試行テストと、`max_frame_delay` / `decode_frame_type` / `inloop_filters` の設定反映テストを追加する
- 10-bit エンコード（AOM の high bit depth 設定）→ dav1d デコードで u16 プレーンを検証するテストを追加する
- `decode_black` で画素値と寸法を検証し、テストベクタの由来コメントを追加する
- PSNR 閾値を見直し、実測ベースラインと余裕の根拠をコメントで明記する
- `pbt/tests/prop_<module>.rs` に proptest で `InloopFilterType` の BitOr 代数（結合則・交換則・単位元）と `EventFlags::contains` の代数を追加する
- `fuzz/` に cargo-fuzz のターゲット（`parse_sequence_header` / `decode` の任意バイト列）を追加する
