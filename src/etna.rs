//! ETNA framework-neutral property functions for unicode-segmentation.
//!
//! Each `property_<name>` is a pure function taking concrete, owned inputs and
//! returning `PropertyResult`. Framework adapters (proptest/quickcheck/crabcheck/hegel)
//! in `src/bin/etna.rs` and deterministic witness tests in `tests/etna_witnesses.rs`
//! both call these functions directly — there is no re-implementation of the
//! invariant inside any adapter.

#![allow(missing_docs)]

use crate::grapheme::{GraphemeCursor, GraphemeIncomplete};
use crate::UnicodeSegmentation;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub enum PropertyResult {
    Pass,
    Fail(String),
    Discard,
}

/// Invariant: `GraphemeCursor::next_boundary` must never panic when fed a
/// well-formed `(chunk, chunk_start)` pair, even if the chunk happens to be
/// empty at the cursor's offset. The fixed code returns
/// `Err(GraphemeIncomplete::NextChunk)` in that case; the pre-`0f55f70` bug
/// unconditionally called `.unwrap()` on `chars().next()` and panicked.
///
/// Inputs are crafted so that `chunk_start == offset < len` holds — i.e. the
/// caller has handed the cursor an empty suffix-chunk that begins exactly at
/// the cursor's offset. That path must yield a `NextChunk` request, nothing
/// else.
pub fn property_grapheme_next_boundary_empty_chunk_no_panic(
    s: String,
    offset: usize,
    is_extended: bool,
) -> PropertyResult {
    let len = s.len();
    if offset == 0 || offset >= len {
        return PropertyResult::Discard;
    }
    if !s.is_char_boundary(offset) {
        return PropertyResult::Discard;
    }
    let mut cursor = GraphemeCursor::new(offset, len, is_extended);
    let empty = &s[offset..offset];
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cursor.next_boundary(empty, offset)
    })) {
        Ok(Ok(_)) => PropertyResult::Fail(format!(
            "next_boundary returned Ok on empty suffix chunk at offset={offset} (expected NextChunk)"
        )),
        Ok(Err(GraphemeIncomplete::NextChunk)) => PropertyResult::Pass,
        Ok(Err(e)) => PropertyResult::Fail(format!(
            "next_boundary returned unexpected error {e:?} on empty suffix chunk at offset={offset}"
        )),
        Err(payload) => {
            let msg = downcast_panic(&payload);
            PropertyResult::Fail(format!(
                "next_boundary panicked on empty suffix chunk at offset={offset}: {msg}"
            ))
        }
    }
}

/// Invariant: `GraphemeCursor::prev_boundary` must never panic when the caller
/// hands it a suffix chunk whose start coincides with the cursor's offset. The
/// fixed code (commit `fb5d7b6`) returns `Err(GraphemeIncomplete::PrevChunk)`
/// immediately; the buggy version sliced `chunk[..0]` and then called
/// `chars().rev().next().unwrap()`, which panicked on an empty iterator.
pub fn property_grapheme_prev_boundary_chunk_start_no_panic(
    s: String,
    offset: usize,
    is_extended: bool,
) -> PropertyResult {
    let len = s.len();
    if offset == 0 || offset > len {
        return PropertyResult::Discard;
    }
    if !s.is_char_boundary(offset) {
        return PropertyResult::Discard;
    }
    let mut cursor = GraphemeCursor::new(offset, len, is_extended);
    let suffix = &s[offset..];
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cursor.prev_boundary(suffix, offset)
    })) {
        Ok(Ok(_)) => PropertyResult::Fail(format!(
            "prev_boundary returned Ok on offset==chunk_start at offset={offset} (expected PrevChunk)"
        )),
        Ok(Err(GraphemeIncomplete::PrevChunk)) => PropertyResult::Pass,
        Ok(Err(e)) => PropertyResult::Fail(format!(
            "prev_boundary returned unexpected error {e:?} on offset==chunk_start at offset={offset}"
        )),
        Err(payload) => {
            let msg = downcast_panic(&payload);
            PropertyResult::Fail(format!(
                "prev_boundary panicked on offset==chunk_start at offset={offset}: {msg}"
            ))
        }
    }
}

/// Invariant: for any ASCII-only input, the ASCII fast-path word-bound
/// iterator (`new_ascii_word_bound_indices`) must yield the exact same
/// `(usize, &str)` pairs as the general Unicode word-bound iterator
/// (`split_word_bound_indices`). This is lifted verbatim from the in-tree
/// proptest at `src/word.rs` — any mutation in the word-bound state machine
/// that breaks ASCII parity with the Unicode path will surface here.
pub fn property_ascii_word_bound_indices_match(s: String) -> PropertyResult {
    if !s.is_ascii() {
        return PropertyResult::Discard;
    }
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let fast: Vec<(usize, &str)> = crate::word::new_ascii_word_bound_indices(&s).collect();
        let uni: Vec<(usize, &str)> = s.split_word_bound_indices().collect();
        (fast, uni)
    })) {
        Ok((fast, uni)) => {
            if fast == uni {
                PropertyResult::Pass
            } else {
                PropertyResult::Fail(format!(
                    "ASCII fast path vs Unicode path diverge for {s:?}: fast={fast:?}, uni={uni:?}"
                ))
            }
        }
        Err(payload) => {
            let msg = downcast_panic(&payload);
            PropertyResult::Fail(format!(
                "word-bound iteration panicked for {s:?}: {msg}"
            ))
        }
    }
}

fn downcast_panic(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else {
        "<non-string panic payload>".to_string()
    }
}
