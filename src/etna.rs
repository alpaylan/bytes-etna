//! ETNA framework-neutral property functions for the `bytes` crate.
//!
//! Each `property_<name>` is a pure function taking concrete, owned inputs
//! and returning `PropertyResult`. Framework adapters in `src/bin/etna.rs`
//! and witness tests in `tests/etna_witnesses.rs` all call these functions
//! directly.

#![allow(missing_docs)]

use std::cmp;
use std::format;
use std::panic;
use std::string::String;
use std::vec;
use std::vec::Vec;

use crate::{Buf, Bytes};

#[derive(Debug)]
pub enum PropertyResult {
    Pass,
    Fail(String),
    Discard,
}

// ---------------------------------------------------------------------------
// sign_extend_zero_nbytes_acd1e0f_1
// ---------------------------------------------------------------------------

/// Invariant: `Buf::get_int(0)` and `Buf::get_int_le(0)` must return `0`
/// without panicking, regardless of buffer contents.
///
/// Bug this catches:
/// - `sign_extend_zero_nbytes_acd1e0f_1`: pre-fix `sign_extend(val, 0)`
///   computed `(val << 64) >> 64`, which overflows the left shift width
///   and panics in debug / yields wrong bits in release. Property calls
///   `get_int(0)` on an arbitrary buffer and checks the return is `0`.
pub fn property_get_int_zero_nbytes(head_bytes: Vec<u8>) -> PropertyResult {
    let mut trimmed = head_bytes;
    if trimmed.len() > 8 {
        trimmed.truncate(8);
    }
    while trimmed.len() < 8 {
        trimmed.push(0);
    }
    let buf_be = &trimmed[..];
    let mut be_slice: &[u8] = buf_be;
    let res_be = panic::catch_unwind(panic::AssertUnwindSafe(|| be_slice.get_int(0)));
    let mut le_slice: &[u8] = buf_be;
    let res_le = panic::catch_unwind(panic::AssertUnwindSafe(|| le_slice.get_int_le(0)));
    match (res_be, res_le) {
        (Ok(0), Ok(0)) => PropertyResult::Pass,
        (Ok(v), _) if v != 0 => PropertyResult::Fail(format!(
            "get_int(0) returned {} for buffer {:?}, expected 0",
            v, buf_be
        )),
        (_, Ok(v)) if v != 0 => PropertyResult::Fail(format!(
            "get_int_le(0) returned {} for buffer {:?}, expected 0",
            v, buf_be
        )),
        (Err(_), _) => PropertyResult::Fail(format!(
            "get_int(0) panicked for buffer {:?}",
            buf_be
        )),
        (_, Err(_)) => PropertyResult::Fail(format!(
            "get_int_le(0) panicked for buffer {:?}",
            buf_be
        )),
        (Ok(_), Ok(_)) => PropertyResult::Pass,
    }
}

// ---------------------------------------------------------------------------
// get_int_sign_extension_79fb853_1
// ---------------------------------------------------------------------------

/// Compute the expected `i64` value of a big-endian `n`-byte two's-complement
/// integer. `bytes.len()` must equal `n` and `1 <= n <= 8`.
fn expected_get_int_be(bytes: &[u8]) -> i64 {
    let n = bytes.len();
    debug_assert!((1..=8).contains(&n));
    let mut acc: u64 = 0;
    for &b in bytes {
        acc = (acc << 8) | (b as u64);
    }
    // Sign-extend from `n` bytes to 64 bits.
    let shift = 64 - (n * 8) as u32;
    ((acc << shift) as i64) >> shift
}

/// Invariant: for any `1 <= n <= 8`, reading an `n`-byte signed integer via
/// `Buf::get_int` must agree with the two's-complement big-endian
/// interpretation of those `n` bytes (sign-extended into `i64`). For `n == 8`
/// this is a plain transmute; for smaller `n` the high bit of the leading
/// byte determines the sign.
///
/// Bug this catches:
/// - `get_int_sign_extension_79fb853_1`: pre-fix `get_int` delegated to the
///   big-endian macro with a smaller-than-8-byte slice, zero-extending
///   the high bits and returning a positive number for any buffer whose
///   leading byte has the high bit set. Property feeds a varied buffer
///   of length 1..=8 and varied byte values, and verifies `get_int(n)`
///   matches `expected_get_int_be(buf)` for every input.
pub fn property_get_int_sign_extension(buf: Vec<u8>) -> PropertyResult {
    // Normalize buffer to length 1..=8 — this is the bug's reachable domain.
    let mut bytes = buf;
    if bytes.is_empty() {
        bytes.push(0);
    }
    if bytes.len() > 8 {
        bytes.truncate(8);
    }
    let n = bytes.len();
    let expected = expected_get_int_be(&bytes);
    let mut s: &[u8] = &bytes[..];
    let got = match panic::catch_unwind(panic::AssertUnwindSafe(|| s.get_int(n))) {
        Ok(v) => v,
        Err(_) => {
            return PropertyResult::Fail(format!(
                "get_int({}) panicked for buffer {:?}",
                n, bytes
            ));
        }
    };
    if got == expected {
        PropertyResult::Pass
    } else {
        PropertyResult::Fail(format!(
            "get_int({}) on {:?} returned {}, expected {}",
            n, bytes, got, expected
        ))
    }
}

// ---------------------------------------------------------------------------
// chain_remaining_saturating_2428c15_1
// ---------------------------------------------------------------------------

/// Invariant: `Chain::remaining` must never return a value smaller than
/// either operand's `remaining()` — the combined length cannot be less
/// than the length of each half. Saturating semantics (cap at `usize::MAX`)
/// satisfy this; a naive `a + b` wraps on overflow and can return `0`.
///
/// Bug this catches:
/// - `chain_remaining_saturating_2428c15_1`: pre-fix the sum wrapped
///   silently; post-fix it uses `saturating_add`. Property drives a
///   pair of fake `Buf` impls advertising configurable `remaining` and
///   verifies the chained value is `>= max(a, b)`.
pub fn property_chain_remaining_saturating(a_rem: usize, b_rem: usize) -> PropertyResult {
    let a = FakeBuf::new(a_rem);
    let b = FakeBuf::new(b_rem);
    let chain = a.chain(b);
    let got = match panic::catch_unwind(panic::AssertUnwindSafe(|| chain.remaining())) {
        Ok(v) => v,
        Err(_) => {
            return PropertyResult::Fail(format!(
                "chain.remaining() panicked with a_rem={}, b_rem={}",
                a_rem, b_rem
            ));
        }
    };
    let lower = cmp::max(a_rem, b_rem);
    if got >= lower {
        PropertyResult::Pass
    } else {
        PropertyResult::Fail(format!(
            "chain.remaining() = {}, below max({}, {}) = {}",
            got, a_rem, b_rem, lower
        ))
    }
}

/// A minimal `Buf` implementation whose `remaining()` can be set to an
/// arbitrary value without requiring that much backing memory. Used only
/// to exercise `Chain::remaining`'s arithmetic path — `chunk()` returns an
/// empty slice and `advance(0)` is a no-op; callers that try to actually
/// read bytes will encounter an empty chunk, which is fine for this
/// property.
struct FakeBuf {
    remaining: usize,
}

impl FakeBuf {
    fn new(remaining: usize) -> Self {
        FakeBuf { remaining }
    }
}

impl Buf for FakeBuf {
    fn remaining(&self) -> usize {
        self.remaining
    }
    fn chunk(&self) -> &[u8] {
        &[]
    }
    fn advance(&mut self, cnt: usize) {
        self.remaining = self.remaining.saturating_sub(cnt);
    }
}

// ---------------------------------------------------------------------------
// partialord_bytes_reversed_939a5ed_1
// ---------------------------------------------------------------------------

/// Invariant: `<[u8] as PartialOrd<Bytes>>::partial_cmp(lhs, rhs)` must
/// agree with `lhs.partial_cmp(rhs.as_ref())` — i.e. comparing `[u8]`
/// against `Bytes` yields the same ordering as comparing two `[u8]`s.
///
/// Bug this catches:
/// - `partialord_bytes_reversed_939a5ed_1`: pre-fix, the reverse impls
///   called `other.partial_cmp(self)` which resolves to
///   `Bytes::partial_cmp(other, self)` with operands reversed, so the
///   ordering came back flipped. Property compares matched and mismatched
///   prefixes and verifies the returned ordering is self-consistent.
pub fn property_partialord_bytes_reversed(lhs: Vec<u8>, rhs: Vec<u8>) -> PropertyResult {
    let bytes_rhs = Bytes::copy_from_slice(&rhs[..]);
    let got: Option<cmp::Ordering> = <[u8] as PartialOrd<Bytes>>::partial_cmp(&lhs[..], &bytes_rhs);
    let expected: Option<cmp::Ordering> = lhs[..].partial_cmp(&rhs[..]);
    if got == expected {
        PropertyResult::Pass
    } else {
        PropertyResult::Fail(format!(
            "[u8]::partial_cmp(&Bytes) = {:?}, expected {:?} (lhs={:?}, rhs={:?})",
            got, expected, lhs, rhs
        ))
    }
}

// ---------------------------------------------------------------------------
// slice_ref_empty_f330ef6_1
// ---------------------------------------------------------------------------

/// Invariant: `Bytes::slice_ref(&[])` must not panic and must return an
/// empty `Bytes`, regardless of the pointer provenance of the input
/// slice. The empty slice is a trivial subslice of any `Bytes`.
///
/// Bug this catches:
/// - `slice_ref_empty_f330ef6_1`: pre-fix `slice_ref` fell through to
///   the pointer-range asserts, which rejected empty subslices (whose
///   pointer is dangling / outside the backing allocation). Property
///   picks a random `Bytes` and calls `slice_ref` with the dangling
///   empty slice `&[]`.
pub fn property_slice_ref_empty(payload: Vec<u8>) -> PropertyResult {
    let haystack = Bytes::from(payload.clone());
    // `&[]` is deliberately dangling w.r.t. `haystack`'s allocation: its
    // pointer is 0x1 (empty-slice sentinel), far outside the Bytes'
    // owned region. This is the exact case the fix explicitly allows.
    let empty: &[u8] = &[];
    let got = panic::catch_unwind(panic::AssertUnwindSafe(|| haystack.slice_ref(empty)));
    match got {
        Ok(b) if b.is_empty() => PropertyResult::Pass,
        Ok(b) => PropertyResult::Fail(format!(
            "slice_ref(&[]) on Bytes of len {} returned non-empty Bytes of len {}",
            payload.len(),
            b.len()
        )),
        Err(_) => PropertyResult::Fail(format!(
            "slice_ref(&[]) panicked on Bytes of len {}",
            payload.len()
        )),
    }
}
