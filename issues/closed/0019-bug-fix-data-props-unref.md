# get_decode_error_data_props() で Dav1dDataProps の参照を解放する

- Created: 2026-08-02
- Completed: 2026-08-06
- Branch: feature/fix-data-props-unref
- Polished: {YYYY-MM-DD}
- Reporter: @voluntas

## reopened にした理由

- 起票時に誤って `issues/pending/` 直下に作成されていたため、`issues/` 直下の open 状態に戻す
- 仕様的に対応が難しい、設計判断が必要などの pending にする理由がない
## 目的

dav1d の所有権契約（呼び出し側が参照の所有権を引き継ぐ）に従い、`Dav1dDataProps` の `user_data_ref` のリーク経路を断つ。

## 現状

- `src/lib.rs` の `Decoder::get_decode_error_data_props()` は `dav1d_get_decode_error_data_props()` の戻り値を `assume_init()` でコピーした後、`dav1d_data_props_unref()` を呼ばずに破棄している
- dav1d.h の仕様は「On success, the caller assumes ownership of the returned reference」であり、dav1d 実装（`src/lib.c` の `dav1d_get_decode_error_data_props`）は `cached_error_props` をコピーしてデフォルトにリセットする。コピー側の `user_data_ref` は呼び出し側が `dav1d_data_props_unref()` で解放する必要がある
- 現状は `dav1d_data_wrap_user_data` を使う公開 API がないため `user_data_ref` は常に NULL で、リークは顕在化していない。ただし issues/pending/0013 の実装と同時に、デコードエラーごとに user_data バッファが永久リークする

## 設計方針

- dav1d の所有権契約に従い、`Dav1dDataProps` をコピーした後に `dav1d_data_props_unref()` を呼ぶ

## 完了条件

- `get_decode_error_data_props()` が返した後も `user_data_ref` が解放され、user_data のリークが発生しないこと

## 解決方法

- `src/lib.rs` の `Decoder::get_decode_error_data_props()` で、`DataProps` への変換後に `sys::dav1d_data_props_unref(&mut props)` を呼ぶようにした
- `dav1d_data_props_unref()` は `Dav1dDataProps` を memset して初期状態に戻すことを dav1d の実装で確認し、コメントに記載した
