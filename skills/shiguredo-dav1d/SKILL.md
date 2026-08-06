---
name: shiguredo-dav1d
description: >-
  時雨堂の dav1d (AV1) Rust バインディング shiguredo_dav1d を利用するためのリファレンス。
  Cargo への追加、prebuilt / source-build の選択、AV1 デコード、EAGAIN 処理、
  DecodedFrame のプレーン / メタデータ、シーケンスヘッダー解析、フィルムグレインに関する質問時に使用。
---

# shiguredo_dav1d

時雨堂が公開している [dav1d](https://github.com/videolan/dav1d) ベースの AV1 デコーダー Rust バインディング。

## バージョン情報

- crate 名: `shiguredo_dav1d`
- crate バージョン: 2026.2.0
- dav1d バージョン: 1.5.4
- Rust Edition: 2024
- 最小 Rust バージョン: 1.93
- ライセンス: Apache-2.0
- crates.io: <https://crates.io/crates/shiguredo_dav1d>
- API ドキュメント: <https://docs.rs/shiguredo_dav1d>

## 対応機能

- AV1 デコード
- I400、I420、I422、I444 のピクセルレイアウト
- 8-bit、10-bit、12-bit のビット深度
- スケーラブル AV1 のオペレーティングポイントと空間レイヤー
- 色空間、フレーム種別、SVC、HDR のメタデータ取得
- デコーダーを生成しないシーケンスヘッダー解析
- フィルムグレインの自動適用と後段適用
- シーク後のデコーダー状態のリセット

dav1d はソフトウェアデコーダー専用である。
AV1 エンコードとハードウェアアクセラレーションは提供しない。

同梱する静的ライブラリのシンボルには `shiguredo_dav1d_` プレフィックスが付く。
別の dav1d 利用ライブラリと同一プロセス内で共存できる。

## Cargo への追加

デフォルトでは、GitHub Releases からダウンロードした prebuilt バイナリを使用する。

```toml
[dependencies]
# AV1 デコードに使用する
shiguredo_dav1d = "2026.2"
```

dav1d をソースからビルドする場合は、`source-build` feature を有効にする。

```toml
[dependencies]
# dav1d をソースからビルドして AV1 デコードに使用する
shiguredo_dav1d = { version = "2026.2", features = ["source-build"] }
```

## ビルド方式

### prebuilt バイナリ

デフォルトのビルドは、crate と同じバージョンの GitHub Release から静的ライブラリを取得する。
アーカイブは SHA-256 チェックサムで検証される。

```bash
cargo build
```

対応環境は次のとおり。

| 環境 | アーキテクチャ |
|---|---|
| Ubuntu 26.04 | x86_64 / arm64 |
| Ubuntu 24.04 | x86_64 / arm64 |
| Ubuntu 22.04 | x86_64 / arm64 |
| macOS 26 / 15 | arm64 |
| Windows 11 / Windows Server 2025 | x86_64 |

デフォルトビルドでは `curl`、`tar` と OS ごとの SHA-256 計算コマンドを使用する。

`DAV1D_TARGET` 環境変数を使うと、取得する prebuilt バイナリを明示できる。

```bash
DAV1D_TARGET=ubuntu-24.04_x86_64 cargo build
```

指定できる値は次のとおり。

- `ubuntu-26.04_x86_64`
- `ubuntu-26.04_arm64`
- `ubuntu-24.04_x86_64`
- `ubuntu-24.04_arm64`
- `ubuntu-22.04_x86_64`
- `ubuntu-22.04_arm64`
- `macos_arm64`
- `windows_x86_64`

`DAV1D_TARGET` は自動判定を上書きする。
実行環境と ABI が一致する値を指定する。

### ソースビルド

prebuilt バイナリに対応していない環境では、`source-build` feature を使用する。
`Cargo.toml` で `features = ["source-build"]` を指定してから、通常どおりビルドする。

```bash
cargo build
```

ソースビルドには次のツールが必要である。

- Git
- C コンパイラ
- Meson
- Ninja
- NASM
- rustup の `llvm-tools` コンポーネント
- Windows では Visual Studio と MSVC

```bash
# Ubuntu
sudo apt-get install -y build-essential meson ninja-build nasm
rustup component add llvm-tools

# macOS
brew install meson nasm
rustup component add llvm-tools

# Windows (Visual Studio はインストール済みであること)
pip install meson
choco install ninja nasm -y
rustup component add llvm-tools
```

## 基本的なデコード

`decode()` へ AV1 データを渡し、`next_frame()` が `None` を返すまでデコード済みフレームを取り出す。
すべての入力を渡した後は、`finish()` を呼んでから残りのフレームを取り出す。

```rust
use shiguredo_dav1d::{DecodedFrame, Decoder, DecoderConfig, Error};

// 現在出力できるフレームをすべて取り出す
fn drain<F>(decoder: &mut Decoder, consume: &mut F) -> Result<(), Error>
where
    F: FnMut(DecodedFrame),
{
    while let Some(frame) = decoder.next_frame()? {
        consume(frame);
    }
    Ok(())
}

// 1 個の入力を、EAGAIN の再送を含めてデコードする
fn decode_packet<F>(
    decoder: &mut Decoder,
    data: &[u8],
    consume: &mut F,
) -> Result<(), Error>
where
    F: FnMut(DecodedFrame),
{
    loop {
        match decoder.decode(data) {
            Ok(()) => break,
            Err(error) if error.is_eagain() => {
                drain(decoder, consume)?;
            }
            Err(error) => return Err(error),
        }
    }

    drain(decoder, consume)
}

fn decode_all(compressed_packets: &[Vec<u8>]) -> Result<(), Error> {
    let mut decoder = Decoder::new(DecoderConfig::new())?;
    let mut consume = |frame: DecodedFrame| {
        println!(
            "{}x{} {:?} {}bpc",
            frame.width(),
            frame.height(),
            frame.pixel_layout(),
            frame.bit_depth()
        );
    };

    for packet in compressed_packets {
        decode_packet(&mut decoder, packet, &mut consume)?;
    }

    decoder.finish()?;
    drain(&mut decoder, &mut consume)?;
    Ok(())
}
```

## EAGAIN の処理

`decode()` は入力を dav1d の管理するバッファーへコピーする。
空のスライスを渡した場合は、何もせず `Ok(())` を返す。

dav1d に未消費データが残っている場合、`decode()` は EAGAIN を返す。
その呼び出しで dav1d に渡したコピーは破棄されるため、`next_frame()` で出力を取り出してから、呼び出し側が保持している同じ入力を再度渡す。

EAGAIN 以外のエラーでも、その呼び出しで渡したコピーは破棄される。
EAGAIN 以外のエラーでは同じ入力を自動的に再送しない。

`next_frame()` は、dav1d 内部の EAGAIN を `Ok(None)` に変換する。
`while let Ok(Some(frame)) = decoder.next_frame()` のように書くと EAGAIN 以外のエラーも失うため、利用例のように `?` でエラーを伝播する。

`finish()` 自体はデコーダーの状態を変更しない。
すべての入力後に `finish()` を呼び、その後で `next_frame()` を再度呼ぶと、遅延していたフレームを取得できる。

## デコーダー設定

```rust
pub struct DecoderConfig {
    pub n_threads: usize,
    pub max_frame_delay: usize,
    pub apply_grain: bool,
    pub operating_point: usize,
    pub all_layers: bool,
    pub frame_size_limit: Option<u32>,
    pub strict_std_compliance: bool,
    pub output_invisible_frames: bool,
    pub inloop_filters: InloopFilterType,
    pub decode_frame_type: DecodeFrameType,
}
```

| フィールド | デフォルト | 説明 |
|---|---|---|
| `n_threads` | `1` | デコードに使用するスレッド数。`0` で論理コア数から自動決定 |
| `max_frame_delay` | `0` | 最大フレーム遅延。`0` で自動決定、`1` で低遅延デコード |
| `apply_grain` | `true` | デコード時にフィルムグレインを適用するか |
| `operating_point` | `0` | スケーラブル AV1 のオペレーティングポイント。`0..=31` |
| `all_layers` | `true` | すべての空間レイヤーを出力するか |
| `frame_size_limit` | `None` | 最大フレームサイズのピクセル数。`None` で無制限 |
| `strict_std_compliance` | `false` | デコード結果に影響しない規格違反でもデコードを中断するか |
| `output_invisible_frames` | `false` | 非表示フレームを符号化順で出力するか |
| `inloop_filters` | `InloopFilterType::ALL` | 有効にするインループフィルター |
| `decode_frame_type` | `DecodeFrameType::All` | デコード対象のフレーム種別 |

`DecoderConfig::default()` と `DecoderConfig::new()` は同じ設定を返す。

`DecoderConfig::frame_delay()` は、この設定で dav1d が使用するフレーム遅延を返す。
戻り値は 1 以上であり、`max_frame_delay == 0` の場合は dav1d が決めた `1..=8` の値になる。

### インループフィルター

```rust
InloopFilterType::NONE
InloopFilterType::DEBLOCK
InloopFilterType::CDEF
InloopFilterType::RESTORATION
InloopFilterType::ALL
```

`InloopFilterType` は `BitOr` を実装している。
デブロッキングと CDEF だけを有効にする場合は、次のように指定する。

```rust
config.inloop_filters = InloopFilterType::DEBLOCK | InloopFilterType::CDEF;
```

### デコードするフレーム種別

```rust
pub enum DecodeFrameType {
    All,
    Reference,
    Intra,
    Key,
}
```

- `All`：すべてのフレーム
- `Reference`：参照フレームだけ
- `Intra`：キーフレームを含むイントラフレームだけ
- `Key`：キーフレームだけ

## Decoder の API

```rust
impl Decoder {
    pub fn new(config: DecoderConfig) -> Result<Self, Error>;
    pub fn decode(&mut self, data: &[u8]) -> Result<(), Error>;
    pub fn next_frame(&mut self) -> Result<Option<DecodedFrame>, Error>;
    pub fn finish(&mut self) -> Result<(), Error>;
    pub fn flush(&mut self);
    pub fn apply_grain(&mut self, frame: &DecodedFrame) -> Result<DecodedFrame, Error>;
    pub fn get_event_flags(&mut self) -> Result<EventFlags, Error>;
    pub fn get_decode_error_data_props(&mut self) -> Result<DataProps, Error>;
}
```

### シーク後のリセット

`flush()` は未消費データとバッファー内のフレームを破棄する。
シーク後に `flush()` を呼び、シーケンスヘッダーを含むデータからデコードを再開する。

`flush()` 後の最初のピクチャには `EventFlags::NEW_SEQUENCE` が立つ。

### フィルムグレインの後段適用

`DecoderConfig::apply_grain` を `false` にすると、デコード時のフィルムグレイン適用を停止できる。
その状態で取得したフレームを、フレームを生成した `Decoder` の `apply_grain()` に渡すと、選択したフレームだけにフィルムグレインを適用できる。

```rust
let mut config = DecoderConfig::new();
config.apply_grain = false;
let mut decoder = Decoder::new(config)?;

// decoder から取得した frame に後からフィルムグレインを適用する
let grained_frame = decoder.apply_grain(&frame)?;
```

デフォルトの `apply_grain = true` では、`next_frame()` が返すフレームに適用済みである。
適用済みのフレームを `apply_grain()` に渡すと二重適用になる。

フレームにグレインメタデータがない場合、`apply_grain()` は入力と同じ画像を指す新しい参照を返す。

### イベントフラグ

```rust
pub struct EventFlags(/* ... */);

impl EventFlags {
    pub const NEW_SEQUENCE: Self;
    pub const NEW_OP_PARAMS_INFO: Self;
    pub fn contains(self, flag: Self) -> bool;
}
```

`NEW_SEQUENCE` は新しいシーケンスヘッダー、`NEW_OP_PARAMS_INFO` は新しいオペレーティングパラメーターを示す。
取得したフラグは、最後に `next_frame()` が返したフレームに対応する。
`get_event_flags()` を呼ぶと内部フラグがクリアされるため、必要な判定を戻り値に対して行う。

```rust
let flags = decoder.get_event_flags()?;
if flags.contains(EventFlags::NEW_SEQUENCE) {
    // 新しいシーケンスの開始を処理する
}
```

### デコードエラーの入力情報

```rust
pub struct DataProps {
    pub timestamp: i64,
    pub duration: i64,
    pub offset: i64,
    pub size: usize,
}
```

`get_decode_error_data_props()` は、最後のデコードエラーに関連する入力情報を返す。

- `timestamp`：コンテナーのタイムスタンプ。不明の場合は `i64::MIN`
- `duration`：コンテナーの再生時間。不明の場合は `0`
- `offset`：ストリーム内のオフセット。不明の場合は `-1`
- `size`：パケットサイズ

## DecodedFrame の API

```rust
impl DecodedFrame {
    pub fn pixel_layout(&self) -> PixelLayout;
    pub fn bit_depth(&self) -> usize;
    pub fn is_high_depth(&self) -> bool;

    pub fn y_plane(&self) -> &[u8];
    pub fn u_plane(&self) -> &[u8];
    pub fn v_plane(&self) -> &[u8];
    pub fn y_plane_u16(&self) -> Option<&[u16]>;
    pub fn u_plane_u16(&self) -> Option<&[u16]>;
    pub fn v_plane_u16(&self) -> Option<&[u16]>;

    pub fn y_stride(&self) -> usize;
    pub fn u_stride(&self) -> usize;
    pub fn v_stride(&self) -> usize;
    pub fn width(&self) -> usize;
    pub fn height(&self) -> usize;
}
```

`DecodedFrame` はデコード済み画像を所有する。
`DecodedFrame` を破棄すると画像の参照も解放される。

### ピクセルレイアウト

```rust
pub enum PixelLayout {
    I400,
    I420,
    I422,
    I444,
    Reserved,
}
```

- `I400`：Y のみ
- `I420`：YUV 4:2:0
- `I422`：YUV 4:2:2
- `I444`：YUV 4:4:4
- `Reserved`：未知または未対応のレイアウト

`Reserved` を受け取った場合、既知のレイアウトとして処理しない。

### プレーンとストライド

`y_plane()`、`u_plane()`、`v_plane()` が返すスライスには各行末のパディングが含まれる。
ストライドはバイト単位である。
有効画素だけを読む場合は、行ごとにストライドで移動し、有効幅に対応する部分だけを使用する。

```rust
let bytes_per_sample = if frame.is_high_depth() { 2 } else { 1 };
let active_row_bytes = frame.width() * bytes_per_sample;

for row in frame
    .y_plane()
    .chunks_exact(frame.y_stride())
    .take(frame.height())
{
    let active_pixels = &row[..active_row_bytes];
    consume_y_row(active_pixels);
}
```

クロマプレーンの有効寸法は次のとおり。

| レイアウト | 幅 | 高さ |
|---|---|---|
| `I400` | 0 | 0 |
| `I420` | `width.div_ceil(2)` | `height.div_ceil(2)` |
| `I422` | `width.div_ceil(2)` | `height` |
| `I444` | `width` | `height` |
| `Reserved` | 0 | 0 |

`u_plane()` と `v_plane()` は、`I400` と `Reserved` で空のスライスを返す。
`u_stride()` と `v_stride()` は `I400` と `Reserved` でも内部値を返すため、クロマプレーンの有無は `pixel_layout()` で判定する。

### ハイビット深度

10-bit と 12-bit のフレームは、1 ピクセルを 2 バイトで保持する。
画素値は下位ビット側に配置され、バイト単位のプレーンアクセサーでもストライドに 2 バイト分が反映される。

`y_plane_u16()`、`u_plane_u16()`、`v_plane_u16()` を使うと、画素を `u16` 単位で参照できる。
`u16` スライス上のストライドは、バイトストライドの半分である。

- `y_plane_u16()` は 8-bit フレームで `None` を返す
- `u_plane_u16()` と `v_plane_u16()` は 8-bit、`I400`、`Reserved` で `None` を返す

## フレームメタデータ

```rust
impl DecodedFrame {
    pub fn frame_type(&self) -> Option<FrameType>;
    pub fn temporal_id(&self) -> Option<u8>;
    pub fn spatial_id(&self) -> Option<u8>;
    pub fn show_frame(&self) -> Option<bool>;

    pub fn color_primaries(&self) -> Option<ColorPrimaries>;
    pub fn transfer_characteristics(&self) -> Option<TransferCharacteristics>;
    pub fn matrix_coefficients(&self) -> Option<MatrixCoefficients>;
    pub fn chroma_sample_position(&self) -> Option<ChromaSamplePosition>;
    pub fn color_range(&self) -> Option<ColorRange>;
    pub fn profile(&self) -> Option<u8>;

    pub fn content_light_level(&self) -> Option<ContentLightLevel>;
    pub fn mastering_display(&self) -> Option<MasteringDisplay>;
}
```

対応するフレームヘッダー、シーケンスヘッダー、HDR メタデータがない場合、各アクセサーは `None` を返す。

```rust
pub enum FrameType {
    Key,
    Inter,
    Intra,
    Switch,
    Unknown,
}

pub enum ColorRange {
    Studio,
    Full,
}

pub struct ContentLightLevel {
    pub max_content_light_level: u16,
    pub max_frame_average_light_level: u16,
}

pub struct MasteringDisplay {
    pub primaries: [[u16; 2]; 3],
    pub white_point: [u16; 2],
    pub max_luminance: u32,
    pub min_luminance: u32,
}
```

`ContentLightLevel` の輝度値は cd/m² 単位である。
`MasteringDisplay::primaries` と `white_point` は 0.16 固定小数点である。
`max_luminance` は 24.8 固定小数点、`min_luminance` は 18.14 固定小数点で、単位は cd/m² である。

### 色空間の列挙型

```rust
pub enum ColorPrimaries {
    BT709, Unknown, BT470M, BT470BG, BT601, SMPTE240, Film,
    BT2020, XYZ, SMPTE431, SMPTE432, EBU3213, Reserved,
}

pub enum TransferCharacteristics {
    BT709, Unknown, BT470M, BT470BG, BT601, SMPTE240, Linear,
    Log100, Log100Sqrt10, IEC61966, BT1361, Srgb, BT2020_10bit,
    BT2020_12bit, SMPTE2084, SMPTE428, Hlg, Reserved,
}

pub enum MatrixCoefficients {
    Identity, BT709, Unknown, FCC, BT470BG, BT601, SMPTE240,
    SMPTEYCgCo, BT2020NCL, BT2020CL, SMPTE2085, ChromatNCL,
    ChromatCL, ICtCp, Reserved,
}

pub enum ChromaSamplePosition {
    Unknown,
    Vertical,
    Colocated,
}
```

`ColorPrimaries`、`TransferCharacteristics`、`MatrixCoefficients` は H.273 に対応する。

## シーケンスヘッダー解析

`parse_sequence_header()` を使うと、デコーダーを生成せずにストリームのメタデータを取得できる。

```rust
pub fn parse_sequence_header(data: &[u8]) -> Result<SequenceHeader, Error>;

pub struct SequenceHeader {
    pub profile: u8,
    pub max_width: usize,
    pub max_height: usize,
    pub layout: PixelLayout,
    pub pri: ColorPrimaries,
    pub trc: TransferCharacteristics,
    pub mtrx: MatrixCoefficients,
    pub chr: ChromaSamplePosition,
    pub bit_depth: usize,
    pub color_range: ColorRange,
}
```

| フィールド | 説明 |
|---|---|
| `profile` | AV1 プロファイル。`0`、`1`、`2` のいずれか |
| `max_width` | ストリームの最大幅 |
| `max_height` | ストリームの最大高さ |
| `layout` | ピクセルレイアウト |
| `pri` | 色域 |
| `trc` | 伝達特性 |
| `mtrx` | 行列係数 |
| `chr` | クロマサンプル位置 |
| `bit_depth` | ビット深度 |
| `color_range` | 色域レンジ |

シーケンスヘッダー以外の OBU は無視される。
複数のシーケンスヘッダーがある場合は、最後の 1 件が返る。
`bit_depth` は `8`、`10`、`12` のいずれかである。

空の入力では EINVAL、シーケンスヘッダーがない入力では ENOENT に対応する `Error` が返る。

## エラー

```rust
pub struct Error { /* ... */ }

impl Error {
    pub fn is_eagain(&self) -> bool;
}
```

`Error` は `std::error::Error` と `Display` を実装する。
`Display` には失敗した dav1d 関数、エラーコード、既知の場合は EAGAIN または ENOMEM が含まれる。

エラーコードを取得する公開アクセサーはない。
EAGAIN の判定には `is_eagain()` を使用する。

## スレッド間での受け渡し

`Decoder` は `Send` だが `Sync` ではない。
デコーダーの所有権を別スレッドへ移動できるが、複数スレッドから同時に操作しない。

`DecodedFrame` は `Send` と `Sync` を実装する。
別スレッドへ移動でき、`Arc<DecodedFrame>` を使って複数スレッドから読み取れる。

## バージョンと対応情報

```rust
pub fn version() -> &'static str;
pub fn version_api() -> u32;

pub const BUILD_REPOSITORY: &str;
pub const BUILD_VERSION: &str;
```

- `version()`：リンクされている dav1d のバージョン文字列
- `version_api()`：`(major << 16) | (minor << 8) | patch` 形式の API バージョン
- `BUILD_REPOSITORY`：ビルド時に参照した dav1d リポジトリ URL
- `BUILD_VERSION`：ビルド時に参照した dav1d のバージョン

`version()` は dav1d のバージョン文字列が不正な UTF-8 の場合にパニックする。
dav1d のバージョン文字列は ASCII なので、通常は発生しない。

汎用的なコーデック選択を行うアプリケーションでは、`supported_codecs()` で対応情報を取得できる。

```rust
pub fn supported_codecs() -> Vec<CodecInfo>;

pub struct CodecInfo {
    pub codec: VideoCodecType,
    pub decoding: DecodingInfo,
    pub encoding: EncodingInfo,
}

pub struct DecodingInfo {
    pub supported: bool,
    pub hardware_accelerated: bool,
    pub profiles: DecodingProfiles,
}

pub struct EncodingInfo {
    pub supported: bool,
    pub hardware_accelerated: bool,
    pub profiles: EncodingProfiles,
}

pub enum VideoCodecType {
    Av1,
}

pub enum DecodingProfiles {
    Av1(Vec<Av1DecodingProfile>),
}

pub enum Av1DecodingProfile {
    Main,
    High,
    Professional,
}

pub enum EncodingProfiles {
    None,
}
```

返される情報は次のとおり。

- コーデック：`VideoCodecType::Av1`
- デコード：対応
- ハードウェアアクセラレーション：非対応
- デコードプロファイル：Main、High、Professional
- エンコード：非対応

## 利用時の制約

- カスタムピクセルアロケーターには対応しない
- ログコールバックには対応しない
- `dav1d_data_wrap` と `dav1d_data_wrap_user_data` を使うゼロコピー入力には対応しない
- `decode()` は入力を内部バッファーへコピーする
- `output_invisible_frames = true` では、show-existing-frame により同じフレームが 2 回出力される場合がある
- 必要なプレーンのポインターが null の場合、プレーンアクセサーはパニックする
- dav1d が負のストライドを返した場合、ストライドアクセサーはパニックする
