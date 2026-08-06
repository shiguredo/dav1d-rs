# canary.py を正式リリースフローに対応させ、誤リリースを防ぐ

- Created: 2026-08-02
- Completed: 2026-08-06
- Branch: feature/fix-canary-release-flow
- Polished: {YYYY-MM-DD}
- Reporter: @voluntas

## reopened にした理由

- 起票時に誤って `issues/pending/` 直下に作成されていたため、`issues/` 直下の open 状態に戻す
- 仕様的に対応が難しい、設計判断が必要などの pending にする理由がない
## 目的

canary.py で正式リリース（例: 2026.2.0-canary.2 → 2026.2.0）を作成できるようにし、タグ push 前のブランチ・作業ツリー検証で誤リリースを防ぐ。

## 現状

- `canary.py` の `update_version` は `-canary.X` のインクリメント（`2026.2.0-canary.2` → `2026.2.0-canary.3`）と、非 canary からの次マイナー化（`2026.1.0` → `2026.2.0-canary.0`）のみ対応で、**canary から正式リリース版への変換機能がない**
- 2026.1.0 のリリースは手動で release ブランチ作成 → バージョン更新 → CHANGES.md のリリース日追記 → タグ push の流れで実施されており、手順が属人化している
- `git_operations_after_build` はブランチ名と作業ツリーの状態を検証せずに `git tag` → `git push` → `git push origin <tag>` を実行する
- `.github/workflows/release.yml` は全タグで発火し、GitHub Release 作成 → 8 資産の prebuilt ビルド → crates.io への `cargo publish` まで自動実行するため、feature ブランチ等での誤実行は未検証コミットの公開につながる

## 設計方針

- 正式リリースへの変換（`-canary.X` の除去）を `update_version` に追加する
- タグ push 前に、カレントブランチがリリース対象（develop または release/）であることと、作業ツリーがクリーンであることを検証する
- ブランチ検証でリリースを実行できない場合（canary でない version や不正なブランチ）はエラーで停止する

## 完了条件

- canary.py で canary → 正式リリース版への変換ができること
- リリース対象外のブランチや dirty な作業ツリーでタグ push が実行されないこと

## 解決方法

- `canary.py` の `update_version` に canary → 正式版（`-canary.X` の除去）の変換を追加する
- `canary.py` の `git_operations_after_build` に、カレントブランチの確認と `git status --porcelain` による作業ツリーのクリーン確認を追加する
- 正式リリース時の CHANGES.md 更新（develop セクションをバージョン + リリース日に変更）の支援を追加する
