# DOCS_RS 向けダミーバインディングを修正し、DOCS_RS 時のビルドを復旧させる

- Created: 2026-08-02
- Completed: {YYYY-MM-DD}
- Branch: feature/fix-docs-rs-dummy-bindings
- Polished: {YYYY-MM-DD}
- Reporter: @voluntas

## reopened にした理由

- 起票時に誤って `issues/pending/` 直下に作成されていたため、`issues/` 直下の open 状態に戻す
- 仕様的に対応が難しい、設計判断が必要などの pending にする理由がない
## 目的

`DOCS_RS=1` を付けた環境で `cargo check` / `cargo build` が失敗する問題と、DOCS_RS ビルド後に通常ビルドまで壊れ続ける問題を修正する。

## 現状

- `build.rs` の DOCS_RS 分岐が書き出すダミー定義は、型と定数のみで関数宣言と構造体フィールドが一切ない。`src/lib.rs` が参照する `dav1d_version` 等の関数群と、`Dav1dPicture` のフィールド（`p` / `data` / `stride` / `frame_hdr` / `seq_hdr` / `content_light` / `mastering_display`）、`Dav1dSettings` のフィールドが未定義のため、`DOCS_RS=1 cargo check` は 82 エラーで失敗する（実測）
- `build.rs` の `cargo::rerun-if-env-changed` に `DOCS_RS` が含まれない。そのため `DOCS_RS=1 cargo doc` を一度実行すると build.rs が再実行されず、以後 DOCS_RS を外した通常ビルドでもダミー bindings.rs が使われ続けて 82 エラーで失敗する（クリーン環境で再現実測）
- 参考: docs.rs 本体は `cargo doc`（rustdoc は関数本体を型検査しない）でビルド成功しているため、docs.rs の公開自体は止まっていない。破綻するのはローカルの `cargo check` / `cargo build` と開発フロー

## 設計方針

- `cargo::rerun-if-env-changed=DOCS_RS` を追加して、DOCS_RS の有無で build.rs が必ず再実行されるようにする
- ダミー定義を `src/lib.rs` が参照する全シンボル（関数・構造体フィールド・定数）に対して網羅するか、DOCS_RS 時にも本物の bindgen 生成バインディングを使う方式に切り替える

## 完了条件

- `DOCS_RS=1 cargo check` と `DOCS_RS=1 cargo build` が通ること
- DOCS_RS 有無を切り替えても、どちらの状態でも `cargo check` / `cargo build` / `cargo test` が通ること

## 解決方法

- `build.rs` の rerun-if-env-changed に `DOCS_RS` を追加する
- DOCS_RS 分岐のダミー定義を `src/lib.rs` / `src/codec_info.rs` が参照する全シンボルに拡張する（`build.rs` の DOCS_RS 分岐）
- または、ダミー定義を廃止して DOCS_RS 時にも bindgen で本物のバインディングを生成する方式を検討する（ネットワーク不要のため、prebuilt ダウンロードをスキップするだけでよい）
