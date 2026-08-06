# canary.py のコミットメッセージを git 規約に適合させる

- Created: 2026-08-02
- Completed: 2026-08-06
- Branch: feature/fix-canary-commit-message
- Polished: {YYYY-MM-DD}
- Reporter: @voluntas

## reopened にした理由

- 起票時に誤って `issues/pending/` 直下に作成されていたため、`issues/` 直下の open 状態に戻す
- 仕様的に対応が難しい、設計判断が必要などの pending にする理由がない
## 目的

canary.py が生成するコミットメッセージを shiguredo-git 規約に適合させ、規約違反のコミットが生成され続けることを防ぐ。

## 現状

- `canary.py` の `git_commit_version` は `[canary] Bump version to <version>` というコミットメッセージを生成する
- このメッセージは shiguredo-git 規約に以下の 3 点で違反する:
  - `[canary]` という prefix（規約の形式に含まれない）
  - 英語（規約は日本語）
  - 命令形でない（規約は「〜する」の命令形）
- 既存のコミット履歴（`[canary] Bump version to 2026.2.0-canary.2` 等）はこのスクリプトで生成されたものであり、スクリプトを修正しない限り規約違反のコミットが生成され続ける

## 設計方針

- shiguredo-git 規約（日本語・命令形）に適合したメッセージを生成する
- 例: `canary バージョンを 2026.2.0-canary.3 に更新する`

## 完了条件

- canary.py が生成するコミットメッセージが shiguredo-git 規約に適合すること

## 解決方法

- `canary.py` の `git_commit_version` のコミットメッセージを規約適合形式に変更する
