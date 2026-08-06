# dav1d-rs

[![crates.io](https://img.shields.io/crates/v/shiguredo_dav1d.svg)](https://crates.io/crates/shiguredo_dav1d)
[![docs.rs](https://docs.rs/shiguredo_dav1d/badge.svg)](https://docs.rs/shiguredo_dav1d)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![GitHub Actions](https://github.com/shiguredo/dav1d-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/shiguredo/dav1d-rs/actions/workflows/ci.yml)
[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?logo=discord&logoColor=white)](https://discord.gg/shiguredo)

## About Shiguredo's open source software

We will not respond to PRs or issues that have not been discussed on Discord. Also, Discord is only available in Japanese.

Please read <https://github.com/shiguredo/oss> before use.

## 時雨堂のオープンソースソフトウェアについて

利用前に <https://github.com/shiguredo/oss> をお読みください。

## 概要

[dav1d](https://code.videolan.org/videolan/dav1d) を利用した AV1 デコーダーの Rust バインディングです。

## 特徴

- AV1 デコーダー
- 全ピクセルレイアウト対応 (I400, I420, I422, I444)
- ハイビット深度対応 (8-bit / 10-bit / 12-bit)
- シンボル名書き換えによるシンボル衝突回避 (`shiguredo_dav1d_` プレフィックス付与)
- prebuilt バイナリによる高速ビルド (デフォルト)
- ソースからのビルドも可能 (`--features source-build`)

## 動作要件

- Ubuntu 26.04 x86_64
- Ubuntu 26.04 arm64
- Ubuntu 24.04 x86_64
- Ubuntu 24.04 arm64
- Ubuntu 22.04 x86_64
- Ubuntu 22.04 arm64
- macOS 26 arm64
- macOS 15 arm64
- Windows 11 x86_64
- Windows Server 2025 x86_64

macOS 26 と macOS 15 は同一の prebuilt バイナリ (`macos_arm64`) を共用し、
Windows 11 と Windows Server 2025 も同一の prebuilt バイナリ (`windows_x86_64`) を共用する。

### ソースビルド時の追加要件

- Git
- C コンパイラ (`build-essential` 等)
- Meson + Ninja (dav1d のビルドシステム)
- NASM (dav1d のアセンブリ最適化に必要)
- Visual Studio (Windows のみ、MSVC コンパイラが必要)

```bash
# Ubuntu
sudo apt-get install -y build-essential meson ninja-build nasm

# macOS
brew install meson nasm

# Windows (Visual Studio がインストール済みであること)
pip install meson
choco install ninja nasm -y
```

## ビルド

デフォルトでは GitHub Releases から prebuilt バイナリをダウンロードしてビルドします。

```bash
cargo build
```

### ソースからビルド

dav1d をソースからビルドする場合は `source-build` feature を有効にしてください。

```bash
cargo build --features source-build
```

### docs.rs 向けビルド

dav1d がない環境では、docs.rs 向けのドキュメント生成のみ可能です。

```bash
DOCS_RS=1 cargo doc --no-deps
```

## 使い方

### デコード

```rust
use shiguredo_dav1d::{Decoder, DecoderConfig};

// デコーダーを作成
let config = DecoderConfig::new();
let mut decoder = Decoder::new(config)?;

// 圧縮データをデコード
// EAGAIN が返された場合は渡したデータが破棄されるため、
// フレームを取り出してから同じデータを再度渡す
loop {
    if let Err(e) = decoder.decode(&compressed_data) {
        if !e.is_eagain() {
            return Err(e);
        }
        // 内部バッファを空にしてから同じデータを再送する
        while let Ok(Some(_)) = decoder.next_frame() {}
        continue;
    }
    break;
}

// デコード済みフレームを取得
while let Ok(Some(frame)) = decoder.next_frame() {
    let y = frame.y_plane();
    let u = frame.u_plane();
    let v = frame.v_plane();
    let layout = frame.pixel_layout();
    let bit_depth = frame.bit_depth();
    println!(
        "{}x{} {:?} {}bpc",
        frame.width(), frame.height(), layout, bit_depth
    );
}

// 残りのフレームをフラッシュ
decoder.finish()?;
while let Ok(Some(frame)) = decoder.next_frame() {
    // ...
}
```

## 設定

### `DecoderConfig`

| フィールド | 型 | デフォルト | 説明 |
|---|---|---|---|
| `n_threads` | `usize` | 1 | デコードに使用するスレッド数 (0 で論理コア数による自動決定) |
| `max_frame_delay` | `usize` | 0 | 最大フレーム遅延 (0 で `ceil(sqrt(n_threads))`、1 で低遅延デコード) |
| `apply_grain` | `bool` | true | フィルムグレインを適用するか |
| `operating_point` | `usize` | 0 | スケーラブル AV1 のオペレーティングポイント (0-31) |
| `all_layers` | `bool` | true | 全空間レイヤーを出力するか |
| `frame_size_limit` | `Option<u32>` | None | 最大フレームサイズ制限 (None で無制限) |
| `strict_std_compliance` | `bool` | false | ビットストリームのデコードに影響しない規格違反 (不整合・無効なメタデータ等) のときにデコードを中断するか |
| `output_invisible_frames` | `bool` | false | 非表示フレームも符号化順で出力するか (show-existing-frame により同じフレームが 2 回出力されうる) |
| `inloop_filters` | `InloopFilterType` | ALL | 有効にするインループフィルター |
| `decode_frame_type` | `DecodeFrameType` | All | デコードするフレーム種別 |

### 未対応の dav1d 機能

- カスタムピクセルアロケータ (`Dav1dSettings.allocator`)
- ログコールバック (`Dav1dSettings.logger`)
- ゼロコピー入力 (`dav1d_data_wrap` / `dav1d_data_wrap_user_data`)

### `PixelLayout`

| バリアント | 説明 |
|---|---|
| `I400` | モノクロ (Y のみ) |
| `I420` | YUV 4:2:0 |
| `I422` | YUV 4:2:2 |
| `I444` | YUV 4:4:4 |

### `Decoder`

| メソッド | 戻り値 | 説明 |
|---|---|---|
| `new(config)` | `Result<Self, Error>` | デコーダーを生成する |
| `decode(data)` | `Result<(), Error>` | 圧縮データをデコードする |
| `next_frame()` | `Result<Option<DecodedFrame>, Error>` | デコード済みフレームを取得する |
| `finish()` | `Result<(), Error>` | ストリーム終了を通知する |
| `flush()` | `()` | デコーダーの内部状態をリセットする (シーク用) |
| `apply_grain(frame)` | `Result<DecodedFrame, Error>` | フレームにフィルムグレインを適用する |
| `get_event_flags()` | `Result<EventFlags, Error>` | デコードイベントフラグを取得する |
| `get_decode_error_data_props()` | `Result<DataProps, Error>` | デコードエラーの詳細を取得する |

### `DecodedFrame`

| メソッド | 戻り値 | 説明 |
|---|---|---|
| `pixel_layout()` | `PixelLayout` | ピクセルレイアウト |
| `bit_depth()` | `usize` | ビット深度 (8, 10, 12) |
| `is_high_depth()` | `bool` | 10-bit 以上かどうか |
| `width()` | `usize` | フレームの幅 |
| `height()` | `usize` | フレームの高さ |
| `y_plane()` | `&[u8]` | Y 成分のデータ |
| `u_plane()` | `&[u8]` | U 成分のデータ (I400 の場合は空) |
| `v_plane()` | `&[u8]` | V 成分のデータ (I400 の場合は空) |
| `y_plane_u16()` | `Option<&[u16]>` | Y 成分のデータ (ハイビット深度用) |
| `u_plane_u16()` | `Option<&[u16]>` | U 成分のデータ (ハイビット深度用) |
| `v_plane_u16()` | `Option<&[u16]>` | V 成分のデータ (ハイビット深度用) |
| `y_stride()` | `usize` | Y 成分のストライド (バイト単位) |
| `u_stride()` | `usize` | U 成分のストライド (バイト単位) |
| `v_stride()` | `usize` | V 成分のストライド (バイト単位) |
| `frame_type()` | `Option<FrameType>` | フレーム種別 (メタデータがない場合は `None`) |
| `temporal_id()` | `Option<u8>` | SVC 用テンポラル ID (メタデータがない場合は `None`) |
| `spatial_id()` | `Option<u8>` | SVC 用スパーシャル ID (メタデータがない場合は `None`) |
| `show_frame()` | `Option<bool>` | 表示フレームかどうか (メタデータがない場合は `None`) |
| `color_primaries()` | `Option<ColorPrimaries>` | 色域 (メタデータがない場合は `None`) |
| `transfer_characteristics()` | `Option<TransferCharacteristics>` | 伝達特性 (メタデータがない場合は `None`) |
| `matrix_coefficients()` | `Option<MatrixCoefficients>` | 行列係数 (メタデータがない場合は `None`) |
| `chroma_sample_position()` | `Option<ChromaSamplePosition>` | クロマサンプル位置 (メタデータがない場合は `None`) |
| `color_range()` | `Option<ColorRange>` | 色域レンジ (メタデータがない場合は `None`) |
| `profile()` | `Option<u8>` | AV1 プロファイル (0, 1, 2) (メタデータがない場合は `None`) |
| `content_light_level()` | `Option<ContentLightLevel>` | HDR コンテンツライトレベル |
| `mastering_display()` | `Option<MasteringDisplay>` | HDR マスタリングディスプレイ情報 |

### `SequenceHeader`

| フィールド | 型 | 説明 |
|---|---|---|
| `profile` | `u8` | AV1 プロファイル (0, 1, 2) |
| `max_width` | `usize` | ストリームの最大幅 |
| `max_height` | `usize` | ストリームの最大高さ |
| `layout` | `PixelLayout` | ピクセルレイアウト |
| `pri` | `ColorPrimaries` | 色域 |
| `trc` | `TransferCharacteristics` | 伝達特性 |
| `mtrx` | `MatrixCoefficients` | 行列係数 |
| `chr` | `ChromaSamplePosition` | クロマサンプル位置 |
| `bit_depth` | `usize` | ビット深度 (8, 10, 12) |
| `color_range` | `ColorRange` | 色域レンジ |

### `Error`

| メソッド | 戻り値 | 説明 |
|---|---|---|
| `is_eagain()` | `bool` | EAGAIN エラーかどうか |

### 関数

| 関数 | 戻り値 | 説明 |
|---|---|---|
| `version()` | `&'static str` | dav1d ランタイムバージョン文字列 |
| `version_api()` | `u32` | dav1d API バージョン (数値) |
| `parse_sequence_header(data)` | `Result<SequenceHeader, Error>` | デコーダーなしにシーケンスヘッダーを解析する |

## 環境変数

| 変数 | 説明 |
|---|---|
| `DAV1D_TARGET` | prebuilt バイナリのプラットフォーム名を明示的に指定する |

## リリース手順

### canary リリース

1. `canary.py` でバージョンをインクリメントし、コミット・タグ・プッシュを実行する

```bash
python3 canary.py
```

- バージョンは `2026.2.0-canary.2` → `2026.2.0-canary.3` のように更新される
- タグのプッシュにより、GitHub Actions の `release.yml` が GitHub Release の作成、全プラットフォームの prebuilt ビルド、crates.io への公開まで自動実行する
- タグのプッシュは `develop` または `release/` ブランチからのみ実行できる

### 正式リリース

1. `release/YYYY.M.P` ブランチを `develop` から作成する

```bash
git checkout -b release/2026.2.0 develop
```

2. `canary.py --release` で canary バージョンを正式リリース版に変換する

```bash
python3 canary.py --release
```

- バージョンは `2026.2.0-canary.2` → `2026.2.0` のように変換される
- `CHANGES.md` の `## develop` セクションが `## 2026.2.0` とリリース日に更新される
- コミット・タグ・プッシュまで自動で実行される

3. プッシュ後、GitHub Actions の `release.yml` が以下を自動実行する

- GitHub Release の作成
- 全 8 プラットフォームの prebuilt バイナリのビルドとアップロード
- crates.io への公開

4. リリース後、`release/` ブランチを `develop` にマージする

## dav1d ライセンス

<https://code.videolan.org/videolan/dav1d/-/blob/master/COPYING>

```text
Copyright © 2018-2025, VideoLAN and dav1d authors
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR
ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```

## ライセンス

Apache License 2.0

```text
Copyright 2026, Shiguredo Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
