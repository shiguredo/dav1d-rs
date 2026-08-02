//! `src/lib.rs` のビットフラグ型 (InloopFilterType / EventFlags) の代数のプロパティテスト

use proptest::prelude::*;
use shiguredo_dav1d::{EventFlags, InloopFilterType};

/// InloopFilterType の全定数
const INLOOP_FILTERS: [InloopFilterType; 5] = [
    InloopFilterType::NONE,
    InloopFilterType::DEBLOCK,
    InloopFilterType::CDEF,
    InloopFilterType::RESTORATION,
    InloopFilterType::ALL,
];

/// EventFlags の全定数
const EVENT_FLAGS: [EventFlags; 2] = [EventFlags::NEW_SEQUENCE, EventFlags::NEW_OP_PARAMS_INFO];

proptest! {
    /// BitOr の結合則: (a | b) | c == a | (b | c)
    #[test]
    fn inloop_filter_bitor_associative(a in 0..INLOOP_FILTERS.len(), b in 0..INLOOP_FILTERS.len(), c in 0..INLOOP_FILTERS.len()) {
        let a = INLOOP_FILTERS[a];
        let b = INLOOP_FILTERS[b];
        let c = INLOOP_FILTERS[c];
        prop_assert_eq!((a | b) | c, a | (b | c));
    }

    /// BitOr の交換則: a | b == b | a
    #[test]
    fn inloop_filter_bitor_commutative(a in 0..INLOOP_FILTERS.len(), b in 0..INLOOP_FILTERS.len()) {
        let a = INLOOP_FILTERS[a];
        let b = INLOOP_FILTERS[b];
        prop_assert_eq!(a | b, b | a);
    }

    /// NONE は BitOr の単位元: a | NONE == a
    #[test]
    fn inloop_filter_bitor_identity(a in 0..INLOOP_FILTERS.len()) {
        let a = INLOOP_FILTERS[a];
        prop_assert_eq!(a | InloopFilterType::NONE, a);
    }

    /// ALL は BitOr の吸収元: a | ALL == ALL
    #[test]
    fn inloop_filter_bitor_absorbing(a in 0..INLOOP_FILTERS.len()) {
        let a = INLOOP_FILTERS[a];
        prop_assert_eq!(a | InloopFilterType::ALL, InloopFilterType::ALL);
    }

    /// contains の反射律: フラグは自分自身を含む
    #[test]
    fn event_flags_contains_reflexive(f in 0..EVENT_FLAGS.len()) {
        let flags = EVENT_FLAGS[f];
        prop_assert!(flags.contains(flags));
    }

    /// contains の相互排他: 異なるフラグは互いを含まない
    #[test]
    fn event_flags_contains_mutually_exclusive(f in 0..EVENT_FLAGS.len(), g in 0..EVENT_FLAGS.len()) {
        let flags = EVENT_FLAGS[f];
        let other = EVENT_FLAGS[g];
        prop_assert_eq!(flags.contains(other), f == g);
    }
}
