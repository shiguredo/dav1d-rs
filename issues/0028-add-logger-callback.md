# dav1d のログ出力をアプリのロガーへ流せるようにし、破棄も選べるようにする

- Created: 2026-10-17
- Completed: {YYYY-MM-DD}
- Branch: feature/add-logger-callback
- Polished: {YYYY-MM-DD}
- Reporter: @voluntas

## 目的

dav1d が stderr へ直接書き出すログを、利用アプリのロガーへ集約できるようにし、不要なら破棄できるようにする。

実リレーの E2E で `Error parsing OBU data` が stderr に現れる。これは購読開始がキーフレームより前のデータを渡したときに dav1d が出すメッセージで、テストが通る以上デコーダーの欠陥ではない。しかし `shiguredo_dav1d` は `Dav1dSettings.logger` を公開していないため、利用側はこの出力の宛先も抑制も選べず、example の標準エラーが汚れる。

## 現状

- `src/lib.rs` の `DecoderConfig` は `Dav1dSettings` のうち `allocator` と `logger` を公開していない。README の「未対応の dav1d 機能」にも両方が記載されている
- `dav1d_default_settings()` は `s->logger.callback = dav1d_log_default_callback` を設定する。このコールバックは `vfprintf(stderr, format, ap)` を呼ぶだけなので、`shiguredo_dav1d` を経由しても出力先は stderr に固定される（dav1d 1.5.4 の `src/lib.c` と `src/log.c`）
- dav1d 1.5.4 の `src/obu.c` には `dav1d_log(c, "Error parsing OBU data\n")` をはじめ `Overrun in OBU bit buffer` / `Error parsing frame header` / `Unknown OBU type` などの出力がある。同梱の `libdav1d.a`（`1.5.4-0-g54706fc`）にこれらの文字列が含まれることを確認済み
- `Dav1dLogger.callback` の第 3 引数は `va_list` であり、Rust からは可変長引数を読めない。整形には `vsnprintf` が必要になる
- dav1d はログをワーカースレッドからも呼び出しうるため、コールバックにはスレッド間で共有できることが要求される
- `build.rs` の DOCS_RS 向けダミーバインディングの `Dav1dSettings` は `allocator` / `logger` / `reserved` を持たない。`Dav1dLogger` と `va_list` も未定義
- prebuilt バイナリはリリース時に生成した `bindings.rs` を同梱する。現在の prebuilt の `bindings.rs` には `Dav1dLogger` / `va_list` が含まれているため、新しいシンボルを要求しない限りリリースを待たずに実装できる

## 設計方針

- `DecoderConfig` にログ出力先を表すフィールドを追加する。値は次の 3 種類とする
  - 既定（dav1d の stderr 出力のまま。既存の挙動を変えない）
  - 破棄（`logger.callback` を `NULL` にする。`dav1d_log()` は callback が NULL なら即座に戻る）
  - コールバック（整形済みの文字列を受け取る）
- 既定を破棄にしない。デコードエラーの手掛かりを無言で捨てる変更は後方互換では済まないため、抑制は明示的な選択にする
- コールバックは `Arc<dyn Fn(&str) + Send + Sync>` を保持し、`Box` にして `logger.cookie` に渡す。`Decoder` がその `Box` を所有する
- `Decoder` の `Drop` は `dav1d_close()` を呼ぶ。close の最中にもワーカースレッドがログを出しうるため、cookie の解放は `dav1d_close()` が完了した後に行う
- `va_list` からの整形は自前のコールバックで行う。`vsnprintf` を固定長バッファ（1 KiB）に対して呼び、確保は行わない。可変長引数は Rust から直接扱えないため、この 1 段を挟まないと安全な API にできない
- `vsnprintf` は libc クレートが主要プラットフォーム（Linux / macOS / Windows / iOS / Android）で公開していないため、bindgen が生成する `sys::va_list` を使って `src/lib.rs` で宣言する
- コールバックへ渡す文字列からは dav1d が付ける末尾の改行を取り除く
- `DecoderConfig` は `Debug` と `Clone` を derive しているため、`Arc` を包む newtype を作り `Debug` を手動実装して derive を維持する
- DOCS_RS 向けダミーに `Dav1dLogger` / `va_list` / `Dav1dSettings.logger` を追加し、`build.rs` のダミー定義が `src/lib.rs` の参照を網羅するという既存の約束を守る

## 完了条件

- 既定の設定では従来どおり dav1d が stderr へ出力すること
- 破棄を選ぶと `Error parsing OBU data` などの出力が stderr に現れないこと
- コールバックを選ぶと整形済みのメッセージ（改行なし）がコールバックに渡ること
- 壊れた OBU を実際にデコードしてログが発生する経路をテストで確認すること（モックは使わない）
- `Decoder` の drop 後にコールバックが呼ばれないこと、および drop 時に cookie が解放されることがテストまたはコメントで担保されていること
- README の設定表と「未対応の dav1d 機能」を更新し、CHANGES.md に [ADD] を追記すること
- prebuilt 経路、`source-build` feature、DOCS_RS の 3 経路でビルドが通ること
