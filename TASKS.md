# unicode-segmentation — ETNA Tasks

Total tasks: 12

## Task Index

| Task | Variant | Framework | Property | Witness |
|------|---------|-----------|----------|---------|
| 001 | `ascii_word_bound_drop_apostrophe_af87c8d_1` | proptest | `AsciiWordBoundIndicesMatch` | `witness_ascii_word_bound_indices_match_case_cant_apostrophe` |
| 002 | `ascii_word_bound_drop_apostrophe_af87c8d_1` | quickcheck | `AsciiWordBoundIndicesMatch` | `witness_ascii_word_bound_indices_match_case_cant_apostrophe` |
| 003 | `ascii_word_bound_drop_apostrophe_af87c8d_1` | crabcheck | `AsciiWordBoundIndicesMatch` | `witness_ascii_word_bound_indices_match_case_cant_apostrophe` |
| 004 | `ascii_word_bound_drop_apostrophe_af87c8d_1` | hegel | `AsciiWordBoundIndicesMatch` | `witness_ascii_word_bound_indices_match_case_cant_apostrophe` |
| 005 | `grapheme_next_boundary_unwrap_0f55f70_1` | proptest | `GraphemeNextBoundaryEmptyChunk` | `witness_grapheme_next_boundary_empty_chunk_no_panic_case_ascii_suffix` |
| 006 | `grapheme_next_boundary_unwrap_0f55f70_1` | quickcheck | `GraphemeNextBoundaryEmptyChunk` | `witness_grapheme_next_boundary_empty_chunk_no_panic_case_ascii_suffix` |
| 007 | `grapheme_next_boundary_unwrap_0f55f70_1` | crabcheck | `GraphemeNextBoundaryEmptyChunk` | `witness_grapheme_next_boundary_empty_chunk_no_panic_case_ascii_suffix` |
| 008 | `grapheme_next_boundary_unwrap_0f55f70_1` | hegel | `GraphemeNextBoundaryEmptyChunk` | `witness_grapheme_next_boundary_empty_chunk_no_panic_case_ascii_suffix` |
| 009 | `grapheme_prev_boundary_chunk_start_fb5d7b6_1` | proptest | `GraphemePrevBoundaryChunkStart` | `witness_grapheme_prev_boundary_chunk_start_no_panic_case_ascii` |
| 010 | `grapheme_prev_boundary_chunk_start_fb5d7b6_1` | quickcheck | `GraphemePrevBoundaryChunkStart` | `witness_grapheme_prev_boundary_chunk_start_no_panic_case_ascii` |
| 011 | `grapheme_prev_boundary_chunk_start_fb5d7b6_1` | crabcheck | `GraphemePrevBoundaryChunkStart` | `witness_grapheme_prev_boundary_chunk_start_no_panic_case_ascii` |
| 012 | `grapheme_prev_boundary_chunk_start_fb5d7b6_1` | hegel | `GraphemePrevBoundaryChunkStart` | `witness_grapheme_prev_boundary_chunk_start_no_panic_case_ascii` |

## Witness Catalog

- `witness_ascii_word_bound_indices_match_case_cant_apostrophe` — base passes, variant fails
- `witness_grapheme_next_boundary_empty_chunk_no_panic_case_ascii_suffix` — base passes, variant fails
- `witness_grapheme_next_boundary_empty_chunk_no_panic_case_legacy_mode` — base passes, variant fails
- `witness_grapheme_next_boundary_empty_chunk_no_panic_case_multibyte` — base passes, variant fails
- `witness_grapheme_prev_boundary_chunk_start_no_panic_case_ascii` — base passes, variant fails
- `witness_grapheme_prev_boundary_chunk_start_no_panic_case_ascii_end` — base passes, variant fails
- `witness_grapheme_prev_boundary_chunk_start_no_panic_case_legacy` — base passes, variant fails
