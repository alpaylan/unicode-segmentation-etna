//! ETNA workload runner for unicode-segmentation.
//!
//! Usage: cargo run --release --features etna --bin etna -- <tool> <property>
//!   tool:     etna | proptest | quickcheck | crabcheck | hegel
//!   property: GraphemeNextBoundaryEmptyChunk | GraphemePrevBoundaryChunkStart
//!             | AsciiWordBoundIndicesMatch | All
//!
//! All four frameworks share the same generator shapes:
//!   * AnyText:   String of up to 16 arbitrary Unicode chars
//!   * AsciiText: String of up to 32 ASCII bytes (0..=127)
//!   * offset tag is a u8 clamped to a valid char boundary via pick_offset
//!   * is_extended is a plain bool
//!
//! Each invocation prints exactly one JSON object to stdout and exits 0
//! (argument-parsing errors exit 2).

use crabcheck::quickcheck as crabcheck_qc;
use crabcheck::quickcheck::Arbitrary as CcArbitrary;
use hegel::{generators as hgen, Hegel, Settings as HegelSettings};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestCaseError, TestError, TestRunner};
use quickcheck::{Arbitrary as QcArbitrary, Gen, QuickCheck, ResultStatus, TestResult};
use rand::Rng;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use unicode_segmentation::etna::{
    property_ascii_word_bound_indices_match, property_grapheme_next_boundary_empty_chunk_no_panic,
    property_grapheme_prev_boundary_chunk_start_no_panic, PropertyResult,
};

// ─────────────────── shared generator newtypes ───────────────────

#[derive(Clone)]
struct AnyText(String);

impl fmt::Debug for AnyText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for AnyText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

fn draw_char_qc(g: &mut Gen) -> char {
    loop {
        let cp = g.random_range(0u32..=0x10FFFFu32);
        if let Some(c) = char::from_u32(cp) {
            return c;
        }
    }
}

fn draw_char_cc<R: Rng>(rng: &mut R) -> char {
    loop {
        let cp = rng.random_range(0u32..=0x10FFFFu32);
        if let Some(c) = char::from_u32(cp) {
            return c;
        }
    }
}

impl QcArbitrary for AnyText {
    fn arbitrary(g: &mut Gen) -> Self {
        let len = g.random_range(0..16u32) as usize;
        let mut s = String::with_capacity(len * 4);
        for _ in 0..len {
            s.push(draw_char_qc(g));
        }
        AnyText(s)
    }
}

impl<R: Rng> CcArbitrary<R> for AnyText {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = rng.random_range(0..16u32) as usize;
        let mut s = String::with_capacity(len * 4);
        for _ in 0..len {
            s.push(draw_char_cc(rng));
        }
        AnyText(s)
    }
}

#[derive(Clone)]
struct AsciiText(String);

impl fmt::Debug for AsciiText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for AsciiText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl QcArbitrary for AsciiText {
    fn arbitrary(g: &mut Gen) -> Self {
        let len = g.random_range(0..32u32) as usize;
        let mut s = String::with_capacity(len);
        for _ in 0..len {
            s.push(g.random_range(0u8..=127u8) as char);
        }
        AsciiText(s)
    }
}

impl<R: Rng> CcArbitrary<R> for AsciiText {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = rng.random_range(0..32u32) as usize;
        let mut s = String::with_capacity(len);
        for _ in 0..len {
            s.push(rng.random_range(0u8..=127u8) as char);
        }
        AsciiText(s)
    }
}

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

/// Clamp an offset tag to a valid UTF-8 char boundary in `s`. Returns an
/// offset in `0..=s.len()` aligned to a char boundary. The downstream property
/// functions Discard on out-of-domain inputs.
fn pick_offset(s: &str, tag: u8) -> usize {
    if s.is_empty() {
        return 0;
    }
    let mut candidate = (tag as usize) % (s.len() + 1);
    while candidate > 0 && !s.is_char_boundary(candidate) {
        candidate -= 1;
    }
    candidate
}

const ALL_PROPERTIES: &[&str] = &[
    "GraphemeNextBoundaryEmptyChunk",
    "GraphemePrevBoundaryChunkStart",
    "AsciiWordBoundIndicesMatch",
];

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

// ------------------------------- etna ---------------------------------------

fn run_etna_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_etna_property);
    }
    let t0 = Instant::now();
    let result = match property {
        "GraphemeNextBoundaryEmptyChunk" => to_err(
            property_grapheme_next_boundary_empty_chunk_no_panic("abcdefgh".to_string(), 4, true),
        ),
        "GraphemePrevBoundaryChunkStart" => to_err(
            property_grapheme_prev_boundary_chunk_start_no_panic("abcd".to_string(), 2, true),
        ),
        "AsciiWordBoundIndicesMatch" => to_err(property_ascii_word_bound_indices_match(
            "can't".to_string(),
        )),
        _ => {
            return (
                Err(format!("Unknown property: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    (result, Metrics { inputs: 1, elapsed_us })
}

// ------------------------------ proptest ------------------------------------

fn any_text_strategy() -> BoxedStrategy<String> {
    proptest::collection::vec(any::<char>(), 0..16)
        .prop_map(|v| v.into_iter().collect::<String>())
        .boxed()
}

fn ascii_text_strategy() -> BoxedStrategy<String> {
    proptest::collection::vec(0u8..=127u8, 0..32)
        .prop_map(|v| v.into_iter().map(|b| b as char).collect::<String>())
        .boxed()
}

fn run_proptest_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_proptest_property);
    }
    let counter = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();
    let mut runner = TestRunner::new(ProptestConfig { cases: 40_000_000, ..ProptestConfig::default() });
    let c = counter.clone();
    let result: Result<(), String> = match property {
        "GraphemeNextBoundaryEmptyChunk" => {
            let strat = (any_text_strategy(), any::<u8>(), any::<bool>());
            runner
                .run(&strat, move |(s, tag, is_extended)| {
                    c.fetch_add(1, Ordering::Relaxed);
                    let off = pick_offset(&s, tag);
                    let s_cex = s.clone();
                    match property_grapheme_next_boundary_empty_chunk_no_panic(s, off, is_extended)
                    {
                        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                        PropertyResult::Fail(_) => Err(TestCaseError::fail(format!("({:?} {} {})", s_cex, off, is_extended))),
                    }
                })
                .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() })
        }
        "GraphemePrevBoundaryChunkStart" => {
            let strat = (any_text_strategy(), any::<u8>(), any::<bool>());
            runner
                .run(&strat, move |(s, tag, is_extended)| {
                    c.fetch_add(1, Ordering::Relaxed);
                    let off = pick_offset(&s, tag);
                    let s_cex = s.clone();
                    match property_grapheme_prev_boundary_chunk_start_no_panic(s, off, is_extended)
                    {
                        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                        PropertyResult::Fail(_) => Err(TestCaseError::fail(format!("({:?} {} {})", s_cex, off, is_extended))),
                    }
                })
                .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() })
        }
        "AsciiWordBoundIndicesMatch" => {
            let strat = ascii_text_strategy();
            runner
                .run(&strat, move |s| {
                    c.fetch_add(1, Ordering::Relaxed);
                    let s_cex = s.clone();
                    match property_ascii_word_bound_indices_match(s) {
                        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                        PropertyResult::Fail(_) => Err(TestCaseError::fail(format!("({:?})", s_cex))),
                    }
                })
                .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() })
        }
        _ => {
            return (
                Err(format!("Unknown property for proptest: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = counter.load(Ordering::Relaxed);
    (result, Metrics { inputs, elapsed_us })
}

// ----------------------------- quickcheck -----------------------------------

static QC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn qc_grapheme_next_boundary_empty_chunk(s: AnyText, off_tag: u8, is_extended: bool) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let off = pick_offset(&s.0, off_tag);
    match property_grapheme_next_boundary_empty_chunk_no_panic(s.0, off, is_extended) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_grapheme_prev_boundary_chunk_start(s: AnyText, off_tag: u8, is_extended: bool) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let off = pick_offset(&s.0, off_tag);
    match property_grapheme_prev_boundary_chunk_start_no_panic(s.0, off, is_extended) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_ascii_word_bound_indices_match(s: AsciiText) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_ascii_word_bound_indices_match(s.0) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn run_quickcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_quickcheck_property);
    }
    QC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let mut qc = QuickCheck::new().tests(40_000_000).max_tests(80_000_000);
    let result = match property {
        "GraphemeNextBoundaryEmptyChunk" => qc.quicktest(
            qc_grapheme_next_boundary_empty_chunk as fn(AnyText, u8, bool) -> TestResult,
        ),
        "GraphemePrevBoundaryChunkStart" => qc.quicktest(
            qc_grapheme_prev_boundary_chunk_start as fn(AnyText, u8, bool) -> TestResult,
        ),
        "AsciiWordBoundIndicesMatch" => {
            qc.quicktest(qc_ascii_word_bound_indices_match as fn(AsciiText) -> TestResult)
        }
        _ => {
            return (
                Err(format!("Unknown property for quickcheck: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = QC_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match result.status {
        ResultStatus::Finished => Ok(()),
        ResultStatus::Failed { arguments } => Err(format!(
            "({})",
            arguments.join(" ")
        )),
        ResultStatus::Aborted { err } => Err(format!("quickcheck aborted: {err:?}")),
        ResultStatus::TimedOut => Err("quickcheck timed out".to_string()),
        ResultStatus::GaveUp => Err(format!(
            "quickcheck gave up: passed={}, discarded={}",
            result.n_tests_passed, result.n_tests_discarded
        )),
    };
    (status, metrics)
}

// ----------------------------- crabcheck ------------------------------------

static CC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cc_grapheme_next_boundary_empty_chunk((s, off_tag, is_extended): (AnyText, u8, bool)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let off = pick_offset(&s.0, off_tag);
    match property_grapheme_next_boundary_empty_chunk_no_panic(s.0, off, is_extended) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_grapheme_prev_boundary_chunk_start((s, off_tag, is_extended): (AnyText, u8, bool)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let off = pick_offset(&s.0, off_tag);
    match property_grapheme_prev_boundary_chunk_start_no_panic(s.0, off, is_extended) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_ascii_word_bound_indices_match(s: AsciiText) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_ascii_word_bound_indices_match(s.0) {
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
    let result = match property {
        "GraphemeNextBoundaryEmptyChunk" => {
            crabcheck_qc::quickcheck(cc_grapheme_next_boundary_empty_chunk)
        }
        "GraphemePrevBoundaryChunkStart" => {
            crabcheck_qc::quickcheck(cc_grapheme_prev_boundary_chunk_start)
        }
        "AsciiWordBoundIndicesMatch" => {
            crabcheck_qc::quickcheck(cc_ascii_word_bound_indices_match)
        }
        _ => {
            return (
                Err(format!("Unknown property for crabcheck: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = CC_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match result.status {
        crabcheck_qc::ResultStatus::Finished => Ok(()),
        crabcheck_qc::ResultStatus::Failed { arguments } => {
            Err(format!("({})", arguments.join(" ")))
        },
        crabcheck_qc::ResultStatus::TimedOut => Err("crabcheck timed out".to_string()),
        crabcheck_qc::ResultStatus::GaveUp => Err(format!(
            "crabcheck gave up: passed={}, discarded={}",
            result.passed, result.discarded
        )),
        crabcheck_qc::ResultStatus::Aborted { error } => {
            Err(format!("crabcheck aborted: {error}"))
        }
    };
    (status, metrics)
}

// ------------------------------- hegel --------------------------------------

static HG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn hegel_settings() -> HegelSettings {
    HegelSettings::new().test_cases(40_000_000)
}

fn run_hegel_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_hegel_property);
    }
    HG_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let settings = hegel_settings();
    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match property {
        "GraphemeNextBoundaryEmptyChunk" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let len = tc.draw(hgen::integers::<u32>().min_value(0).max_value(15)) as usize;
                let mut s = String::with_capacity(len * 4);
                for _ in 0..len {
                    loop {
                        let cp = tc.draw(hgen::integers::<u32>().min_value(0).max_value(0x10FFFF));
                        if let Some(c) = char::from_u32(cp) {
                            s.push(c);
                            break;
                        }
                    }
                }
                let off_tag = tc.draw(hgen::integers::<u8>());
                let flag = tc.draw(hgen::integers::<u8>());
                let off = pick_offset(&s, off_tag);
                let is_extended = (flag & 1) == 1;
                let s_cex = s.clone();
                if let PropertyResult::Fail(_) =
                    property_grapheme_next_boundary_empty_chunk_no_panic(s, off, is_extended)
                {
                    panic!("({:?} {} {})", s_cex, off, is_extended);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "GraphemePrevBoundaryChunkStart" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let len = tc.draw(hgen::integers::<u32>().min_value(0).max_value(15)) as usize;
                let mut s = String::with_capacity(len * 4);
                for _ in 0..len {
                    loop {
                        let cp = tc.draw(hgen::integers::<u32>().min_value(0).max_value(0x10FFFF));
                        if let Some(c) = char::from_u32(cp) {
                            s.push(c);
                            break;
                        }
                    }
                }
                let off_tag = tc.draw(hgen::integers::<u8>());
                let flag = tc.draw(hgen::integers::<u8>());
                let off = pick_offset(&s, off_tag);
                let is_extended = (flag & 1) == 1;
                let s_cex = s.clone();
                if let PropertyResult::Fail(_) =
                    property_grapheme_prev_boundary_chunk_start_no_panic(s, off, is_extended)
                {
                    panic!("({:?} {} {})", s_cex, off, is_extended);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "AsciiWordBoundIndicesMatch" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let len = tc.draw(hgen::integers::<u32>().min_value(0).max_value(31)) as usize;
                let mut s = String::with_capacity(len);
                for _ in 0..len {
                    let b = tc.draw(hgen::integers::<u8>().min_value(0).max_value(127));
                    s.push(b as char);
                }
                let s_cex = s.clone();
                if let PropertyResult::Fail(_) = property_ascii_word_bound_indices_match(s) {
                    panic!("({:?})", s_cex);
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
            Err(msg.strip_prefix("Property test failed: ").unwrap_or(&msg).to_string())
        }
    };
    (status, metrics)
}

// ------------------------------ dispatch ------------------------------------

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
            "Properties: GraphemeNextBoundaryEmptyChunk | GraphemePrevBoundaryChunkStart | AsciiWordBoundIndicesMatch | All"
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
            emit_json(
                tool,
                property,
                "aborted",
                Metrics::default(),
                None,
                Some(&format!("adapter panic: {msg}")),
            );
            return;
        }
    };

    match result {
        Ok(()) => emit_json(tool, property, "passed", metrics, None, None),
        Err(msg) => emit_json(tool, property, "failed", metrics, Some(&msg), None),
    }
}
