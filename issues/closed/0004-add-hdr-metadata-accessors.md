Created: 2026-03-19
Completed: 2026-03-19

# DecodedFrame に HDR メタデータのアクセサを追加する

Model: Opus 4.6

## 概要

dav1d の `Dav1dPicture` には HDR 関連のメタデータとして
`content_light` (Content Light Level) と `mastering_display` (Mastering Display Color Volume) が
含まれている。HDR コンテンツの正しいトーンマッピングに必要な情報であり、
デコーダーラッパーとして公開すべき。

## 追加すべき構造体

### ContentLightLevel

```rust
pub struct ContentLightLevel {
    /// 最大コンテンツ輝度 (cd/m²)
    pub max_cll: u16,
    /// 最大フレーム平均輝度 (cd/m²)
    pub max_fall: u16,
}
```

### MasteringDisplay

```rust
pub struct MasteringDisplay {
    /// 色域の原色座標 (R, G, B)、各 (x, y) は 0.16 固定小数点
    pub primaries: [[u16; 2]; 3],
    /// ホワイトポイント座標 (x, y)、0.16 固定小数点
    pub white_point: [u16; 2],
    /// 最大輝度、24.8 固定小数点 (cd/m²)
    pub max_luminance: u32,
    /// 最小輝度、18.14 固定小数点 (cd/m²)
    pub min_luminance: u32,
}
```

## 追加すべきアクセサ

- `content_light_level() -> Option<ContentLightLevel>` — ポインタが null の場合は None
- `mastering_display() -> Option<MasteringDisplay>` — ポインタが null の場合は None

## 根拠

HDR10 コンテンツでは Content Light Level と Mastering Display Color Volume が
トーンマッピングの入力パラメータとして使われる。
dav1d はこれらをフレームごとに提供しており、
svt-av1-rs のエンコーダー側でも同等の構造体を定義している。
デコーダー側でもこれを取得できるようにすることで、
デコード → 処理 → エンコードのパイプラインが完結する。

## 解決方法

`ContentLightLevel` と `MasteringDisplay` 構造体を定義し、
`DecodedFrame` に `content_light_level()` と `mastering_display()` アクセサを追加した。
ポインタが null の場合は `None` を返す。
