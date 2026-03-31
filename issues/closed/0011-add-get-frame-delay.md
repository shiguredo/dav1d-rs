Created: 2026-03-19
Completed: 2026-03-19

# dav1d_get_frame_delay に対応する関数を追加する

Model: Opus 4.6

## 概要

dav1d C API には `dav1d_get_frame_delay()` 関数があるが、
現在の Rust ラッパーにはこれに対応する関数がない。
この関数は設定に対するデコーダーの実際のフレーム遅延を返す。

## 追加すべきメソッド

```rust
impl DecoderConfig {
    /// この設定でのデコーダーのフレーム遅延を取得する
    ///
    /// 戻り値は 1 以上 max_frame_delay 以下であることが保証される
    pub fn frame_delay(&self) -> Result<usize, Error> { ... }
}
```

## 根拠

低遅延デコードにおいて、設定した `max_frame_delay` に対して
実際にどれだけのフレーム遅延が発生するかを事前に確認する手段が必要である。
デコーダーを開く前にバッファサイズの計画を立てられる。

## 解決方法

`DecoderConfig::frame_delay()` メソッドを追加し、内部で `dav1d_get_frame_delay()` を呼び出すようにした。
