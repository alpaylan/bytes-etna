//! ETNA witness tests for the `bytes` crate.
//!
//! Each `witness_<name>_case_<tag>` calls one property function with
//! the exact canonical input embedded in the `etna` replay path
//! (`src/bin/etna.rs`). On base, every witness passes. On its paired
//! variant branch (patched with the bug), the corresponding witness
//! fails.

#![cfg(feature = "etna")]

use bytes::etna::{
    property_chain_remaining_saturating, property_get_int_sign_extension,
    property_get_int_zero_nbytes, property_partialord_bytes_reversed,
    property_slice_ref_empty, PropertyResult,
};

fn assert_pass(r: PropertyResult) {
    match r {
        PropertyResult::Pass | PropertyResult::Discard => {}
        PropertyResult::Fail(m) => panic!("property failed: {}", m),
    }
}

/// Triggers `sign_extend_zero_nbytes_acd1e0f_1`. `get_int(0)` must
/// return 0 without panicking.
#[test]
fn witness_get_int_zero_nbytes_case_eight_zero_bytes() {
    assert_pass(property_get_int_zero_nbytes(vec![0u8; 8]));
}

/// Triggers `get_int_sign_extension_79fb853_1`. A 4-byte `[0xff; 4]`
/// read as a signed int must come back as `-1`, not `0x00ff_ffff_ff`.
#[test]
fn witness_get_int_sign_extension_case_four_ff_bytes() {
    // n_byte_select = 3 → n = 4
    assert_pass(property_get_int_sign_extension(3));
}

/// Triggers `chain_remaining_saturating_2428c15_1`. Two operands whose
/// `remaining()` sum overflows `usize` must produce a `Chain::remaining()`
/// value >= max of the two (saturating semantics).
#[test]
fn witness_chain_remaining_saturating_case_max_plus_one() {
    assert_pass(property_chain_remaining_saturating(usize::MAX, 1));
}

/// Triggers `partialord_bytes_reversed_939a5ed_1`. Comparing a `[u8]`
/// slice against a strictly-greater `Bytes` must return `Less`, not
/// the reversed `Greater`.
#[test]
fn witness_partialord_bytes_reversed_case_lhs_less() {
    assert_pass(property_partialord_bytes_reversed(vec![1, 2, 3], vec![1, 2, 4]));
}

/// Triggers `slice_ref_empty_f330ef6_1`. `slice_ref(&[])` must not
/// panic and must return an empty `Bytes` for any source buffer.
#[test]
fn witness_slice_ref_empty_case_nonempty_source() {
    assert_pass(property_slice_ref_empty(vec![0, 1, 2, 3, 4, 5]));
}
