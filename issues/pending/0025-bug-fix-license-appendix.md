# LICENSE ファイルを正規の Apache-2.0 全文に修正する

- Created: 2026-08-02
- Completed: {YYYY-MM-DD}
- Branch: feature/fix-license-appendix
- Polished: {YYYY-MM-DD}
- Reporter: @voluntas

## 目的

LICENSE ファイルと README のライセンス表記を正規の Apache-2.0 の形に整える。

## 現状

- LICENSE ファイルは「END OF TERMS AND CONDITIONS」で終わっており、正規の Apache License 2.0 全文に含まれる APPENDIX（How to apply the Apache License to your work.）セクションが欠落している
- セクション 1-9 の法的本文は完全だが、慣例上全文（APPENDIX 含む）を配布する
- README のライセンス表記（Copyright 2026-2026, Shiguredo Inc.）が単一年なのにレンジ形式になっている

## 設計方針

- Apache-2.0 の正規全文（APPENDIX 含む）に置き換える
- README の Copyright 表記を単一年に修正する

## 完了条件

- LICENSE が正規の Apache-2.0 全文（APPENDIX 含む）であること

## 解決方法

- LICENSE を Apache-2.0 の正規全文（APPENDIX 含む）に置き換える
- README.md の Copyright 表記を「Copyright 2026, Shiguredo Inc.」に修正する
