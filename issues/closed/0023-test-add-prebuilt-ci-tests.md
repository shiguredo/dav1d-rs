# CI で prebuilt（デフォルト）ビルド経路を検証する

- Created: 2026-08-02
- Completed: 2026-08-06
- Branch: feature/add-prebuilt-ci-tests
- Polished: {YYYY-MM-DD}
- Reporter: @voluntas

## reopened にした理由

- 起票時に誤って `issues/pending/` 直下に作成されていたため、`issues/` 直下の open 状態に戻す
- 仕様的に対応が難しい、設計判断が必要などの pending にする理由がない
## 目的

デフォルトのビルド経路（GitHub Releases からの prebuilt ダウンロード）を CI で検証し、配布資産の破損や不足をリリース前に検出できるようにする。

## 現状

- `.github/workflows/ci.yml` の test ジョブは常に `--features source-build` で実行され、prebuilt 経路（`build.rs` の `download_prebuilt` による curl → SHA256 検証 → tar 展開 → コピー → リンク）は CI のどのジョブでも実行されていない
- 特に Windows の prebuilt 経路（curl.exe / certutil による SHA256 解析 / COFF アーカイブのリンク）は一度も検証されない
- prebuilt 資産が「実際にビルドできるか」は、リリース後に初めてユーザーがデフォルトビルドを実行した時点で判明する

## 設計方針

- CI に prebuilt 経路のジョブ（またはステップ）を追加し、canary リリース時点の資産を含めて検証する
- source-build ジョブとの排他ではなく、両方の経路を CI で常に検証する

## 完了条件

- CI で prebuilt 経路の `cargo check` / `cargo test` が実行され、通ること
- Windows の prebuilt 経路（可能であれば）が検証されること

## 解決方法

- `.github/workflows/ci.yml` に prebuilt 経路（`--features` なし）のジョブを追加する
- リリース検証（issue 0015）と合わせて、全 8 資産のダウンロード + SHA256 照合をリリースフローに組み込む
