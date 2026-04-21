// ETNA workload runner for bytes.
//
// Usage: cargo run --release --bin etna -- <tool> <property>
//   tool:     etna | proptest | quickcheck | crabcheck | hegel
//   property: GetIntZeroNbytes | GetIntSignExtension | ChainRemainingSaturating
//             | PartialOrdBytesReversed | SliceRefEmpty | All
//
// Every invocation prints exactly one JSON line to stdout and exits 0
// (except argv parsing, which exits 2). Etna reads status from JSON —
// not the exit code — so framework-level failures (counterexamples,
// timeouts) still produce exit 0.

use bytes::etna::{
    property_chain_remaining_saturating, property_get_int_sign_extension,
    property_get_int_zero_nbytes, property_partialord_bytes_reversed,
    property_slice_ref_empty, PropertyResult,
};

use crabcheck::quickcheck as crabcheck_qc;
use crabcheck::quickcheck::Arbitrary as CcArbitrary;
use hegel::{generators as hgen, HealthCheck, Hegel, Settings as HegelSettings, TestCase};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestCaseError, TestError};
use quickcheck_etna::{Arbitrary as QcArbitrary, Gen, QuickCheck, ResultStatus, TestResult};
use rand_etna::Rng;

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
struct Metrics {
    inputs: u64,
    elapsed_us: u128,
}

impl Metrics {
    fn combine(self, other: Metrics) -> Metrics {
        Metrics {
            inputs: self.inputs + other.inputs,
            elapsed_us: self.elapsed_us + other.elapsed_us,
        }
    }
}

type Outcome = (Result<(), String>, Metrics);

fn to_err(r: PropertyResult) -> Result<(), String> {
    match r {
        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
        PropertyResult::Fail(m) => Err(m),
    }
}

const ALL_PROPERTIES: &[&str] = &[
    "GetIntZeroNbytes",
    "GetIntSignExtension",
    "ChainRemainingSaturating",
    "PartialOrdBytesReversed",
    "SliceRefEmpty",
];

fn cases_budget() -> u64 {
    std::env::var("ETNA_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200)
}

fn run_all<F: FnMut(&str) -> Outcome>(mut f: F) -> Outcome {
    let mut total = Metrics::default();
    for p in ALL_PROPERTIES {
        let (r, m) = f(p);
        total = total.combine(m);
        if let Err(e) = r {
            return (Err(e), total);
        }
    }
    (Ok(()), total)
}

// ============================================================================
// Input wrappers
// ============================================================================

#[derive(Clone)]
struct GetIntZeroInput {
    buf: Vec<u8>,
}

impl fmt::Debug for GetIntZeroInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "buf={:?}", self.buf)
    }
}

impl fmt::Display for GetIntZeroInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone)]
struct GetIntSignExtensionInput {
    n_byte_select: u8,
}

impl fmt::Debug for GetIntSignExtensionInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "n_byte_select={}", self.n_byte_select)
    }
}

impl fmt::Display for GetIntSignExtensionInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone)]
struct ChainRemainingInput {
    a_rem: usize,
    b_rem: usize,
}

impl fmt::Debug for ChainRemainingInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a_rem={} b_rem={}", self.a_rem, self.b_rem)
    }
}

impl fmt::Display for ChainRemainingInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone)]
struct PartialOrdInput {
    lhs: Vec<u8>,
    rhs: Vec<u8>,
}

impl fmt::Debug for PartialOrdInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lhs={:?} rhs={:?}", self.lhs, self.rhs)
    }
}

impl fmt::Display for PartialOrdInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone)]
struct SliceRefEmptyInput {
    payload: Vec<u8>,
}

impl fmt::Debug for SliceRefEmptyInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "payload_len={}", self.payload.len())
    }
}

impl fmt::Display for SliceRefEmptyInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

// ============================================================================
// Canonical witness inputs — keep in sync with tests/etna_witnesses.rs.
// ============================================================================

fn canonical_get_int_zero() -> GetIntZeroInput {
    GetIntZeroInput {
        buf: vec![0u8; 8],
    }
}

fn canonical_get_int_sign_extension() -> GetIntSignExtensionInput {
    GetIntSignExtensionInput { n_byte_select: 3 }
}

fn canonical_chain_remaining() -> ChainRemainingInput {
    ChainRemainingInput {
        a_rem: usize::MAX,
        b_rem: 1,
    }
}

fn canonical_partialord() -> PartialOrdInput {
    PartialOrdInput {
        lhs: vec![1, 2, 3],
        rhs: vec![1, 2, 4],
    }
}

fn canonical_slice_ref_empty() -> SliceRefEmptyInput {
    SliceRefEmptyInput {
        payload: vec![0, 1, 2, 3, 4, 5],
    }
}

fn check_get_int_zero_nbytes() -> Result<(), String> {
    to_err(property_get_int_zero_nbytes(canonical_get_int_zero().buf))
}

fn check_get_int_sign_extension() -> Result<(), String> {
    to_err(property_get_int_sign_extension(
        canonical_get_int_sign_extension().n_byte_select,
    ))
}

fn check_chain_remaining_saturating() -> Result<(), String> {
    let v = canonical_chain_remaining();
    to_err(property_chain_remaining_saturating(v.a_rem, v.b_rem))
}

fn check_partialord_bytes_reversed() -> Result<(), String> {
    let v = canonical_partialord();
    to_err(property_partialord_bytes_reversed(v.lhs, v.rhs))
}

fn check_slice_ref_empty() -> Result<(), String> {
    to_err(property_slice_ref_empty(canonical_slice_ref_empty().payload))
}

// ============================================================================
// etna tool — deterministic canonical replay.
// ============================================================================

fn run_etna_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_etna_property);
    }
    let t0 = Instant::now();
    let result = match property {
        "GetIntZeroNbytes" => check_get_int_zero_nbytes(),
        "GetIntSignExtension" => check_get_int_sign_extension(),
        "ChainRemainingSaturating" => check_chain_remaining_saturating(),
        "PartialOrdBytesReversed" => check_partialord_bytes_reversed(),
        "SliceRefEmpty" => check_slice_ref_empty(),
        _ => {
            return (
                Err(format!("Unknown property for etna: {property}")),
                Metrics::default(),
            );
        }
    };
    (
        result,
        Metrics {
            inputs: 1,
            elapsed_us: t0.elapsed().as_micros(),
        },
    )
}

// ============================================================================
// quickcheck Arbitrary
// ============================================================================

impl QcArbitrary for GetIntZeroInput {
    fn arbitrary(g: &mut Gen) -> Self {
        let n: usize = (<usize as QcArbitrary>::arbitrary(g) % 16) + 1;
        let buf: Vec<u8> = (0..n).map(|_| <u8 as QcArbitrary>::arbitrary(g)).collect();
        GetIntZeroInput { buf }
    }
}

impl QcArbitrary for GetIntSignExtensionInput {
    fn arbitrary(g: &mut Gen) -> Self {
        GetIntSignExtensionInput {
            n_byte_select: <u8 as QcArbitrary>::arbitrary(g),
        }
    }
}

impl QcArbitrary for ChainRemainingInput {
    fn arbitrary(g: &mut Gen) -> Self {
        ChainRemainingInput {
            a_rem: <usize as QcArbitrary>::arbitrary(g),
            b_rem: <usize as QcArbitrary>::arbitrary(g),
        }
    }
}

impl QcArbitrary for PartialOrdInput {
    fn arbitrary(g: &mut Gen) -> Self {
        let n: usize = (<usize as QcArbitrary>::arbitrary(g) % 16) + 1;
        let mut lhs: Vec<u8> = (0..n).map(|_| <u8 as QcArbitrary>::arbitrary(g)).collect();
        let mut rhs = lhs.clone();
        // Perturb rhs so it's sometimes equal, sometimes different.
        let flip: u8 = <u8 as QcArbitrary>::arbitrary(g);
        match flip % 4 {
            0 => {}
            1 => {
                if !rhs.is_empty() {
                    let idx = (<usize as QcArbitrary>::arbitrary(g)) % rhs.len();
                    rhs[idx] = rhs[idx].wrapping_add(1);
                }
            }
            2 => rhs.push(<u8 as QcArbitrary>::arbitrary(g)),
            _ => {
                lhs.push(<u8 as QcArbitrary>::arbitrary(g));
            }
        }
        PartialOrdInput { lhs, rhs }
    }
}

impl QcArbitrary for SliceRefEmptyInput {
    fn arbitrary(g: &mut Gen) -> Self {
        let n: usize = (<usize as QcArbitrary>::arbitrary(g) % 32) + 1;
        SliceRefEmptyInput {
            payload: (0..n).map(|_| <u8 as QcArbitrary>::arbitrary(g)).collect(),
        }
    }
}

// ============================================================================
// crabcheck Arbitrary
// ============================================================================

impl<R: Rng> CcArbitrary<R> for GetIntZeroInput {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let n: usize = ((rng.random::<u32>() as usize) % 16) + 1;
        GetIntZeroInput {
            buf: (0..n).map(|_| rng.random::<u8>()).collect(),
        }
    }
}

impl<R: Rng> CcArbitrary<R> for GetIntSignExtensionInput {
    fn generate(rng: &mut R, _n: usize) -> Self {
        GetIntSignExtensionInput {
            n_byte_select: rng.random::<u8>(),
        }
    }
}

impl<R: Rng> CcArbitrary<R> for ChainRemainingInput {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let a_hi = rng.random::<u32>() as u64;
        let a_lo = rng.random::<u32>() as u64;
        let b_hi = rng.random::<u32>() as u64;
        let b_lo = rng.random::<u32>() as u64;
        ChainRemainingInput {
            a_rem: ((a_hi << 32) | a_lo) as usize,
            b_rem: ((b_hi << 32) | b_lo) as usize,
        }
    }
}

impl<R: Rng> CcArbitrary<R> for PartialOrdInput {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let n: usize = ((rng.random::<u32>() as usize) % 16) + 1;
        let mut lhs: Vec<u8> = (0..n).map(|_| rng.random::<u8>()).collect();
        let mut rhs = lhs.clone();
        match rng.random::<u32>() % 4 {
            0 => {}
            1 => {
                if !rhs.is_empty() {
                    let idx = (rng.random::<u32>() as usize) % rhs.len();
                    rhs[idx] = rhs[idx].wrapping_add(1);
                }
            }
            2 => rhs.push(rng.random::<u8>()),
            _ => lhs.push(rng.random::<u8>()),
        }
        PartialOrdInput { lhs, rhs }
    }
}

impl<R: Rng> CcArbitrary<R> for SliceRefEmptyInput {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let n: usize = ((rng.random::<u32>() as usize) % 32) + 1;
        SliceRefEmptyInput {
            payload: (0..n).map(|_| rng.random::<u8>()).collect(),
        }
    }
}

// ============================================================================
// proptest strategies
// ============================================================================

fn get_int_zero_strategy() -> BoxedStrategy<GetIntZeroInput> {
    proptest::collection::vec(any::<u8>(), 1..16usize)
        .prop_map(|buf| GetIntZeroInput { buf })
        .boxed()
}

fn get_int_sign_extension_strategy() -> BoxedStrategy<GetIntSignExtensionInput> {
    any::<u8>()
        .prop_map(|n_byte_select| GetIntSignExtensionInput { n_byte_select })
        .boxed()
}

fn chain_remaining_strategy() -> BoxedStrategy<ChainRemainingInput> {
    (any::<usize>(), any::<usize>())
        .prop_map(|(a_rem, b_rem)| ChainRemainingInput { a_rem, b_rem })
        .boxed()
}

fn partialord_strategy() -> BoxedStrategy<PartialOrdInput> {
    (
        proptest::collection::vec(any::<u8>(), 1..16usize),
        proptest::collection::vec(any::<u8>(), 1..16usize),
    )
        .prop_map(|(lhs, rhs)| PartialOrdInput { lhs, rhs })
        .boxed()
}

fn slice_ref_empty_strategy() -> BoxedStrategy<SliceRefEmptyInput> {
    proptest::collection::vec(any::<u8>(), 1..32usize)
        .prop_map(|payload| SliceRefEmptyInput { payload })
        .boxed()
}

// ============================================================================
// proptest adapter
// ============================================================================

fn run_proptest_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_proptest_property);
    }
    let counter = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();
    let cfg = proptest::test_runner::Config {
        cases: cases_budget().min(u32::MAX as u64) as u32,
        max_shrink_iters: 32,
        failure_persistence: None,
        ..ProptestConfig::default()
    };
    let mut runner = proptest::test_runner::TestRunner::new(cfg);
    let c = counter.clone();
    let result: Result<(), String> = match property {
        "GetIntZeroNbytes" => runner
            .run(&get_int_zero_strategy(), move |v| {
                c.fetch_add(1, Ordering::Relaxed);
                let cex = format!("({:?})", v);
                let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_get_int_zero_nbytes(v.buf.clone())
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(cex)),
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        "GetIntSignExtension" => runner
            .run(&get_int_sign_extension_strategy(), move |v| {
                c.fetch_add(1, Ordering::Relaxed);
                let cex = format!("({:?})", v);
                let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_get_int_sign_extension(v.n_byte_select)
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(cex)),
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        "ChainRemainingSaturating" => runner
            .run(&chain_remaining_strategy(), move |v| {
                c.fetch_add(1, Ordering::Relaxed);
                let cex = format!("({:?})", v);
                let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_chain_remaining_saturating(v.a_rem, v.b_rem)
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(cex)),
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        "PartialOrdBytesReversed" => runner
            .run(&partialord_strategy(), move |v| {
                c.fetch_add(1, Ordering::Relaxed);
                let cex = format!("({:?})", v);
                let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_partialord_bytes_reversed(v.lhs.clone(), v.rhs.clone())
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(cex)),
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        "SliceRefEmpty" => runner
            .run(&slice_ref_empty_strategy(), move |v| {
                c.fetch_add(1, Ordering::Relaxed);
                let cex = format!("({:?})", v);
                let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_slice_ref_empty(v.payload.clone())
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(cex)),
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        _ => {
            return (
                Err(format!("Unknown property for proptest: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = counter.load(Ordering::Relaxed);
    (result, Metrics { inputs, elapsed_us })
}

// ============================================================================
// quickcheck adapter (fork with `etna` feature — fn-pointer API)
// ============================================================================

static QC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn qc_get_int_zero(v: GetIntZeroInput) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        property_get_int_zero_nbytes(v.buf)
    }));
    match out {
        Ok(PropertyResult::Pass) => TestResult::passed(),
        Ok(PropertyResult::Discard) => TestResult::discard(),
        Ok(PropertyResult::Fail(_)) | Err(_) => TestResult::failed(),
    }
}

fn qc_get_int_sign_extension(v: GetIntSignExtensionInput) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        property_get_int_sign_extension(v.n_byte_select)
    }));
    match out {
        Ok(PropertyResult::Pass) => TestResult::passed(),
        Ok(PropertyResult::Discard) => TestResult::discard(),
        Ok(PropertyResult::Fail(_)) | Err(_) => TestResult::failed(),
    }
}

fn qc_chain_remaining(v: ChainRemainingInput) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        property_chain_remaining_saturating(v.a_rem, v.b_rem)
    }));
    match out {
        Ok(PropertyResult::Pass) => TestResult::passed(),
        Ok(PropertyResult::Discard) => TestResult::discard(),
        Ok(PropertyResult::Fail(_)) | Err(_) => TestResult::failed(),
    }
}

fn qc_partialord(v: PartialOrdInput) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        property_partialord_bytes_reversed(v.lhs, v.rhs)
    }));
    match out {
        Ok(PropertyResult::Pass) => TestResult::passed(),
        Ok(PropertyResult::Discard) => TestResult::discard(),
        Ok(PropertyResult::Fail(_)) | Err(_) => TestResult::failed(),
    }
}

fn qc_slice_ref_empty(v: SliceRefEmptyInput) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        property_slice_ref_empty(v.payload)
    }));
    match out {
        Ok(PropertyResult::Pass) => TestResult::passed(),
        Ok(PropertyResult::Discard) => TestResult::discard(),
        Ok(PropertyResult::Fail(_)) | Err(_) => TestResult::failed(),
    }
}

fn run_quickcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_quickcheck_property);
    }
    QC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let budget = cases_budget();
    let mut qc = QuickCheck::new()
        .tests(budget)
        .max_tests(budget.saturating_mul(4))
        .max_time(Duration::from_secs(86_400));
    let result = match property {
        "GetIntZeroNbytes" => qc.quicktest(qc_get_int_zero as fn(GetIntZeroInput) -> TestResult),
        "GetIntSignExtension" => {
            qc.quicktest(qc_get_int_sign_extension as fn(GetIntSignExtensionInput) -> TestResult)
        }
        "ChainRemainingSaturating" => {
            qc.quicktest(qc_chain_remaining as fn(ChainRemainingInput) -> TestResult)
        }
        "PartialOrdBytesReversed" => qc.quicktest(qc_partialord as fn(PartialOrdInput) -> TestResult),
        "SliceRefEmpty" => {
            qc.quicktest(qc_slice_ref_empty as fn(SliceRefEmptyInput) -> TestResult)
        }
        _ => {
            return (
                Err(format!("Unknown property for quickcheck: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = QC_COUNTER.load(Ordering::Relaxed);
    let status = match result.status {
        ResultStatus::Finished => Ok(()),
        ResultStatus::Failed { arguments } => Err(format!("({})", arguments.join(" "))),
        ResultStatus::Aborted { err } => Err(format!("quickcheck aborted: {err:?}")),
        ResultStatus::TimedOut => Err("quickcheck timed out".to_string()),
        ResultStatus::GaveUp => Err(format!(
            "quickcheck gave up after {} tests",
            result.n_tests_passed
        )),
    };
    (status, Metrics { inputs, elapsed_us })
}

// ============================================================================
// crabcheck adapter (fn-pointer API)
// ============================================================================

static CC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cc_get_int_zero(v: GetIntZeroInput) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_get_int_zero_nbytes(v.buf) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_get_int_sign_extension(v: GetIntSignExtensionInput) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_get_int_sign_extension(v.n_byte_select) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_chain_remaining(v: ChainRemainingInput) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_chain_remaining_saturating(v.a_rem, v.b_rem) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_partialord(v: PartialOrdInput) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_partialord_bytes_reversed(v.lhs, v.rhs) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_slice_ref_empty(v: SliceRefEmptyInput) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_slice_ref_empty(v.payload) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn run_crabcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_crabcheck_property);
    }
    CC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let cc_config = crabcheck_qc::Config {
        tests: cases_budget(),
    };
    let result = match property {
        "GetIntZeroNbytes" => crabcheck_qc::quickcheck_with_config(cc_config, cc_get_int_zero),
        "GetIntSignExtension" => {
            crabcheck_qc::quickcheck_with_config(cc_config, cc_get_int_sign_extension)
        }
        "ChainRemainingSaturating" => {
            crabcheck_qc::quickcheck_with_config(cc_config, cc_chain_remaining)
        }
        "PartialOrdBytesReversed" => {
            crabcheck_qc::quickcheck_with_config(cc_config, cc_partialord)
        }
        "SliceRefEmpty" => {
            crabcheck_qc::quickcheck_with_config(cc_config, cc_slice_ref_empty)
        }
        _ => {
            return (
                Err(format!("Unknown property for crabcheck: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = CC_COUNTER.load(Ordering::Relaxed);
    let status = match result.status {
        crabcheck_qc::ResultStatus::Finished => Ok(()),
        crabcheck_qc::ResultStatus::Failed { arguments } => {
            Err(format!("({})", arguments.join(" ")))
        }
        crabcheck_qc::ResultStatus::TimedOut => Err("crabcheck timed out".to_string()),
        crabcheck_qc::ResultStatus::GaveUp => Err(format!(
            "crabcheck gave up: passed={}, discarded={}",
            result.passed, result.discarded
        )),
        crabcheck_qc::ResultStatus::Aborted { error } => {
            Err(format!("crabcheck aborted: {error}"))
        }
    };
    (status, Metrics { inputs, elapsed_us })
}

// ============================================================================
// hegel adapter (real hegeltest 0.3.7 — panic-on-cex API)
// ============================================================================

static HG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn hegel_settings() -> HegelSettings {
    HegelSettings::new()
        .test_cases(cases_budget())
        .suppress_health_check(HealthCheck::all())
}

fn hg_draw_usize(tc: &TestCase) -> usize {
    let hi = tc.draw(hgen::integers::<u32>().min_value(0).max_value(u32::MAX)) as u64;
    let lo = tc.draw(hgen::integers::<u32>().min_value(0).max_value(u32::MAX)) as u64;
    ((hi << 32) | lo) as usize
}

fn hg_draw_u8(tc: &TestCase) -> u8 {
    tc.draw(hgen::integers::<u32>().min_value(0).max_value(255)) as u8
}

fn hg_draw_bytes(tc: &TestCase, max_len: usize) -> Vec<u8> {
    let n = tc.draw(hgen::integers::<u32>().min_value(1).max_value(max_len as u32)) as usize;
    (0..n).map(|_| hg_draw_u8(tc)).collect()
}

fn run_hegel_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_hegel_property);
    }
    HG_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let settings = hegel_settings();
    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match property {
        "GetIntZeroNbytes" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let buf = hg_draw_bytes(&tc, 16);
                let cex = format!("(buf={:?})", buf);
                let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_get_int_zero_nbytes(buf.clone())
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{}", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "GetIntSignExtension" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let n_byte_select = hg_draw_u8(&tc);
                let cex = format!("(n_byte_select={})", n_byte_select);
                let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_get_int_sign_extension(n_byte_select)
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{}", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "ChainRemainingSaturating" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let a_rem = hg_draw_usize(&tc);
                let b_rem = hg_draw_usize(&tc);
                let cex = format!("(a_rem={} b_rem={})", a_rem, b_rem);
                let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_chain_remaining_saturating(a_rem, b_rem)
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{}", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "PartialOrdBytesReversed" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let lhs = hg_draw_bytes(&tc, 16);
                let rhs = hg_draw_bytes(&tc, 16);
                let cex = format!("(lhs={:?} rhs={:?})", lhs, rhs);
                let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_partialord_bytes_reversed(lhs.clone(), rhs.clone())
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{}", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "SliceRefEmpty" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let payload = hg_draw_bytes(&tc, 32);
                let cex = format!("(payload_len={})", payload.len());
                let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_slice_ref_empty(payload.clone())
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{}", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        _ => panic!("__unknown_property:{}", property),
    }));
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = HG_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match run_result {
        Ok(()) => Ok(()),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "hegel panicked with non-string payload".to_string()
            };
            if let Some(rest) = msg.strip_prefix("__unknown_property:") {
                return (
                    Err(format!("Unknown property for hegel: {rest}")),
                    Metrics::default(),
                );
            }
            Err(msg
                .strip_prefix("Property test failed: ")
                .unwrap_or(&msg)
                .to_string())
        }
    };
    (status, metrics)
}

// ============================================================================
// dispatch + main
// ============================================================================

fn run(tool: &str, property: &str) -> Outcome {
    match tool {
        "etna" => run_etna_property(property),
        "proptest" => run_proptest_property(property),
        "quickcheck" => run_quickcheck_property(property),
        "crabcheck" => run_crabcheck_property(property),
        "hegel" => run_hegel_property(property),
        _ => (Err(format!("Unknown tool: {tool}")), Metrics::default()),
    }
}

fn json_str(s: &str) -> String {
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

fn emit_json(
    tool: &str,
    property: &str,
    status: &str,
    metrics: Metrics,
    counterexample: Option<&str>,
    error: Option<&str>,
) {
    let cex = counterexample.map_or("null".to_string(), json_str);
    let err = error.map_or("null".to_string(), json_str);
    println!(
        "{{\"status\":{},\"tests\":{},\"discards\":0,\"time\":{},\"counterexample\":{},\"error\":{},\"tool\":{},\"property\":{}}}",
        json_str(status),
        metrics.inputs,
        json_str(&format!("{}us", metrics.elapsed_us)),
        cex,
        err,
        json_str(tool),
        json_str(property),
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <tool> <property>", args[0]);
        eprintln!("Tools: etna | proptest | quickcheck | crabcheck | hegel");
        eprintln!(
            "Properties: GetIntZeroNbytes | GetIntSignExtension | ChainRemainingSaturating | PartialOrdBytesReversed | SliceRefEmpty | All"
        );
        std::process::exit(2);
    }
    let (tool, property) = (args[1].as_str(), args[2].as_str());

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(tool, property)));
    std::panic::set_hook(previous_hook);

    let (result, metrics) = match caught {
        Ok(outcome) => outcome,
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "panic with non-string payload".to_string()
            };
            emit_json(tool, property, "aborted", Metrics::default(), None, Some(&msg));
            return;
        }
    };

    match result {
        Ok(()) => emit_json(tool, property, "passed", metrics, None, None),
        Err(e) => emit_json(tool, property, "failed", metrics, Some(&e), None),
    }
}
