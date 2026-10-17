# dav1d のログ出力をアプリのロガーへ流せるようにし、破棄も選べるようにする

- Created: 2026-10-17
- Completed: {YYYY-MM-DD}
- Branch: feature/add-logger-callback
- Polished: 2026-10-17
- Reporter: @voluntas

## 目的

dav1d が stderr へ直接書き出すログを、利用アプリのロガーへ集約できるようにし、不要なら破棄できるようにする。

実リレーの E2E で `Error parsing OBU data` が stderr に現れる。これは購読開始がキーフレームより前のデータを渡したときに dav1d が出すメッセージで、テストが通る以上デコーダーの欠陥ではない。しかし `shiguredo_dav1d` は `Dav1dSettings.logger` を公開していないため、利用側はこの出力の宛先も抑制も選べず、example の標準エラーが汚れる。

## 現状

- `src/lib.rs` の `DecoderConfig` は `Dav1dSettings` のうち `allocator` と `logger` を公開していない。README の「未対応の dav1d 機能」にも両方が記載されている
- `dav1d_default_settings()` は `s->logger.callback = dav1d_log_default_callback` を設定する。このコールバックは `vfprintf(stderr, format, ap)` を呼ぶだけなので、`shiguredo_dav1d` を経由しても出力先は stderr に固定される（dav1d 1.5.4 の `src/lib.c` と `src/log.c`）
- dav1d 1.5.4 の `src/obu.c` には `dav1d_log(c, "Error parsing OBU data\n")` をはじめ `Overrun in OBU bit buffer` / `Error parsing frame header` / `Unknown OBU type` などの出力がある。リリース資産 `dav1d-macos_arm64.tar.gz`（2026.3.0）の `libdav1d.a` にこれらの文字列が含まれることを確認済み
- ログはメインスレッドとワーカースレッドの両方から出る。`src/obu.c` の `dav1d_parse_obus()` は `dav1d_send_data()` / `dav1d_get_picture()` から `gen_picture()` 経由でメインスレッドが呼び、`src/decode.c` の `dav1d_decode_frame_init()` は `src/thread_task.c` からワーカースレッドが呼ぶ
- `Dav1dLogger.callback` の第 3 引数は `va_list` であり、Rust からは可変長引数を読めない。整形には `vsnprintf` が必要になる
- `va_list` のバインディング上の型はプラットフォームで異なる。2026.3.0 のリリース資産に含まれる `bindings.rs` で確認した実際の型は次のとおり

  | ターゲット | コールバックの `ap` の型 | `sys::va_list` |
  |---|---|---|
  | ubuntu-22.04 / 24.04 / 26.04 x86_64 | `*mut __va_list_tag` | `[__va_list_tag; 1]` |
  | android_x86_64 | `*mut __va_list_tag` | `[__va_list_tag; 1]` |
  | ubuntu-*.04 arm64 / android_arm64 | `va_list` | `__BindgenOpaqueArray<u64, 4>` |
  | macos_arm64 / ios_arm64 | `va_list` | `*mut c_char` |
  | windows_x86_64 | `va_list` | `*mut c_char` |

- libc クレートは上記の対応プラットフォームで `va_list` と `vsnprintf` を公開していない（`teeos` / `solid` / `qurt` など一部のターゲットのみ）
- `build.rs` の DOCS_RS 向けダミーバインディングの `Dav1dSettings` は `allocator` / `logger` / `reserved` を持たず、`Dav1dLogger` / `va_list` / `__va_list_tag` も未定義
- 現在の prebuilt バイナリはリリース時に生成した `bindings.rs` を同梱する。2026.3.0 の全リリース資産（`ubuntu-22.04` / `ubuntu-24.04` / `ubuntu-26.04` の x86_64 と arm64、`macos_arm64`、`windows_x86_64`、`ios_arm64` / `ios-sim_arm64`、`android_arm64` / `android_x86_64`）の `bindings.rs` に `Dav1dLogger` / `va_list` / `__va_list_tag` が含まれることを確認済み。新しい dav1d シンボルを要求しない限り、リリースを待たずに prebuilt 経路でも実装できる

## 設計方針

### 公開 API

`DecoderConfig` にログ出力の扱いを表すフィールドを追加する。

```rust
pub struct DecoderConfig {
    // 既存フィールドは変更しない
    /// dav1d のログ出力の扱い
    pub logger: Logger,
}

/// dav1d のログ出力の扱い
#[derive(Clone)]
pub enum Logger {
    /// dav1d 既定の stderr 出力（既定値）
    Stderr,
    /// ログを破棄する
    Discard,
    /// 整形済みのメッセージをコールバックへ渡す
    Callback(Arc<dyn Fn(&str) + Send + Sync>),
}
```

- 既定は `Logger::Stderr` とし、既存の挙動を変えない。デコードエラーの手掛かりを無言で捨てる変更は後方互換では済まないため、抑制は明示的な選択にする
- `DecoderConfig` は `Debug` と `Clone` を derive している。`Arc<dyn Fn>` は `Debug` を実装しないため、`Logger` の `Debug` だけ手動実装する（`Callback(..)` は中身を出さない）。`Clone` は `Arc` の clone で成立するため derive のまま
- コールバックは `Send + Sync` を要求する。ワーカースレッドから呼ばれるため
- `'static` を要求する。`Decoder` が `dav1d_close()` まで保持するため

### va_list の扱い

- コールバックの第 3 引数の型はプラットフォームで異なるため、`src/lib.rs` に private の型エイリアスを置き、`cfg` で切り替える
  - x86_64 の Linux / Android: `*mut sys::__va_list_tag`（C では配列型の `va_list` が関数引数でポインタへ減衰する）
  - それ以外の対応ターゲット: `sys::va_list`
- `vsnprintf` は libc クレートが公開していないため、`src/lib.rs` で自前の `unsafe extern "C"` 宣言を行う。引数の `ap` には上記の型エイリアスを使う。宣言が C 側の実引数と食い違えばコンパイル時に落ちるため、型のずれはビルドで検出できる
- 整形は private の `unsafe extern "C" fn` で行う。1 KiB の固定長バッファへ `vsnprintf` し、戻り値が負なら何もせず戻る。確保は行わない
- コールバックへ渡す文字列からは dav1d が付ける末尾の改行を取り除く。UTF-8 として解釈できないメッセージは捨てる（dav1d のメッセージはすべて ASCII）
- 1 KiB を超えるメッセージは切り詰められる。dav1d 1.5.4 のメッセージは `Unknown OBU type %d of size %td` のような短いものしかなく、現状 1 KiB を超えるものはない

### cookie の寿命

- コールバックを `Arc` から `Box` に取り出し、その `Box` のポインタを `logger.cookie` に渡す。`Decoder` がその `Box` を所有する
- `Decoder` の `Drop` は `dav1d_close()` を呼ぶ。close の完了前に cookie を解放すると、close の最中にワーカースレッドがログを出した場合に解放済みメモリを読む恐れがあるため、解放は `dav1d_close()` の完了後に行う。フィールドの宣言順で `ctx` の後に cookie 所有者を置く
- `Decoder` は `Debug` を derive しているため、cookie 所有者には `Debug` を手動実装する

### 契約としてドキュメントに書くこと

- コールバックはデコード中の任意のタイミングで、メインスレッドとワーカースレッドの両方から呼ばれる
- コールバックの中でデコーダーのメソッドを呼ばないこと（再入の保証はしない）
- コールバックがパニックすると `extern "C"` の境界を越えるためプロセスが abort する
- コールバックが長時間ブロックするとデコードが止まる

### テスト

- 統合テスト相当（`src/lib.rs` の `#[cfg(test)]`）: 既存テストの 640x480 黒フレームのストリームを途中で切ったデータを `Decoder::decode()` に渡し、`Logger::Callback` が空でないメッセージを受け取ることを検証する。dav1d は切り詰められた OBU に対して `Overrun in OBU bit buffer` または `Error parsing OBU data` を出す
- 単体テスト: `Logger::Stderr` は `dav1d_default_settings()` が設定したコールバックを変更せず、`Logger::Discard` は `callback` を `None` にし、`Logger::Callback` は `Some` と cookie を設定することを検証する。dav1d の `dav1d_log()` は `callback` が `NULL` なら即座に戻る（`src/log.c`）ため、この検証で抑制の配線を確認できる
- モックやスタブは使わない。実際の `Decoder` と実際の dav1d を通す

### ドキュメントとダミーバインディング

- README の `DecoderConfig` 設定表に `logger` を追加し、「未対応の dav1d 機能」からログコールバックの行を削除する
- `build.rs` の DOCS_RS 向けダミーに `Dav1dLogger` / `__va_list_tag` / `va_list` と `Dav1dSettings.logger` を追加する。`Dav1dSettings` のダミーは `reserved` も含めて実物の並びに寄せる
- CHANGES.md の `## develop` に `[ADD]` を追記する

## 完了条件

- 既定の設定（`Logger::Stderr`）では従来どおり dav1d が stderr へ出力すること
- `Logger::Discard` を選ぶと dav1d のコールバックが `None` になり、`Error parsing OBU data` などの出力が stderr に現れないこと
- `Logger::Callback` を選ぶと整形済みのメッセージ（改行なし、`Error parsing OBU data` または `Overrun in OBU bit buffer` を含む）がコールバックに渡ること
- 壊れた入力でログが実際に発生することを、実際の `Decoder` を使ったテストで確認すること
- `Logger::Stderr` / `Logger::Discard` / `Logger::Callback` の設定反映を単体テストで確認すること
- `Decoder` の drop と cookie の解放順がコメントで説明されていること
- README の設定表と「未対応の dav1d 機能」、`build.rs` の DOCS_RS ダミー、CHANGES.md が追従していること
- prebuilt 経路、`source-build` feature、DOCS_RS の 3 経路でビルドが通ること
- `cargo clippy --workspace --all-targets --features source-build -- -D warnings` と `cargo test --workspace --features source-build` が通ること
