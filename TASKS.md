# Tasks

One [[variant]] from `etna.toml` × one framework = one task. 3 variants ×
4 frameworks = **12 tasks**. Each task is exercised by:

- `cargo run --release --features etna --bin etna -- <framework> <property>`
  on base HEAD (must print `"status":"passed"`)
- The same command on `etna/<variant>` (must print `"status":"failed"` and
  include a non-empty `counterexample`)

## Task Index

| # | Variant                                           | Framework   | Property (runner arg)                  |
|---|---------------------------------------------------|-------------|----------------------------------------|
| 01 | grapheme_next_boundary_unwrap_0f55f70_1          | proptest    | GraphemeNextBoundaryEmptyChunk        |
| 02 | grapheme_next_boundary_unwrap_0f55f70_1          | quickcheck  | GraphemeNextBoundaryEmptyChunk        |
| 03 | grapheme_next_boundary_unwrap_0f55f70_1          | crabcheck   | GraphemeNextBoundaryEmptyChunk        |
| 04 | grapheme_next_boundary_unwrap_0f55f70_1          | hegel       | GraphemeNextBoundaryEmptyChunk        |
| 05 | grapheme_prev_boundary_chunk_start_fb5d7b6_1     | proptest    | GraphemePrevBoundaryChunkStart        |
| 06 | grapheme_prev_boundary_chunk_start_fb5d7b6_1     | quickcheck  | GraphemePrevBoundaryChunkStart        |
| 07 | grapheme_prev_boundary_chunk_start_fb5d7b6_1     | crabcheck   | GraphemePrevBoundaryChunkStart        |
| 08 | grapheme_prev_boundary_chunk_start_fb5d7b6_1     | hegel       | GraphemePrevBoundaryChunkStart        |
| 09 | ascii_word_bound_drop_apostrophe_af87c8d_1       | proptest    | AsciiWordBoundIndicesMatch            |
| 10 | ascii_word_bound_drop_apostrophe_af87c8d_1       | quickcheck  | AsciiWordBoundIndicesMatch            |
| 11 | ascii_word_bound_drop_apostrophe_af87c8d_1       | crabcheck   | AsciiWordBoundIndicesMatch            |
| 12 | ascii_word_bound_drop_apostrophe_af87c8d_1       | hegel       | AsciiWordBoundIndicesMatch            |

## Witness Catalog

Deterministic witness tests live in `tests/etna_witnesses.rs`. Base must pass
every witness. Each variant must make at least one matching witness fail:

| Property                                       | Witness tests                                                                 |
|------------------------------------------------|-------------------------------------------------------------------------------|
| `grapheme_next_boundary_empty_chunk_no_panic`  | `_case_ascii_suffix`, `_case_legacy_mode`, `_case_multibyte`                  |
| `grapheme_prev_boundary_chunk_start_no_panic`  | `_case_ascii`, `_case_ascii_end`, `_case_legacy`                              |
| `ascii_word_bound_indices_match`               | `_case_hello_comma_world`, `_case_cant_apostrophe`, `_case_mixed_numeric_punct` |

For the `ascii_word_bound` variant, only `_case_cant_apostrophe` fails on
the variant branch — the other two witnesses are passing sanity inputs that
stay green everywhere, which keeps the property function honest and
prevents spurious breaks from unrelated refactors.
