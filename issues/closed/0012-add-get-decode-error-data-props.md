Created: 2026-03-19
Completed: 2026-03-19

# dav1d_get_decode_error_data_props に対応するメソッドを追加する

Model: Opus 4.6

## 概要

dav1d C API には `dav1d_get_decode_error_data_props()` 関数があるが、
現在の Rust ラッパーにはこれに対応するメソッドがない。
この関数は最後のデコードエラーに関連する入力データのプロパティを取得できる。

## 追加すべき型とメソッド

```rust
/// デコードエラーに関連するデータプロパティ
pub struct DataProps {
    /// データのタイムスタンプ
    pub timestamp: i64,
    /// データの再生時間
    pub duration: i64,
    /// データのオフセット
    pub offset: i64,
    /// データのサイズ
    pub size: usize,
}

impl Decoder {
    /// 最後のデコードエラーに関連するデータプロパティを取得する
    pub fn get_decode_error_data_props(&mut self) -> Result<DataProps, Error> { ... }
}
```

## 根拠

デコードエラーが発生した際に、どの入力データがエラーを引き起こしたかを特定する手段が必要である。
タイムスタンプやオフセットの情報があれば、
ストリーム中の問題箇所を特定してデバッグやエラーレポートに活用できる。

## 解決方法

`DataProps` 構造体と `Decoder::get_decode_error_data_props()` メソッドを追加した。
