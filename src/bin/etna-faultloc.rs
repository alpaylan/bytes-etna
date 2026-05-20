use std::fmt;

use bytes::etna::{
    property_chain_remaining_saturating, property_get_int_sign_extension,
    property_get_int_zero_nbytes, property_partialord_bytes_reversed,
    property_slice_ref_empty, PropertyResult,
};
use crabcheck::profiling::quickcheck;
use crabcheck::quickcheck::{Arbitrary, Mutate};
use rand_etna::Rng;

#[derive(Clone)]
struct Bytes(Vec<u8>);
impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

/// Buffer of length 1..=8 — the reachable domain of `Buf::get_int(nbytes)`.
#[derive(Clone)]
struct GetIntBuf(Vec<u8>);
impl fmt::Debug for GetIntBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

#[derive(Clone)]
struct ChainInput { a: usize, b: usize }
impl fmt::Debug for ChainInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a_rem={} b_rem={}", self.a, self.b)
    }
}

#[derive(Clone)]
struct TwoBytes { lhs: Vec<u8>, rhs: Vec<u8> }
impl fmt::Debug for TwoBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lhs={:?} rhs={:?}", self.lhs, self.rhs)
    }
}

fn gen_bytes<R: Rng>(rng: &mut R, max: usize) -> Vec<u8> {
    let len = rng.random_range(0usize..=max);
    (0..len).map(|_| rng.random_range(0u16..=255) as u8).collect()
}

impl<R: Rng> Arbitrary<R> for Bytes {
    fn generate(rng: &mut R, _n: usize) -> Self { Bytes(gen_bytes(rng, 16)) }
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
        // Mirror existing crabcheck adapter: assemble full 64-bit usize from
        // two u32 halves so values near usize::MAX (where the saturating-add
        // bug fires) are reachable. (`(a_hi << 32) | a_lo` pattern.)
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
        TwoBytes { lhs: gen_bytes(rng, 16), rhs: gen_bytes(rng, 16) }
    }
}

fn mutate_bytes<R: Rng>(rng: &mut R, v: &[u8], max: usize) -> Vec<u8> {
    let mut out = v.to_vec();
    match rng.random_range(0u8..3) {
        0 if !out.is_empty() => {
            let i = rng.random_range(0..out.len());
            let b = rng.random_range(0u32..8);
            out[i] ^= 1u8 << b;
        },
        1 if out.len() < max => out.push(rng.random_range(0u16..=255) as u8),
        _ if !out.is_empty() => { out.pop(); },
        _ => {},
    }
    out
}

impl<R: Rng> Mutate<R> for Bytes {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self { Bytes(mutate_bytes(rng, &self.0, 16)) }
}
impl<R: Rng> Mutate<R> for GetIntBuf {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut out = self.0.clone();
        match rng.random_range(0u8..3) {
            // Flip a bit — exercises positive/negative MSB transitions.
            0 if !out.is_empty() => {
                let i = rng.random_range(0..out.len());
                let b = rng.random_range(0u32..8);
                out[i] ^= 1u8 << b;
            }
            // Grow length up to 8.
            1 if out.len() < 8 => out.push(rng.random::<u8>()),
            // Shrink length but keep at least 1.
            _ if out.len() > 1 => { out.pop(); }
            _ => {}
        }
        GetIntBuf(out)
    }
}
impl<R: Rng> Mutate<R> for ChainInput {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut out = self.clone();
        let bit = rng.random_range(0u32..(usize::BITS));
        if rng.random_bool(0.5) { out.a ^= 1usize << bit; } else { out.b ^= 1usize << bit; }
        out
    }
}
impl<R: Rng> Mutate<R> for TwoBytes {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut out = self.clone();
        if rng.random_bool(0.5) { out.lhs = mutate_bytes(rng, &out.lhs, 16); }
        else { out.rhs = mutate_bytes(rng, &out.rhs, 16); }
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


fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 { return; }
    let tool = args[1].as_str();
    let property = args[2].as_str();
    let result = match (tool, property) {
        ("crabcheck", "GetIntZeroNbytes") => {
            quickcheck(|i: Bytes| to_opt(property_get_int_zero_nbytes(i.0)))
        },
        ("crabcheck", "GetIntSignExtension") => {
            quickcheck(|GetIntBuf(buf)| to_opt(property_get_int_sign_extension(buf)))
        },
        ("crabcheck", "ChainRemainingSaturating") => {
            quickcheck(|i: ChainInput| {
                to_opt(property_chain_remaining_saturating(i.a, i.b))
            })
        },
        ("crabcheck", "PartialordBytesReversed") => {
            quickcheck(|i: TwoBytes| {
                to_opt(property_partialord_bytes_reversed(i.lhs, i.rhs))
            })
        },
        ("crabcheck", "SliceRefEmpty") => {
            quickcheck(|i: Bytes| to_opt(property_slice_ref_empty(i.0)))
        },
        _ => panic!("Unknown: {tool} {property}"),
    };
    println!("Result: {:?}", result);
}
