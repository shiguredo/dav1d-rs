Created: 2026-03-19

# dav1d_data_wrap / dav1d_data_wrap_user_data に対応する機能を追加する

Model: Opus 4.6

## 概要

dav1d C API には `dav1d_data_wrap()` と `dav1d_data_wrap_user_data()` 関数があるが、
現在の Rust ラッパーにはこれらに対応する機能がない。
これらの関数は既存のバッファをゼロコピーでデコーダーに渡すための機能であり、
コピーを避けることでパフォーマンスを向上させる。

## pending にした理由

これらの関数はコールバック関数ポインタ (`free_callback`) と
生ポインタ (`cookie`) を引数に取るため、safe Rust API としての設計が困難である。
ライフタイム管理やメモリ安全性の保証を Rust 側で行う必要があり、
API 設計の検討が必要。

## 根拠

現在の `Decoder::decode()` はデータをコピーして `dav1d_data_create()` に渡しているが、
大量のフレームを処理する場合にはコピーのオーバーヘッドが無視できなくなる可能性がある。
ゼロコピーの入力パスを提供することでパフォーマンスを改善できる。
