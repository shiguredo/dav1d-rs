# リリース時の prebuilt 資産検証とタグ整合検証を強化する

- Created: 2026-08-02
- Completed: {YYYY-MM-DD}
- Branch: feature/fix-verify-release-prebuilt-assets
- Polished: {YYYY-MM-DD}
- Reporter: @voluntas

## 目的

正式リリースで配布する prebuilt 資産の破損・欠落・バージョン不一致を、リリースプロセス内で自動検出できるようにする。

## 現状

- `.github/workflows/release.yml` の `cargo publish` による検証は、publish ジョブの実行環境（ubuntu-24.04）の 1 資産のみをダウンロードして検証する。残り 7 資産（ubuntu-26.04 x86_64/arm64、ubuntu-24.04 arm64、ubuntu-22.04 x86_64/arm64、macos_arm64、windows_x86_64）は `gh release upload` の成功しか保証されず、アップロード後の破損や SHA256 不一致は検知されない
- `.github/workflows/release.yml` の github-release ジョブに、タグ名と `Cargo.toml` の `package.version` の一致検証がない。誤ったタグ（`v` プレフィックス付き等）を push すると正式リリースが作成され、8 資産がアップロードされた後に publish が「prebuilt 404」で失敗する。crates.io への不正公開は防がれるが、ゴミ Release と紛らわしいエラーが残る
- crates.io に公開済みの 2025.1.0 / 2025.1.0-canary.0/1/2 は GitHub Release が存在せず、prebuilt パス（デフォルトビルド）で利用すると `build.rs` のダウンロードが 404 で panic する（実測）。リリース資産の恒久保存方針が未決定

## 設計方針

- リリースプロセス内に「全資産の完全性検証」と「タグとバージョンの一致検証」を組み込む
- 過去バージョンの資産欠落については運用方針として対応を決める

## 完了条件

- リリース時に全 8 資産のダウンロードと SHA256 照合が自動実行されること
- タグ名と `Cargo.toml` の `package.version` の不一致が、Release 作成前に検出されること
- 過去バージョンの資産欠落への対応方針（補完または運用ルール）が決まること

## 解決方法

- `.github/workflows/release.yml` の github-release ジョブに、タグ名と `Cargo.toml` の `package.version` の一致を検証するステップを追加する
- publish ジョブ（またはその前段）に、全 8 資産の `curl -fsSL` ダウンロードと SHA256 再計算の検証ステップを追加する
- 2025.1.0 系の資産欠落については、欠落しているバージョンの資産を作り直すか、該当バージョンの yank など方針を決めて対応する
