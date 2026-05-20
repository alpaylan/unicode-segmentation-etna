//! Fault-localization integration tests for unicode-segmentation.

use std::fmt;

use crabcheck::quickcheck::{Arbitrary, Mutate};
use rand::Rng;
use unicode_segmentation::etna::{
    property_ascii_word_bound_indices_match, property_grapheme_next_boundary_empty_chunk_no_panic,
    property_grapheme_prev_boundary_chunk_start_no_panic, PropertyResult,
};

#[derive(Clone)]
struct AnyText(String);
impl fmt::Debug for AnyText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

#[derive(Clone)]
struct AsciiText(String);
impl fmt::Debug for AsciiText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

fn draw_char<R: Rng>(rng: &mut R) -> char {
    loop {
        let cp = rng.random_range(0u32..=0x10FFFFu32);
        if let Some(c) = char::from_u32(cp) { return c; }
    }
}

impl<R: Rng> Arbitrary<R> for AnyText {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = rng.random_range(0..16u32) as usize;
        let mut s = String::with_capacity(len * 4);
        for _ in 0..len { s.push(draw_char(rng)); }
        AnyText(s)
    }
}

impl<R: Rng> Arbitrary<R> for AsciiText {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = rng.random_range(0..32u32) as usize;
        let mut s = String::with_capacity(len);
        for _ in 0..len { s.push(rng.random_range(0u8..=127u8) as char); }
        AsciiText(s)
    }
}

impl<R: Rng> Mutate<R> for AnyText {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut chars: Vec<char> = self.0.chars().collect();
        match rng.random_range(0u8..3) {
            0 if !chars.is_empty() => {
                let i = rng.random_range(0..chars.len());
                chars[i] = draw_char(rng);
            }
            1 if chars.len() < 16 => chars.push(draw_char(rng)),
            _ if !chars.is_empty() => { chars.pop(); }
            _ => {}
        }
        AnyText(chars.into_iter().collect())
    }
}

impl<R: Rng> Mutate<R> for AsciiText {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut bytes = self.0.as_bytes().to_vec();
        match rng.random_range(0u8..3) {
            0 if !bytes.is_empty() => {
                let i = rng.random_range(0..bytes.len());
                bytes[i] = rng.random_range(0u8..=127u8);
            }
            1 if bytes.len() < 32 => bytes.push(rng.random_range(0u8..=127u8)),
            _ if !bytes.is_empty() => { bytes.pop(); }
            _ => {}
        }
        AsciiText(String::from_utf8(bytes).unwrap_or_default())
    }
}

fn pick_offset(s: &str, tag: u8) -> usize {
    if s.is_empty() { return 0; }
    let mut candidate = (tag as usize) % (s.len() + 1);
    while candidate > 0 && !s.is_char_boundary(candidate) { candidate -= 1; }
    candidate
}

fn to_opt(r: PropertyResult) -> Option<bool> {
    match r {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn property_grapheme_next_boundary_empty_chunk_test(input: (AnyText, usize, bool)) -> Option<bool> {
    let (AnyText(s), tag, ext) = input;
    let off = pick_offset(&s, tag as u8);
    to_opt(property_grapheme_next_boundary_empty_chunk_no_panic(s, off, ext))
}

fn property_grapheme_prev_boundary_chunk_start_test(input: (AnyText, usize, bool)) -> Option<bool> {
    let (AnyText(s), tag, ext) = input;
    let off = pick_offset(&s, tag as u8);
    to_opt(property_grapheme_prev_boundary_chunk_start_no_panic(s, off, ext))
}

fn property_ascii_word_bound_indices_match_test(AsciiText(s): AsciiText) -> Option<bool> {
    to_opt(property_ascii_word_bound_indices_match(s))
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
        serde_json::json!({
            "rank": s.rank, "file": s.region.file, "function": s.region.function,
            "start_line": s.region.start_line, "end_line": s.region.end_line,
            "ochiai": s.region.suspiciousness.ochiai, "delta": s.region.delta,
            "panic_overlap": s.panic_overlap,
            "confidence": format!("{}", s.confidence),
            "confidence_rule": s.confidence_rule,
        })
    } else { serde_json::Value::Null };
    let top_5: Vec<_> = r.suspects.iter().take(5).map(|s| serde_json::json!({
        "rank": s.rank, "file": s.region.file, "function": s.region.function,
        "start_line": s.region.start_line, "end_line": s.region.end_line,
        "confidence": format!("{}", s.confidence),
        "confidence_rule": s.confidence_rule,
        "panic_overlap": s.panic_overlap,
    })).collect();
    let diags: Vec<_> = r.diagnostics.iter().map(|d| d.tag()).collect();
    let out = serde_json::json!({
        "status": status, "passed": r.run.passed, "discarded": r.run.discarded,
        "n_panics": r.n_panics, "n_suspects": r.suspects.len(),
        "top": top, "top_5": top_5, "diagnostics": diags,
    });
    println!("@@LOCATE@@ {}", out);
}

#[test]
fn locate_grapheme_next_boundary_empty_chunk() {
    let report = crabcheck::quickcheck_with_locate!(property_grapheme_next_boundary_empty_chunk_test, "unicode_segmentation");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_grapheme_prev_boundary_chunk_start() {
    let report = crabcheck::quickcheck_with_locate!(property_grapheme_prev_boundary_chunk_start_test, "unicode_segmentation");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_ascii_word_bound_indices_match() {
    let report = crabcheck::quickcheck_with_locate!(property_ascii_word_bound_indices_match_test, "unicode_segmentation");
    eprintln!("{report}");
    emit_locate_json(&report);
}
