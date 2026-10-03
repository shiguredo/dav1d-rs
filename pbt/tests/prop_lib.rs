//! `src/lib.rs` のビットフラグ型 (InloopFilterType / EventFlags) の代数のプロパティテスト

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

// PBT の 1 ケースあたりの実行数
//
// 対象の値域は 5 種類 / 2 種類と小さく、全ての値が一様に選ばれるため、
// 256 件で全組み合わせを十分な確率で網羅できる。
const CASES: usize = 256;

/// InloopFilterType の定数を一様に選ぶ
fn sample_inloop_filter(ctx: &mut noprop::TestCaseContext) -> InloopFilterType {
    INLOOP_FILTERS[noprop::sample_usize_in(ctx, 0..INLOOP_FILTERS.len())]
}

/// EventFlags の定数を一様に選ぶ
fn sample_event_flags(ctx: &mut noprop::TestCaseContext) -> EventFlags {
    EVENT_FLAGS[noprop::sample_usize_in(ctx, 0..EVENT_FLAGS.len())]
}

/// BitOr の結合則: (a | b) | c == a | (b | c)
#[test]
fn inloop_filter_bitor_associative() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("SHIGUREDO_DAV1D_PBT_SEED")?;
    let mut runner = noprop::Runner::new(seed);

    runner.run(CASES, |ctx| {
        let a = sample_inloop_filter(ctx);
        let b = sample_inloop_filter(ctx);
        let c = sample_inloop_filter(ctx);
        assert_eq!((a | b) | c, a | (b | c), "BitOr の結合則が成立しない");
        Ok(())
    })?;
    Ok(())
}

/// BitOr の交換則: a | b == b | a
#[test]
fn inloop_filter_bitor_commutative() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("SHIGUREDO_DAV1D_PBT_SEED")?;
    let mut runner = noprop::Runner::new(seed);

    runner.run(CASES, |ctx| {
        let a = sample_inloop_filter(ctx);
        let b = sample_inloop_filter(ctx);
        assert_eq!(a | b, b | a, "BitOr の交換則が成立しない");
        Ok(())
    })?;
    Ok(())
}

/// NONE は BitOr の単位元: a | NONE == a
#[test]
fn inloop_filter_bitor_identity() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("SHIGUREDO_DAV1D_PBT_SEED")?;
    let mut runner = noprop::Runner::new(seed);

    runner.run(CASES, |ctx| {
        let a = sample_inloop_filter(ctx);
        assert_eq!(a | InloopFilterType::NONE, a, "NONE が単位元になっていない");
        Ok(())
    })?;
    Ok(())
}

/// ALL は BitOr の吸収元: a | ALL == ALL
#[test]
fn inloop_filter_bitor_absorbing() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("SHIGUREDO_DAV1D_PBT_SEED")?;
    let mut runner = noprop::Runner::new(seed);

    runner.run(CASES, |ctx| {
        let a = sample_inloop_filter(ctx);
        assert_eq!(
            a | InloopFilterType::ALL,
            InloopFilterType::ALL,
            "ALL が吸収元になっていない"
        );
        Ok(())
    })?;
    Ok(())
}

/// contains の反射律: フラグは自分自身を含む
#[test]
fn event_flags_contains_reflexive() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("SHIGUREDO_DAV1D_PBT_SEED")?;
    let mut runner = noprop::Runner::new(seed);

    runner.run(CASES, |ctx| {
        let flags = sample_event_flags(ctx);
        assert!(flags.contains(flags), "自分自身を含んでいない");
        Ok(())
    })?;
    Ok(())
}

/// contains の相互排他: 異なるフラグは互いを含まない
#[test]
fn event_flags_contains_mutually_exclusive() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("SHIGUREDO_DAV1D_PBT_SEED")?;
    let mut runner = noprop::Runner::new(seed);
    // 同じフラグの組と異なるフラグの組の両方を検証したことを確認する。
    // 片方しか現れないと contains の対応関係を検証したことにならないため。
    let same = std::cell::Cell::new(0usize);
    let different = std::cell::Cell::new(0usize);

    runner.run(CASES, |ctx| {
        let f = noprop::sample_usize_in(ctx, 0..EVENT_FLAGS.len());
        let g = noprop::sample_usize_in(ctx, 0..EVENT_FLAGS.len());
        let flags = EVENT_FLAGS[f];
        let other = EVENT_FLAGS[g];
        assert_eq!(
            flags.contains(other),
            f == g,
            "contains の結果がフラグの一致状況と対応していない"
        );
        if f == g {
            same.set(same.get() + 1);
        } else {
            different.set(different.get() + 1);
        }
        Ok(())
    })?;

    assert!(
        same.get() > 0,
        "同じフラグの組のケースが 1 件も無い\n{runner}"
    );
    assert!(
        different.get() > 0,
        "異なるフラグの組のケースが 1 件も無い\n{runner}"
    );
    Ok(())
}
