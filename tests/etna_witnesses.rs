//! Deterministic witness tests for the ETNA unicode-segmentation workload.
//!
//! Each `witness_<name>_case_<tag>` is a concrete `#[test]` that feeds the
//! framework-neutral `property_<name>` function frozen inputs. Base HEAD must
//! pass every witness; each `etna/<variant>` branch (via a patch in
//! `patches/`) must make the matching witness fail — that's how the runner's
//! validation stage detects the injected bug.

use unicode_segmentation::etna::{
    property_ascii_word_bound_indices_match, property_grapheme_next_boundary_empty_chunk_no_panic,
    property_grapheme_prev_boundary_chunk_start_no_panic, PropertyResult,
};

fn assert_pass(r: PropertyResult, ctx: &str) {
    match r {
        PropertyResult::Pass => {}
        PropertyResult::Discard => {
            panic!(
                "{}: witness inputs are out of domain (Discard) — fix the witness",
                ctx
            )
        }
        PropertyResult::Fail(m) => panic!("{}: {}", ctx, m),
    }
}

// ------- grapheme_next_boundary_empty_chunk -------

#[test]
fn witness_grapheme_next_boundary_empty_chunk_no_panic_case_ascii_suffix() {
    assert_pass(
        property_grapheme_next_boundary_empty_chunk_no_panic("abcdefgh".to_string(), 4, true),
        "ascii suffix offset=4",
    );
}

#[test]
fn witness_grapheme_next_boundary_empty_chunk_no_panic_case_legacy_mode() {
    assert_pass(
        property_grapheme_next_boundary_empty_chunk_no_panic("abcdefgh".to_string(), 4, false),
        "ascii suffix offset=4 legacy",
    );
}

#[test]
fn witness_grapheme_next_boundary_empty_chunk_no_panic_case_multibyte() {
    // "héllo" — é is 2 bytes at offset 1..3. Empty chunk starting at offset 3.
    assert_pass(
        property_grapheme_next_boundary_empty_chunk_no_panic("héllo".to_string(), 3, true),
        "multibyte suffix offset=3",
    );
}

// ------- grapheme_prev_boundary_chunk_start -------

#[test]
fn witness_grapheme_prev_boundary_chunk_start_no_panic_case_ascii() {
    assert_pass(
        property_grapheme_prev_boundary_chunk_start_no_panic("abcd".to_string(), 2, true),
        "ascii chunk_start=2",
    );
}

#[test]
fn witness_grapheme_prev_boundary_chunk_start_no_panic_case_ascii_end() {
    assert_pass(
        property_grapheme_prev_boundary_chunk_start_no_panic("abcd".to_string(), 4, true),
        "ascii chunk_start=4 (end)",
    );
}

#[test]
fn witness_grapheme_prev_boundary_chunk_start_no_panic_case_legacy() {
    assert_pass(
        property_grapheme_prev_boundary_chunk_start_no_panic("abcd".to_string(), 2, false),
        "ascii chunk_start=2 legacy",
    );
}

// ------- ascii_word_bound_indices_match -------

#[test]
fn witness_ascii_word_bound_indices_match_case_hello_comma_world() {
    assert_pass(
        property_ascii_word_bound_indices_match("Hello, world!".to_string()),
        "Hello, world!",
    );
}

#[test]
fn witness_ascii_word_bound_indices_match_case_cant_apostrophe() {
    assert_pass(
        property_ascii_word_bound_indices_match("can't".to_string()),
        "can't",
    );
}

#[test]
fn witness_ascii_word_bound_indices_match_case_mixed_numeric_punct() {
    assert_pass(
        property_ascii_word_bound_indices_match("127.0.0.1:9090".to_string()),
        "127.0.0.1:9090",
    );
}
