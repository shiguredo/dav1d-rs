# iOS / Android 向けの prebuilt を追加する

- Created: 2026-10-03
- Completed: {YYYY-MM-DD}
- Branch: feature/add-mobile-prebuilt
- Polished: {YYYY-MM-DD}

## 目的

モバイル向けの Rust アプリケーションでも、dav1d をソースからビルドせずに prebuilt で利用できるようにする。
ユーザーからの「opus-rs や aom-rs と同様に iOS / Android 向けの prebuilt を用意したい」という要望に対応する。

## 現状

- `build.rs` の `get_target_platform` は Linux、macOS、Windows 向けのみを扱い、iOS / Android のターゲットを指定すると panic する
- `build.rs` の `rewrite_symbols` は Mach-O のシンボル先頭の `_` を `CARGO_CFG_TARGET_OS == "macos"` の場合だけ処理しており、iOS を追加するとシンボル書き換えが壊れる
- `build.rs` の `build_from_source` に iOS の Xcode SDK と Android NDK を使う設定がなく、モバイル向けのソースビルドができない
- `.github/workflows/ci.yml` と `.github/workflows/release.yml` にモバイル向けのビルドジョブがない
- `README.md` に iOS / Android 向けのビルド手順と prebuilt の記述がない
- `Cargo.toml` の dev-dependencies (`shiguredo_aom` / `shiguredo_svt_av1`) はモバイル prebuilt 未対応であり、モバイルで `cargo test --lib --no-run` によるリンク検証を行うと dev-dependencies のビルドで失敗する

## 設計方針

### 対象ターゲット

| 対象 | Rust ターゲット | prebuilt のプラットフォーム名 |
| --- | --- | --- |
| iOS 実機 arm64 | `aarch64-apple-ios` | `ios_arm64` |
| iOS シミュレーター arm64 | `aarch64-apple-ios-sim` | `ios-sim_arm64` |
| iOS シミュレーター x86_64 | `x86_64-apple-ios` | `ios-sim_x86_64` |
| Android arm64-v8a | `aarch64-linux-android` | `android_arm64` |
| Android x86_64 | `x86_64-linux-android` | `android_x86_64` |

### ビルド

- iOS の prebuilt は実機と x86_64 シミュレーターが iOS 13.0 以降、arm64 シミュレーターが iOS 14.0 以降を対象とする
- Android の prebuilt は arm64-v8a と x86_64 の 2 ABI、API level 21 以降を対象とし、NDK `28.2.13676358` でビルドする
- iOS では `xcrun` で Xcode SDK のパスを解決し、meson の cross file と bindgen に同じ SDK、アーキテクチャ、最小 OS バージョンを指定する
- Android では `ANDROID_NDK_HOME` の NDK ツールチェーンを使い、`ANDROID_PLATFORM` で最小 API level を指定する (21 未満は拒否する)
- モバイルでは meson に `-Denable_tools=false -Denable_tests=false` を渡し、CLI ツールとテストをビルドしない
- Mach-O のシンボル書き換えは Apple プラットフォーム判定 (`CARGO_CFG_TARGET_VENDOR == "apple"`) に変更して iOS でも動くようにする
- モバイルのリンク検証は dev-dependencies のビルドを回避するため、dev-dependencies をモバイル以外のターゲットに限定する

### 配布と検証

- `.github/workflows/mobile.yml` を追加し、CI とリリースから共用する
- ワークフローではソースビルド、Rust のリンク、シンボルのプレフィックス検証、アーカイブ生成、SHA256 検証、展開物の一致確認を行う
- アーカイブにはシンボル書き換え済みの `lib/libdav1d.a`、`bindings.rs`、dav1d の `COPYING` を収録する
- リリースではアップロード後に prebuilt の自動選択とリンクを検証し、成功を `publish` の前提にする

### 事前検証

dav1d 1.5.4 をローカルの meson 1.11 + Xcode 26.5 / NDK r29 でクロスビルドし、全 5 ターゲットで静的ライブラリの生成に成功している。

| ターゲット | 結果 |
| --- | --- |
| `aarch64-apple-ios` | 成功 (Mach-O arm64、NEON シンボルあり) |
| `aarch64-apple-ios-sim` | 成功 |
| `x86_64-apple-ios` | 成功 (NASM macho64、AVX2 シンボルあり) |
| `aarch64-linux-android` | 成功 (ELF、NEON シンボルあり) |
| `x86_64-linux-android` | 成功 (NASM elf64、AVX2 シンボルあり) |

また、`dav1d.h` が bindgen に渡す clang 引数 (`--target` + `-isysroot` / `--sysroot`) で全ターゲットのコンパイルを通ることを確認している。

## 完了条件

- 全 5 ターゲットで静的ライブラリとバインディングを生成できる
- 全定義済み外部シンボルが、Mach-O 固有の先頭 `_` を除いて `shiguredo_dav1d_` プレフィックスを持つ
- 各アーカイブの SHA256 が一致し、収録したライブラリとバインディングを用いて Rust のリンクが成功する
- GitHub Actions の CI でモバイル向けビルドとリンクの検証が通る
- 次回リリースでアーカイブとチェックサムのアップロード、公開された prebuilt の自動選択とリンクの検証、crates.io への公開の順に進むワークフローを構成し、検証に失敗した場合は `publish` を開始しない
- 既存の単体テスト、PBT、フォーマット、Clippy が通る
