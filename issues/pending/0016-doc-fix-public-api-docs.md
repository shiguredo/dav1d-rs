# 公開 API のドキュメントを実装と dav1d 仕様に一致させる

- Created: 2026-08-02
- Completed: {YYYY-MM-DD}
- Branch: feature/fix-public-api-docs
- Polished: {YYYY-MM-DD}
- Reporter: @voluntas

## 目的

README と `src/lib.rs` の doc コメントの誤り・情報欠落を修正し、API 利用者が誤った情報に基づいてコードを書かないようにする。

## 現状

- `README.md` の `DecodedFrame` メソッド表（`frame_type()` / `temporal_id()` / `spatial_id()` / `show_frame()` / `color_primaries()` / `transfer_characteristics()` / `matrix_coefficients()` / `chroma_sample_position()` / `color_range()` / `profile()` の 10 行）が、戻り値の `Option` 化（CHANGES.md の 2026.1.0 [CHANGE]）を反映していない。実装は全て `Option` を返す
- `Decoder::decode()` の doc に、EAGAIN を返した際に渡したデータが破棄されることが未記載。ユーザーは同じデータを再度渡し直す必要があるが、「再度呼び出すこと」としか書かれておらず、データが失われることが読み取れない（フレーム欠落の起点）
- `DecoderConfig::frame_delay()` の doc「戻り値は 1 以上 `max_frame_delay` 以下であることが保証される」が、デフォルト設定（`max_frame_delay = 0`）で成立しない（dav1d 実装では 0 のとき 1 〜 8 が返る）
- `DecoderConfig` の各フィールド doc が dav1d.h の限定情報を欠落している:
  - `n_threads`: dav1d では 0 が「論理コア数で自動決定」だが未記載
  - `strict_std_compliance`: dav1d.h は「実際のデコードに影響しない規格違反（不整合・無効なメタデータ）のみ中断」と限定しているが、doc は「ビットストリーム規格違反時にデコードを中断するか」のみ
  - `output_invisible_frames`: dav1d.h は「符号化順で出力」「show-existing-frame により同じフレームが 2 回出現しうる」と明記しているが、doc は「非表示フレームも出力するか」のみ
  - `max_frame_delay`: dav1d.h の「1 で低遅延デコード」という推奨が未記載
- `Decoder::apply_grain()` の doc に、`DecoderConfig::apply_grain` が true（デフォルト）のまま呼ぶとフィルムグレインが二重適用されるという dav1d.h の警告が未記載
- `DecodeFrameType::Intra` の説明「イントラフレームのみ」に、dav1d.h の「キーフレームを含む」が欠落
- `parse_sequence_header()` の doc に、複数シーケンスヘッダー OBU がある場合「最後の 1 つだけが返る」ことと、シーケンスヘッダーが無い場合に ENOENT エラーが返ることが未記載
- `EventFlags::NEW_SEQUENCE` の説明に、dav1d.h の「最後に返されたピクチャに紐づく」「flush 直後のピクチャにも立つ」という限定が欠落
- `DecodedFrame::v_stride()` に `# Panics` セクションがない（実装は `u_stride()` を呼ぶため負の stride で panic しうる）
- ハイビット深度プレーンの doc「リトルエンディアン」表記が、dav1d の一次資料（picture.h: ピクセルは LSB ビットに配置）と一致しない

## 設計方針

- README の API 表と `src/lib.rs` の doc コメントを、実装と dav1d 1.5.4 の dav1d.h / picture.h に合わせて修正する
- doc コメントの情報欠落は、dav1d.h の対応する定義から限定情報を転記する

## 完了条件

- README の全 API 表が実装（戻り値の型含む）と一致すること
- `src/lib.rs` の全公開 doc に、dav1d.h の限定情報（`n_threads` の 0、`strict_std_compliance` の対象範囲、`output_invisible_frames` の符号化順・二重出現、`apply_grain` の二重適用警告等）が反映されること

## 解決方法

- `README.md` の `DecodedFrame` メソッド表の戻り値型を `Option` 付きに修正する
- `src/lib.rs` の `Decoder::decode()` の doc に「EAGAIN 時は渡したデータが破棄されるため、`next_frame()` でフレームを取り出した後に同じデータを再度渡すこと」を追記する
- `DecoderConfig::frame_delay()` の doc を「戻り値は 1 以上。`max_frame_delay` が 0 のときは dav1d が自動決定した値（1 〜 8）」に修正する
- `DecoderConfig` の各フィールドと `apply_grain()` / `DecodeFrameType::Intra` / `parse_sequence_header()` / `EventFlags::NEW_SEQUENCE` の doc を dav1d.h の記述に合わせて修正する
- `DecodedFrame::v_stride()` に `# Panics` を追記し、ハイビット深度のエンディアン表記を一次資料に合わせて修正する
