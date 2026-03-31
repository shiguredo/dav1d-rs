Created: 2026-03-19
Completed: 2026-03-19

# dav1d_get_event_flags に対応するメソッドを追加する

Model: Opus 4.6

## 概要

dav1d C API には `dav1d_get_event_flags()` 関数があるが、
現在の Rust ラッパーにはこれに対応するメソッドがない。
この関数はデコード中に発生したイベント (新しいシーケンスヘッダーの検出など) を取得できる。

## 追加すべき型とメソッド

```rust
/// デコードイベントフラグ (ビットフラグ)
pub struct EventFlags(u32);

impl EventFlags {
    /// 新しいシーケンスヘッダーが検出された
    pub const NEW_SEQUENCE: Self = Self(1);
    /// 現在のシーケンスで新しいオペレーティングパラメータが検出された
    pub const NEW_OP_PARAMS_INFO: Self = Self(2);

    /// 指定したフラグが含まれているかを判定する
    pub fn contains(self, flag: Self) -> bool { ... }
}

impl Decoder {
    /// デコード中に発生したイベントフラグを取得する
    ///
    /// 呼び出すと内部のフラグはクリアされる
    pub fn get_event_flags(&mut self) -> Result<EventFlags, Error> { ... }
}
```

## 根拠

ストリーム中で解像度変更やパラメータ変更が発生した場合に検知する手段が必要である。
ライブストリーミングでは途中でエンコーダー設定が変わることがあり、
アプリケーション側でレンダラーやバッファの再構成を行う必要がある。

## 解決方法

`EventFlags` ビットフラグ型と `Decoder::get_event_flags()` メソッドを追加した。
