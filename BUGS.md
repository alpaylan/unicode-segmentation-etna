# unicode-segmentation — Injected Bugs

Total mutations: 3

## Bug Index

| # | Variant | Name | Location | Injection | Fix Commit |
|---|---------|------|----------|-----------|------------|
| 1 | `ascii_word_bound_drop_apostrophe_af87c8d_1` | `ascii_word_bound_drop_apostrophe` | `src/word.rs` | `patch` | `af87c8d331b81d2398997d89d2164c252ae6e9f5` |
| 2 | `grapheme_next_boundary_unwrap_0f55f70_1` | `grapheme_next_boundary_unwrap` | `src/grapheme.rs` | `patch` | `0f55f70b445202fd9d3c101b9936e6649e808441` |
| 3 | `grapheme_prev_boundary_chunk_start_fb5d7b6_1` | `grapheme_prev_boundary_chunk_start` | `src/grapheme.rs` | `patch` | `fb5d7b6714d265aae844ce8f7df35d675505026f` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `ascii_word_bound_drop_apostrophe_af87c8d_1` | `AsciiWordBoundIndicesMatch` | `witness_ascii_word_bound_indices_match_case_cant_apostrophe` |
| `grapheme_next_boundary_unwrap_0f55f70_1` | `GraphemeNextBoundaryEmptyChunk` | `witness_grapheme_next_boundary_empty_chunk_no_panic_case_ascii_suffix`, `witness_grapheme_next_boundary_empty_chunk_no_panic_case_legacy_mode`, `witness_grapheme_next_boundary_empty_chunk_no_panic_case_multibyte` |
| `grapheme_prev_boundary_chunk_start_fb5d7b6_1` | `GraphemePrevBoundaryChunkStart` | `witness_grapheme_prev_boundary_chunk_start_no_panic_case_ascii`, `witness_grapheme_prev_boundary_chunk_start_no_panic_case_ascii_end`, `witness_grapheme_prev_boundary_chunk_start_no_panic_case_legacy` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `AsciiWordBoundIndicesMatch` | ✓ | ✓ | ✓ | ✓ |
| `GraphemeNextBoundaryEmptyChunk` | ✓ | ✓ | ✓ | ✓ |
| `GraphemePrevBoundaryChunkStart` | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. ascii_word_bound_drop_apostrophe

- **Variant**: `ascii_word_bound_drop_apostrophe_af87c8d_1`
- **Location**: `src/word.rs`
- **Property**: `AsciiWordBoundIndicesMatch`
- **Witness(es)**:
  - `witness_ascii_word_bound_indices_match_case_cant_apostrophe`
- **Source**: [#147](https://github.com/unicode-rs/unicode-segmentation/pull/147) — fast ascii path for word boundary indices (#147)
  > The ASCII fast path in `split_word_bound_indices` classified `b'\''` as MidNumLetQ between two alphabetic bytes to match UAX#29 WB6/WB7; dropping it from `is_infix` makes `"can't"` split into three segments instead of one, diverging from the Unicode path.
- **Fix commit**: `af87c8d331b81d2398997d89d2164c252ae6e9f5` — fast ascii path for word boundary indices (#147)
- **Invariant violated**: For any `s.is_ascii()`, the ASCII fast path iterator must produce the same `(usize, &str)` sequence as the Unicode `split_word_bound_indices` iterator.
- **How the mutation triggers**: The fix's `is_infix` arm accepts `b'.'`, `b':'`, and `b'\''`; the patch drops the apostrophe, so `"can't"` yields `[(0, "can"), (3, "'"), (4, "t")]` on the ASCII path but `[(0, "can't")]` on the Unicode path.

### 2. grapheme_next_boundary_unwrap

- **Variant**: `grapheme_next_boundary_unwrap_0f55f70_1`
- **Location**: `src/grapheme.rs`
- **Property**: `GraphemeNextBoundaryEmptyChunk`
- **Witness(es)**:
  - `witness_grapheme_next_boundary_empty_chunk_no_panic_case_ascii_suffix`
  - `witness_grapheme_next_boundary_empty_chunk_no_panic_case_legacy_mode`
  - `witness_grapheme_next_boundary_empty_chunk_no_panic_case_multibyte`
- **Source**: [#137](https://github.com/unicode-rs/unicode-segmentation/pull/137) — Fix unwrap panic in next_boundary() (#137)
  > `GraphemeCursor::next_boundary` called `chunk.chars().next().unwrap()` on the provided chunk slice; when the caller handed in an empty chunk at the cursor's own offset, the unwrap panicked instead of returning `Err(GraphemeIncomplete::NextChunk)`.
- **Fix commit**: `0f55f70b445202fd9d3c101b9936e6649e808441` — Fix unwrap panic in next_boundary() (#137)
- **Invariant violated**: `GraphemeCursor::next_boundary(chunk, chunk_start)` must not panic when called with an empty chunk at `chunk_start == cursor.offset`; it must return `Err(GraphemeIncomplete::NextChunk)` so the caller can supply the next chunk.
- **How the mutation triggers**: The fix replaced `chunks().next().unwrap()` with a `match` that returns `Err(NextChunk)` when the iterator is empty; the patch reverts to the unguarded unwrap, so any `chunk_start == cursor.offset < len` input panics on the empty iterator.

### 3. grapheme_prev_boundary_chunk_start

- **Variant**: `grapheme_prev_boundary_chunk_start_fb5d7b6_1`
- **Location**: `src/grapheme.rs`
- **Property**: `GraphemePrevBoundaryChunkStart`
- **Witness(es)**:
  - `witness_grapheme_prev_boundary_chunk_start_no_panic_case_ascii`
  - `witness_grapheme_prev_boundary_chunk_start_no_panic_case_ascii_end`
  - `witness_grapheme_prev_boundary_chunk_start_no_panic_case_legacy`
- **Source**: [#38](https://github.com/unicode-rs/unicode-segmentation/issues/38), [#39](https://github.com/unicode-rs/unicode-segmentation/issues/39) — fix crashes on prev_boundary
  > `GraphemeCursor::prev_boundary` sliced `chunk[..0]` and unwrapped the reversed iterator when the chunk started exactly at the cursor's offset, panicking instead of returning `Err(GraphemeIncomplete::PrevChunk)`.
- **Fix commit**: `fb5d7b6714d265aae844ce8f7df35d675505026f` — fix crashes on prev_boundary
- **Invariant violated**: `GraphemeCursor::prev_boundary(chunk, chunk_start)` with `chunk_start == cursor.offset` must return `Err(GraphemeIncomplete::PrevChunk)`; it must not panic.
- **How the mutation triggers**: The fix added `if self.offset == chunk_start { return Err(PrevChunk); }` early-return; the patch removes that guard, so the next line's `chars().rev().next().unwrap()` on an empty `chunk[..0]` panics.
