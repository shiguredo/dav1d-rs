# codec_info モジュールの推測的抽象化を削除する

- Created: 2026-08-02
- Completed: {YYYY-MM-DD}
- Branch: feature/refactor-remove-codec-info
- Polished: {YYYY-MM-DD}
- Reporter: @voluntas

## reopened にした理由

- 起票時に誤って `issues/pending/` 直下に作成されていたため、`issues/` 直下の open 状態に戻す
- 仕様的に対応が難しい、設計判断が必要などの pending にする理由がない
## 目的

単一コーデック（AV1）・デコード専用という確定事実を過剰な型構造で表現している `codec_info` モジュールを削除し、公開 API サーフェスを実態に合わせる。

## 現状

- `src/codec_info.rs` は 7 型（`VideoCodecType` / `CodecInfo` / `DecodingInfo` / `EncodingInfo` / `DecodingProfiles` / `EncodingProfiles` / `Av1DecodingProfile`）でコーデック情報を表現するが、`VideoCodecType` はバリアント 1 つの enum、`DecodingProfiles::Av1(Vec<...>)` と `EncodingProfiles::None` もバリアント 1 つである
- `supported_codecs()` が返す値は全て固定値（`decoding_info()` は `supported: true` / `hardware_accelerated: false` 固定、`encoding_info()` は `supported: false` 固定）で、動的情報を一切持たない
- リポジトリ内の利用は `codec_info.rs` 自身のテストのみで、`src/lib.rs` の `pub use codec_info::*;` により公開 API になっているが、README と CHANGES.md に一切記載がない
- タグ 2026.1.0 の公開クレートに含まれているが、CHANGES.md の 2026.1.0 セクションにも未記載である

## 設計方針

- 推測で追加された抽象化（YAGNI 違反）としてモジュールごと削除する
- 公開 API の破壊的変更を伴うため、2026.2.0 では実施せず次期リリースで対応する
- 削除しない選択肢（README / CHANGES.md への記載と動的情報の追加）もあるが、dav1d は AV1 デコード専用であり固定値 API に情報量がないため削除が適切

## 完了条件

- `src/codec_info.rs` と `src/lib.rs` の `mod codec_info;` / `pub use codec_info::*;` が削除され、ビルドとテストが通ること
- 削除に伴う破壊的変更が CHANGES.md に [CHANGE] として記載されること

## 解決方法

- `src/codec_info.rs` を削除する
- `src/lib.rs` から `mod codec_info;` と `pub use codec_info::*;` を削除する
- CHANGES.md の該当リリースセクションに [CHANGE] を追記する
