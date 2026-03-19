Created: 2026-03-19
Completed: 2026-03-19

# dav1d_parse_sequence_header に対応する関数を追加する

Model: Opus 4.6

## 概要

dav1d C API には `dav1d_parse_sequence_header()` 関数があるが、
現在の Rust ラッパーにはこれに対応する関数がない。
この関数はデコーダーを開かずにビットストリームからシーケンスヘッダーを解析できる。

## 追加すべき関数

```rust
/// ビットストリームからシーケンスヘッダーを解析する
///
/// デコーダーを開かずにストリームのメタデータを事前に取得できる。
/// シーケンスヘッダー以外の OBU が含まれていても無視される
pub fn parse_sequence_header(data: &[u8]) -> Result<SequenceHeader, Error> { ... }
```

`SequenceHeader` 構造体にはシーケンスヘッダーから取得できる主要なフィールドを含める。

## 根拠

ストリームの解像度、プロファイル、色空間情報をデコーダーを開かずに事前に取得できると、
コーデック判定やストリーム情報の表示に有用である。
デコーダーのリソースを確保する前にストリームの特性を確認できる。

## 解決方法

`SequenceHeader` 構造体と `parse_sequence_header()` 関数を追加した。
`dav1d_parse_sequence_header()` を呼び出し、結果を Rust の型に変換して返す。
