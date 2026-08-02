// bindgen が生成するバインディングはプラットフォームによって違反する lint が
// 異なる (macOS と Linux で生成される型名・定数名が違う) ため、
// expect ではなく allow で抑止する。
// expect は「必ず満たされる」場合のみ使えるが、生成物では保証できない。
// DOCS_RS 向けダミーバインディングは命名規約に従った定義であり
// lint が発生しないため、そのビルドでは improper_ctypes のみ追加で抑止する
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
// clippy::all は生成コードで必ず違反するとは限らないため expect は使えない
#![allow(clippy::all)]
// DOCS_RS 向けダミーバインディングの Dav1dContext / Dav1dData は
// フィールドを持たない不完全型の代用であり FFI-safe ではないが、
// ドキュメント生成専用のため問題ない
#![cfg_attr(docs_rs_dummy, allow(improper_ctypes))]

include!(concat!(env!("OUT_DIR"), "/metadata.rs"));
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
