// bindgen が生成するバインディングは動的生成コードであり、
// 生成コードに lint を適用しない (aom-rs / svt-av1-rs と同じ構成)
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(improper_ctypes)]
#![allow(unnecessary_transmutes)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/metadata.rs"));
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
