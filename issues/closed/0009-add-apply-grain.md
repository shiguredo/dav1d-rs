Created: 2026-03-19
Completed: 2026-03-19

# dav1d_apply_grain に対応するメソッドを追加する

Model: Opus 4.6

## 概要

dav1d C API には `dav1d_apply_grain()` 関数があるが、
現在の Rust ラッパーにはこれに対応するメソッドがない。
この関数は `apply_grain=false` でデコードしたフレームに後からフィルムグレインを適用できる。

## 追加すべきメソッド

```rust
impl Decoder {
    /// デコード済みフレームにフィルムグレインを適用する
    ///
    /// `DecoderConfig::apply_grain` が `false` の場合に、
    /// 選択したフレームだけに後からグレインを適用できる。
    /// フレームにグレインメタデータがない場合は新しい参照を返す
    pub fn apply_grain(&mut self, frame: &DecodedFrame) -> Result<DecodedFrame, Error> { ... }
}
```

## 根拠

`apply_grain=false` で高速にデコードし、表示するフレームだけに後からグレインを適用するという
パフォーマンス最適化パターンが可能になる。
dav1d のドキュメントでもこのユースケースが想定されている。

## 解決方法

`Decoder::apply_grain()` メソッドを追加し、内部で `dav1d_apply_grain()` を呼び出すようにした。
