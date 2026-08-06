# README にリリース手順を記載する

- Created: 2026-08-02
- Completed: {YYYY-MM-DD}
- Branch: feature/add-release-procedure
- Polished: {YYYY-MM-DD}
- Reporter: @voluntas

## reopened にした理由

- 起票時に誤って `issues/pending/` 直下に作成されていたため、`issues/` 直下の open 状態に戻す
- 仕様的に対応が難しい、設計判断が必要などの pending にする理由がない
## 目的

リリース手順（canary リリースと正式リリース）を README に文書化し、属人化を解消する。

## 現状

- README にリリース手順のセクションがない
- `canary.py` の使い方（対話式バージョン更新 → コミット → タグ → push）が README に説明されていない
- 2026.1.0 の正式リリースは「release ブランチ作成 → Cargo.toml のバージョン更新 → CHANGES.md の develop セクションをバージョン + リリース日に更新 → タグ push → release.yml が prebuilt ビルドと cargo publish を実行」の手順で実施されたが、この手順がどこにも文書化されていない
- リリース作業は担当者の暗黙知に依存しており、手順の確認漏れでタグと Cargo.toml の version 不一致（prebuilt 資産の 404）等が発生しうる

## 設計方針

- README に「リリース手順」セクションを追加し、canary リリースと正式リリースの両方を記載する
- canary.py の使い方（実行タイミング・引数・前提条件）を含める

## 完了条件

- README の手順に従って canary リリースと正式リリースを実施できること

## 解決方法

- README.md に以下の内容を記載する:
  - canary リリース: `canary.py` の実行手順（バージョン更新 → コミット → タグ → push）と、release.yml が GitHub Release 作成 → prebuilt ビルド → cargo publish まで自動実行すること
  - 正式リリース: release ブランチ作成 → バージョン更新（canary の除去）→ CHANGES.md の develop セクションをリリース版 + リリース日付に更新 → タグ push
