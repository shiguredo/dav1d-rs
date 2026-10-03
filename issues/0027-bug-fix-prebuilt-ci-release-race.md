# prebuilt CI がリリース資産のアップロードと競合するのを防ぐ

- Created: 2026-10-03
- Completed: {YYYY-MM-DD}
- Branch: feature/fix-prebuilt-ci-race
- Polished: {YYYY-MM-DD}
- Reporter: @voluntas

## 目的

リリース / canary のバージョン bump と同一の push で CI の prebuilt ジョブが起動しても、Release ワークフローの資産アップロードと競合して build.rs のダウンロードが 404 で失敗しないようにする。

## 現状

- `.github/workflows/ci.yml` の `prebuilt` ジョブは、`cargo check` で build.rs の `download_prebuilt`（デフォルトの prebuilt ダウンロード経路）を検証する
- build.rs は `CARGO_PKG_VERSION` のバージョンのリリース資産をダウンロードするため、バージョンを上げる push では Release ワークフローが作成する資産が必要になる
- 2026.3.0 のリリースでは、タグ `2026.3.0` の push と develop へのマージが同時（2026-10-03 04:53:36 UTC）に発生し、CI と Release が同時に起動した
- Release ワークフローは Release 作成後に資産を順次アップロードする。`dav1d-windows_x86_64.tar.gz` は 05:01:01 まで公開されず、04:55:52 にダウンロードを試みた `prebuilt-windows-2025` が 404 で失敗した（`dav1d-ubuntu-24.04_x86_64` は 04:55:17 に公開され、CI が間に合った）
- 2026.2.0 の CI でも `prebuilt-windows-2025` は 1 回目の実行で失敗した後に再実行されており（2 回目で成功）、同じ競合が再発している

## 設計方針

- `prebuilt` ジョブ内で、build.rs がダウンロードする資産が公開されるまで待機してから `cargo check` を実行する
- 待機対象は build.rs と同じ資産名にする。matrix にターゲット名を追加し `DAV1D_TARGET` で両者を一致させる
- `.tar.gz` と `.sha256` の両方が公開されたことをもって待機完了とする。`.sha256` は `.tar.gz` より後にアップロードされるため、両方の確認で不完全な状態でのダウンロードを防ぐ
- 待機には上限（15 分）を設け、リリース前のバージョンで資産が存在しない場合は明示的に失敗させる

## 完了条件

- リリース / canary の push で prebuilt ジョブが資産アップロードと競合しても失敗しないこと
- リリース済みバージョンに対する通常の push では待機せずに検証が実行されること

## 解決方法

- `.github/workflows/ci.yml` の `prebuilt` ジョブに matrix の `target` と `DAV1D_TARGET` を追加する
- 資産が公開されるまで最大 15 分ポーリングする `Wait for prebuilt assets` ステップを `cargo check` の前に追加する
- 待機分を見込み `timeout-minutes` を 15 から 30 に引き上げる
