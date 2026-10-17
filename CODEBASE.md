# CODEBASE

このリポジトリ固有の規約と設定をまとめる。共通規約は `AGENTS.md` と各スキルを参照すること。

## 概要

- `shiguredo_dav1d` は dav1d を静的リンクする AV1 デコーダーの Rust バインディング
- dav1d のバージョンは `Cargo.toml` の `[package.metadata.external-dependencies.dav1d]` が正本
- 実行時依存クレートは持たない (dav1d は C の静的ライブラリとして同梱する)
- カレンダーバージョニング (`YYYY.M.P`) を採用する
- ライセンスは Apache-2.0、同梱する dav1d は BSD-2-Clause

## ディレクトリ構成

| パス | 役割 |
|---|---|
| `src/lib.rs` | 公開 API のほぼ全て (Decoder / DecodedFrame / DecoderConfig / 列挙型 / エラー) |
| `src/codec_info.rs` | `supported_codecs()` とコーデック情報の型 |
| `src/sys.rs` | `build.rs` が生成するバインディングを取り込むだけのモジュール |
| `build.rs` | prebuilt 取得 / ソースビルド / モバイル向け cross file / シンボル書き換え / docs.rs 向けダミー |
| `tests/test_psnr.rs` | AOM / SVT-AV1 で実際にエンコードし、デコード結果の PSNR を検証する |
| `pbt/tests/prop_lib.rs` | noprop による公開 API のプロパティテスト |
| `fuzz/` | cargo-fuzz のターゲット。ワークスペースからは除外している |
| `skills/shiguredo-dav1d/SKILL.md` | 利用者向けの API リファレンス |
| `canary.py` | バージョン更新・CHANGES 更新・タグ付け・プッシュを行うリリーススクリプト |
| `.github/workflows/` | CI / モバイル prebuilt / リリース |

## ビルド経路

`build.rs` は 3 つの経路を持つ。経路によって必要な外部コマンドと生成物が変わる。

### prebuilt (既定)

`source-build` feature が無効な場合の経路。

- `https://github.com/shiguredo/dav1d-rs/releases/download/{Cargo.toml の version}/dav1d-{target}.tar.gz` を取得する
- アーカイブは `lib/libdav1d.a`、`bindings.rs`、`COPYING` で構成し、別途 `.sha256` を配布する
- 取得したアーカイブは SHA-256 で検証してから展開する
- 必須コマンド: `curl`、`tar`、`sha256sum` (macOS は `shasum`、Windows は `certutil`)
- ターゲットは OS とアーキテクチャから自動判定する。Linux は `/etc/os-release` の `VERSION_ID` も見る
- `DAV1D_TARGET` を設定すると自動判定を上書きできる。対応外の環境ではこの環境変数が必要になる
- リリース資産が存在しないバージョンでは 404 で失敗する。canary であっても prebuilt を必ずアップロードする
- prebuilt の `bindings.rs` は dav1d の全公開 API を含むため、既存 API を使う限り prebuilt のままで新しい機能を追加できる
- dav1d のバージョンを更新した場合は、新しいバージョンに対応する prebuilt を作り直す必要がある

### source-build

`--features source-build` を指定した場合の経路。

- `Cargo.toml` の URL とバージョンから dav1d を shallow clone し、タグをチェックアウトする
- Meson / Ninja でビルドし、NASM でアセンブリ最適化を有効にする
- `bindgen` で `include/dav1d/dav1d.h` からバインディングを生成する
- 生成したバインディングに `#[link_name]` を付けてシンボル書き換えを透過的に行う
- 必須コマンド: `git`、C コンパイラ、`meson`、`ninja`、`nasm`、rustup の `llvm-tools` コンポーネント
- iOS は `IPHONEOS_DEPLOYMENT_TARGET`、Android は `ANDROID_NDK_HOME` と `ANDROID_PLATFORM` から cross file を生成する
- iOS と Android の Rust ターゲットは `build.rs` が明示的に列挙している。対応外のターゲットでは panic する

### docs.rs

`DOCS_RS=1` の場合の経路。

- git clone もビルドも行わず、`build.rs` 内のダミーバインディングを `OUT_DIR/bindings.rs` に書き出す
- `src/lib.rs` が参照する `sys` の関数・構造体・定数を追加したら、ダミーにも必ず追記する
- CI の `docs-rs` ジョブが `cargo doc --no-deps` で検証する

## シンボル書き換え

別の dav1d 利用ライブラリと同一プロセスで共存できるようにするための仕組み。

- 静的ライブラリ内の全外部シンボルに `shiguredo_dav1d_` プレフィックスを付ける
- C の公開 API は `dav1d_` を `shiguredo_dav1d_` に置換し、内部シンボルは単純にプレフィックスを付与する
- macOS / iOS の Mach-O は先頭の `_` を含めて扱う
- CI と `release.yml` が `llvm-nm` で全外部シンボルを検証する
- prebuilt は書き換え済みの状態で配布する
- 新しい C ライブラリを追加する場合も同じ方針を守る

## モバイル

- iOS 実機 / iOS シミュレーター / Android arm64-v8a / Android x86_64 の 4 ターゲットを prebuilt で配布する
- モバイル向け prebuilt は `release.yml` から `mobile.yml` を呼び出して生成する
- `mobile.yml` は実機実行ができないため、テスト実行ファイルの生成 (リンク) までを検証する
- `shiguredo_aom` と `shiguredo_svt_av1` はモバイル向け prebuilt を持たないため、dev-dependencies をモバイル以外に限定している
- `tests/test_psnr.rs` もモバイルではテスト全体を無効化している
- モバイル向け prebuilt のアーカイブにも dav1d の `COPYING` を同梱する

## バージョンとリリース

- `Cargo.toml` の `version` と `CHANGES.md` は `canary.py` 経由で更新する。手作業で書き換えない
- `canary.py` は `develop` または `release/` ブランチでのみ実行でき、作業ツリーがクリーンであることを要求する
- canary リリース: `python3 canary.py` で canary 番号をインクリメントし、コミット・タグ・プッシュまで行う
- 正式リリース: `release/YYYY.M.P` ブランチで `python3 canary.py --release` を実行する
  - `CHANGES.md` の `## develop` が `## YYYY.M.P` とリリース日に置き換わる
- タグは `Cargo.toml` の `version` と完全に一致させる。`v` プレフィックスは付けない
- タグをプッシュすると `release.yml` が次を実行する
  - GitHub Release の作成
  - prebuilt 12 個 (デスクトップ 8 + モバイル 4) のビルドとアップロード
  - crates.io への公開
- リリース後は `release/` ブランチを `develop` にマージする
- `CHANGES.md` の書式は `shiguredo-changelog` スキルに従う

## テスト

- `Makefile` の `make test` / `make cover` / `make clippy` / `make fmt` を利用する
- `make test` は `cargo test --workspace --features source-build` を実行する
- CI では `shiguredo_aom/source-build` と `shiguredo_svt_av1/source-build` も有効にする
- `tests/test_psnr.rs` は AOM と SVT-AV1 で実際に AV1 を生成し、dav1d のデコード結果を PSNR で検証する
  - モックやスタブは使わず、実際のエンコーダーとデコーダーを使う
  - PSNR の閾値はプラットフォーム間の SIMD 差を考慮した根拠コメント付きで管理する。閾値を変えるときは根拠も更新する
- `pbt/` は noprop を使う。再現用のシードは `SHIGUREDO_DAV1D_PBT_SEED` で指定できる
- `fuzz/` は cargo-fuzz が独立したワークスペースとして管理する。ターゲットを追加したら `fuzz/Cargo.toml` に `[[bin]]` を追加する
- 単体テストは公開 API に対して `tests/test_<module>.rs` に、private な実装は `src/<module>.rs` の `#[cfg(test)]` モジュールに書く
- PBT は `pbt/tests/prop_<module>.rs` に書く
- テストのログメッセージと `#[cfg(test)]` 内の `expect` メッセージは日本語にする

## CI

- `ci.yml` は `fmt-clippy`、`test` (9 環境)、`prebuilt`、`mobile`、`docs-rs`、`slack_notify` で構成する
- `paths-ignore` に `**.md` を指定しているため、Markdown だけの変更では CI が動かない
- GitHub Actions はコミットハッシュ固定で参照し、`# vX.Y.Z` のコメントを付ける
- action の更新は `update-actions` スキルに従う
- ツールチェーンは `rust-toolchain.toml` の `channel` で固定する。CI の `rust-cache` に渡す `toolchain` も同じ値にする
- `prebuilt` ジョブはリリース資産のアップロード完了を最大 15 分待ってから `cargo check` する
- `mobile.yml` は `workflow_call` 専用で、iOS / Android の prebuilt をビルド・検証・アップロードする
- `release.yml` はタグ起点で、タグと `Cargo.toml` の一致検証、シンボル検証、アーカイブ検証を行ってから公開する

### MSRV を更新するときの同時更新箇所

- `Cargo.toml` の `rust-version`
- `rust-toolchain.toml` の `channel`
- `pbt/Cargo.toml` と `fuzz/Cargo.toml` の `rust-version`
- `.github/workflows/ci.yml` の `rust-cache` に渡す `toolchain`
- `README.md` と `skills/shiguredo-dav1d/SKILL.md` の最小 Rust バージョン
- `CHANGES.md` の misc

## ドキュメント

- `README.md` が利用者向けの一次ドキュメントである。公開 API・設定・対応環境を変更したら README の表も更新する
- `skills/shiguredo-dav1d/SKILL.md` は利用者向けの API リファレンスである
  - crate バージョン、dav1d バージョン、対応機能、API、利用時の制約を README と実装に同期させる
- `CHANGES.md` は変更内容そのものを書く。issue 番号や issue への言及は書かない
- 公開 API には `#![warn(missing_docs)]` により doc コメントが必須である
- 仕様由来の処理 (H.273 の列挙値、dav1d の API 契約など) は根拠資料名・節番号と、将来変更されうることをコメントに書く

## 実装上の注意

- `src/sys.rs` は bindgen 生成コードを取り込むだけのモジュールである
  - 生成コード由来の lint を抑制するために `#![allow(...)]` を使う唯一の例外である
  - ここ以外で `#[allow(...)]` を追加しない
- FFI 呼び出しの戻り値は `Error::check()` で検査し、失敗した dav1d 関数名を保持する。EAGAIN は `Error::is_eagain()` で判定する
- リソースは RAII で管理する。`Decoder` は `Drop` で `dav1d_close`、`DecodedFrame` は `Drop` で参照を解放する
- `unsafe impl Send` / `unsafe impl Sync` には `SAFETY` コメントで根拠を書く。現状は `Decoder` が `Send` のみ、`DecodedFrame` が `Send + Sync` である
- フレームのプレーンには行末パディングが含まれる。有効画素を読むときはストライドを考慮する
- 未対応機能はカスタムピクセルアロケーターとゼロコピー入力である
  - ログ出力は `Logger` で対応済みであり、README やスキルの「未対応」に書かない
- `canary.py` は Python で書かれている。変更するときは `shiguredo-python` スキルに従う

## 開発環境

- Git フックは `prek install --prepare-hooks` で導入する
- pre-commit では `cargo fmt --check`、`cargo clippy` (source-build)、tombi の lint / format、組み込みフックを実行する
- pre-push では `cargo test` (source-build) を実行する
- ソースビルドには Meson / Ninja / NASM と `llvm-tools` が必要である
- prebuilt 経路をローカルで確認するときは、対象バージョンの GitHub Release に prebuilt が存在している必要がある
- `make check` で prebuilt 経路の `cargo check` を実行できる
