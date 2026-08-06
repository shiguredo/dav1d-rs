# AGENTS.md / 時雨堂 Rust 規約違反を修正する

- Created: 2026-08-02
- Completed: {YYYY-MM-DD}
- Branch: feature/fix-convention-violations
- Polished: {YYYY-MM-DD}
- Reporter: @voluntas

## reopened にした理由

- 起票時に誤って `issues/pending/` 直下に作成されていたため、`issues/` 直下の open 状態に戻す
- 仕様的に対応が難しい、設計判断が必要などの pending にする理由がない
## 目的

プロジェクト規約（AGENTS.md）と時雨堂 Rust 規約（shiguredo-rust スキル）への違反を解消し、規約の適用を一貫させる。

## 現状

- `tests/test_psnr.rs` の assert / expect メッセージの多数が英語（例: `"decoded {} frames, expected {num_frames}"` / `"frame {i}: width mismatch"` / `"frame {i}: PSNR {psnr:.1} dB < {min_psnr_db} dB"` / `"frame {i}: empty Y plane"`）。AGENTS.md の「テストのログメッセージは全て日本語にすること」に反する。`src/lib.rs` 内のテストの expect メッセージ（`"parse error"` 等）も同様
- `build.rs` の `eprintln!("prebuilt ライブラリをダウンロード中: ...")` が日本語。AGENTS.md の「ログメッセージは全て英語にすること」に反する（同じ関数内の SHA256 検証メッセージは英語で不統一）
- `Cargo.toml` の `rust-version = "1.88"` が、時雨堂 Rust 規約の MSRV 1.93 と不一致（意図的な緩和なら理由の明記が必要）
- `CHANGES.md` の種別順が shiguredo-changelog 規約の「CHANGE → ADD → UPDATE → FIX」に反する（`## develop` は UPDATE → ADD、2026.1.0 セクションは UPDATE → ADD → CHANGE → FIX）
- ~~`src/sys.rs` の `#![allow(...)]` が、時雨堂 Rust 規約の「lint 警告の抑制は `#[allow]` ではなく `#[expect]` を使うこと」に反する~~ → **撤回（2026-08-02）**
  - `sys.rs` は bindgen が生成する動的コードを include するためのモジュールであり、生成コードに lint を適用しない（aom-rs / svt-av1-rs と同じ構成）
  - expect は「必ず違反が発生する」手書きコード向けの機能であり、生成コードはプラットフォームによって違反する lint が異なる（dav1d.h が errno.h を include するため、macOS と Linux で生成物が変わる）。実際に CI (ubuntu-24.04) で `expect(non_camel_case_types)` が unfulfilled になり失敗することを確認
  - `#![allow(...)]` は `sys.rs` モジュール内に閉じており、手書きコード（`lib.rs` 等）には影響しない

## 設計方針

- 各違反箇所を規約に合わせて修正する
- `rust-version` は規約（1.93）に合わせるか、意図的に緩和するならその理由をコメントで明記する

## 完了条件

- 上記の規約違反が全て解消されること
- `cargo fmt` / `cargo clippy` / `cargo test` が通ること

## 解決方法

- `tests/test_psnr.rs` と `src/lib.rs` のテストメッセージを日本語に変更する（対応済み）
- `build.rs` の日本語 eprintln を英語に変更する（対応済み）
- `Cargo.toml` の `rust-version` を 1.93 に変更する（対応済み）
- `CHANGES.md` の種別順を CHANGE → ADD → UPDATE → FIX に並べ替える（対応済み）
- `src/sys.rs` の `#![allow]` は生成コード用の lint 抑止であり、expect 化しない（撤回。aom-rs / svt-av1-rs と同じ構成を維持する）
