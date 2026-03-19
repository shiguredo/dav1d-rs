Created: 2026-03-19
Completed: 2026-03-19

# 色空間メタデータ用の enum を追加する

Model: Opus 4.6

## 概要

dav1d はデコード結果のシーケンスヘッダーから色空間メタデータを取得できるが、
現在の `DecodedFrame` にはこれらへのアクセサがない。
まず対応する Rust の enum 型を定義する必要がある。

## 追加すべき enum

### ColorPrimaries (H.273 準拠)

- `BT709` (1)
- `Unknown` (2)
- `BT470M` (4)
- `BT470BG` (5)
- `BT601` (6)
- `SMPTE240` (7)
- `Film` (8)
- `BT2020` (9)
- `XYZ` (10)
- `SMPTE431` (11)
- `SMPTE432` (12)
- `EBU3213` (22)

### TransferCharacteristics (H.273 準拠)

- `BT709` (1)
- `Unknown` (2)
- `BT470M` (4)
- `BT470BG` (5)
- `BT601` (6)
- `SMPTE240` (7)
- `Linear` (8)
- `Log100` (9)
- `Log100Sqrt10` (10)
- `IEC61966` (11)
- `BT1361` (12)
- `Srgb` (13)
- `BT2020_10bit` (14)
- `BT2020_12bit` (15)
- `SMPTE2084` (16) — PQ
- `SMPTE428` (17)
- `Hlg` (18)

### MatrixCoefficients (H.273 準拠)

- `Identity` (0)
- `BT709` (1)
- `Unknown` (2)
- `FCC` (4)
- `BT470BG` (5)
- `BT601` (6)
- `SMPTE240` (7)
- `SMPTEYCgCo` (8)
- `BT2020NCL` (9)
- `BT2020CL` (10)
- `SMPTE2085` (11)
- `ChromatNCL` (12)
- `ChromatCL` (13)
- `ICtCp` (14)

### ChromaSamplePosition

- `Unknown` (0)
- `Vertical` (1)
- `Colocated` (2)

### ColorRange

- `Studio` (0) — MPEG レンジ (8-bit: Y [16,235], C [16,240])
- `Full` (1) — JPEG レンジ (8-bit: [0,255])

## 根拠

色空間情報はレンダリングやトランスコードの際に不可欠なメタデータ。
特に HDR コンテンツでは `BT2020` + `SMPTE2084` (PQ) のような組み合わせを
正しく識別する必要がある。
dav1d のシーケンスヘッダーからこれらの情報が取得できるのに公開しないのは
ラッパーライブラリとして機能不足になる。

## 解決方法

`ColorPrimaries`, `TransferCharacteristics`, `MatrixCoefficients`, `ChromaSamplePosition`, `ColorRange` の 5 つの enum を定義し、
それぞれ dav1d の FFI 定数からの変換メソッド `from_raw()` を実装した。
