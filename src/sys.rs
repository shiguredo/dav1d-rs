// bindgen が生成するバインディングはこれらの lint に必ず違反するため、
// 期待通りに発生する警告として expect で抑止する。
// DOCS_RS 向けダミーバインディングは命名規約に従った定義であり
// lint が発生しないため、そのビルドでは allow に切り替える
#![expect(non_upper_case_globals)]
#![cfg_attr(docs_rs_dummy, allow(non_camel_case_types, non_snake_case, dead_code))]
#![cfg_attr(
    not(docs_rs_dummy),
    expect(non_camel_case_types, non_snake_case, dead_code)
)]
// clippy::all は生成コードで必ず違反するとは限らないため expect は使えない
#![allow(clippy::all)]
// DOCS_RS 向けダミーバインディングの Dav1dContext / Dav1dData は
// フィールドを持たない不完全型の代用であり FFI-safe ではないが、
// ドキュメント生成専用のため問題ない
#![cfg_attr(docs_rs_dummy, allow(improper_ctypes))]

include!(concat!(env!("OUT_DIR"), "/metadata.rs"));
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
