//! Fault-localization integration tests for bytes.
//!
//! One `#[test]` per property in src/bin/etna-faultloc.rs's dispatch.

#![cfg(feature = "etna")]

use bytes::etna::{
    property_chain_remaining_saturating, property_get_int_sign_extension,
    property_get_int_zero_nbytes, property_partialord_bytes_reversed,
    property_slice_ref_empty, PropertyResult,
};
use crabcheck::quickcheck::{Arbitrary, Mutate};
use rand_etna::Rng;
use std::fmt;

#[derive(Clone)]
struct Bytes(Vec<u8>);
impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Buffer of length 1..=8 — the reachable domain of `Buf::get_int(nbytes)`.
#[derive(Clone)]
struct GetIntBuf(Vec<u8>);
impl fmt::Debug for GetIntBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone)]
struct ChainInput {
    a: usize,
    b: usize,
}
impl fmt::Debug for ChainInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a_rem={} b_rem={}", self.a, self.b)
    }
}

#[derive(Clone)]
struct TwoBytes {
    lhs: Vec<u8>,
    rhs: Vec<u8>,
}
impl fmt::Debug for TwoBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lhs={:?} rhs={:?}", self.lhs, self.rhs)
    }
}

fn gen_bytes<R: Rng>(rng: &mut R, max: usize) -> Vec<u8> {
    let len = rng.random_range(0usize..=max);
    (0..len)
        .map(|_| rng.random_range(0u16..=255) as u8)
        .collect()
}

impl<R: Rng> Arbitrary<R> for Bytes {
    fn generate(rng: &mut R, _n: usize) -> Self {
        Bytes(gen_bytes(rng, 16))
    }
}
impl<R: Rng> Arbitrary<R> for GetIntBuf {
    fn generate(rng: &mut R, _n: usize) -> Self {
        // Length 1..=8 with random byte values stresses positive/negative MSBs.
        let len = rng.random_range(1usize..=8);
        let buf: Vec<u8> = (0..len).map(|_| rng.random::<u8>()).collect();
        GetIntBuf(buf)
    }
}
impl<R: Rng> Arbitrary<R> for ChainInput {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let a_hi = rng.random::<u32>() as u64;
        let a_lo = rng.random::<u32>() as u64;
        let b_hi = rng.random::<u32>() as u64;
        let b_lo = rng.random::<u32>() as u64;
        ChainInput {
            a: ((a_hi << 32) | a_lo) as usize,
            b: ((b_hi << 32) | b_lo) as usize,
        }
    }
}
impl<R: Rng> Arbitrary<R> for TwoBytes {
    fn generate(rng: &mut R, _n: usize) -> Self {
        TwoBytes {
            lhs: gen_bytes(rng, 16),
            rhs: gen_bytes(rng, 16),
        }
    }
}

fn mutate_bytes<R: Rng>(rng: &mut R, v: &[u8], max: usize) -> Vec<u8> {
    let mut out = v.to_vec();
    match rng.random_range(0u8..3) {
        0 if !out.is_empty() => {
            let i = rng.random_range(0..out.len());
            let b = rng.random_range(0u32..8);
            out[i] ^= 1u8 << b;
        }
        1 if out.len() < max => out.push(rng.random_range(0u16..=255) as u8),
        _ if !out.is_empty() => {
            out.pop();
        }
        _ => {}
    }
    out
}

impl<R: Rng> Mutate<R> for Bytes {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        Bytes(mutate_bytes(rng, &self.0, 16))
    }
}
impl<R: Rng> Mutate<R> for GetIntBuf {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut out = self.0.clone();
        match rng.random_range(0u8..3) {
            0 if !out.is_empty() => {
                let i = rng.random_range(0..out.len());
                let b = rng.random_range(0u32..8);
                out[i] ^= 1u8 << b;
            }
            1 if out.len() < 8 => out.push(rng.random::<u8>()),
            _ if out.len() > 1 => {
                out.pop();
            }
            _ => {}
        }
        GetIntBuf(out)
    }
}
impl<R: Rng> Mutate<R> for ChainInput {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut out = self.clone();
        let bit = rng.random_range(0u32..(usize::BITS));
        if rng.random_bool(0.5) {
            out.a ^= 1usize << bit;
        } else {
            out.b ^= 1usize << bit;
        }
        out
    }
}
impl<R: Rng> Mutate<R> for TwoBytes {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut out = self.clone();
        if rng.random_bool(0.5) {
            out.lhs = mutate_bytes(rng, &out.lhs, 16);
        } else {
            out.rhs = mutate_bytes(rng, &out.rhs, 16);
        }
        out
    }
}

fn to_opt(r: PropertyResult) -> Option<bool> {
    match r {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn property_get_int_zero_nbytes_test(i: Bytes) -> Option<bool> {
    to_opt(property_get_int_zero_nbytes(i.0))
}

fn property_get_int_sign_extension_test(i: GetIntBuf) -> Option<bool> {
    to_opt(property_get_int_sign_extension(i.0))
}

fn property_chain_remaining_saturating_test(i: ChainInput) -> Option<bool> {
    to_opt(property_chain_remaining_saturating(i.a, i.b))
}

fn property_partialord_bytes_reversed_test(i: TwoBytes) -> Option<bool> {
    to_opt(property_partialord_bytes_reversed(i.lhs, i.rhs))
}

fn property_slice_ref_empty_test(i: Bytes) -> Option<bool> {
    to_opt(property_slice_ref_empty(i.0))
}

// Manual JSON emitter (we don't depend on serde_json in dev-deps).
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_f64(x: f64) -> String {
    if x.is_finite() {
        format!("{}", x)
    } else {
        "null".to_string()
    }
}

fn emit_locate_json(r: &crabcheck::profiling::LocateResult) {
    use crabcheck::quickcheck::ResultStatus;
    let status = match &r.run.status {
        ResultStatus::Failed { .. } => "Failed",
        ResultStatus::Finished => "Finished",
        ResultStatus::GaveUp => "GaveUp",
        ResultStatus::TimedOut => "TimedOut",
        ResultStatus::Aborted { .. } => "Aborted",
    };
    let top = if let Some(s) = r.top() {
        format!(
            "{{\"rank\":{},\"file\":{},\"function\":{},\"start_line\":{},\"end_line\":{},\"ochiai\":{},\"delta\":{},\"panic_overlap\":{},\"confidence\":{},\"confidence_rule\":{}}}",
            s.rank,
            json_escape(&s.region.file),
            json_escape(&s.region.function),
            s.region.start_line,
            s.region.end_line,
            json_f64(s.region.suspiciousness.ochiai as f64),
            json_f64(s.region.delta as f64),
            s.panic_overlap,
            json_escape(&format!("{}", s.confidence)),
            json_escape(s.confidence_rule),
        )
    } else {
        "null".to_string()
    };
    let top_5_items: Vec<String> = r
        .suspects
        .iter()
        .take(5)
        .map(|s| {
            format!(
                "{{\"rank\":{},\"file\":{},\"function\":{},\"start_line\":{},\"end_line\":{},\"confidence\":{},\"confidence_rule\":{},\"panic_overlap\":{}}}",
                s.rank,
                json_escape(&s.region.file),
                json_escape(&s.region.function),
                s.region.start_line,
                s.region.end_line,
                json_escape(&format!("{}", s.confidence)),
                json_escape(s.confidence_rule),
                s.panic_overlap,
            )
        })
        .collect();
    let top_5 = format!("[{}]", top_5_items.join(","));
    let diag_items: Vec<String> = r.diagnostics.iter().map(|d| json_escape(d.tag())).collect();
    let diags = format!("[{}]", diag_items.join(","));
    let out = format!(
        "{{\"status\":{},\"passed\":{},\"discarded\":{},\"n_panics\":{},\"n_suspects\":{},\"top\":{},\"top_5\":{},\"diagnostics\":{}}}",
        json_escape(status),
        r.run.passed,
        r.run.discarded,
        r.n_panics,
        r.suspects.len(),
        top,
        top_5,
        diags,
    );
    println!("@@LOCATE@@ {}", out);
}

#[test]
fn locate_get_int_zero_nbytes() {
    let report = crabcheck::quickcheck_with_locate!(property_get_int_zero_nbytes_test, "bytes");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_get_int_sign_extension() {
    let report = crabcheck::quickcheck_with_locate!(property_get_int_sign_extension_test, "bytes");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_chain_remaining_saturating() {
    let report =
        crabcheck::quickcheck_with_locate!(property_chain_remaining_saturating_test, "bytes");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_partialord_bytes_reversed() {
    let report =
        crabcheck::quickcheck_with_locate!(property_partialord_bytes_reversed_test, "bytes");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_slice_ref_empty() {
    let report = crabcheck::quickcheck_with_locate!(property_slice_ref_empty_test, "bytes");
    eprintln!("{report}");
    emit_locate_json(&report);
}
