# Bugs

This workload injects three regressions into `unicode-segmentation`. Each
maps to one `etna/<variant>` branch that applies a patch from `patches/`,
and one framework-neutral `property_<name>` function in `src/etna.rs`.

## Bug Index

| Variant                                           | Origin (fix commit)      | Target file       |
|---------------------------------------------------|--------------------------|-------------------|
| `grapheme_next_boundary_unwrap_0f55f70_1`         | `0f55f70` (#137, 2024)   | `src/grapheme.rs` |
| `grapheme_prev_boundary_chunk_start_fb5d7b6_1`    | `fb5d7b6` (#38,#39, 2018)| `src/grapheme.rs` |
| `ascii_word_bound_drop_apostrophe_af87c8d_1`      | `af87c8d` (#147, 2025)   | `src/word.rs`     |

## Property Mapping

| Variant                                           | Property (framework-neutral)                       |
|---------------------------------------------------|----------------------------------------------------|
| `grapheme_next_boundary_unwrap_0f55f70_1`         | `property_grapheme_next_boundary_empty_chunk_no_panic` |
| `grapheme_prev_boundary_chunk_start_fb5d7b6_1`    | `property_grapheme_prev_boundary_chunk_start_no_panic` |
| `ascii_word_bound_drop_apostrophe_af87c8d_1`      | `property_ascii_word_bound_indices_match`          |

## Framework Coverage

Every variant has adapters for all four frameworks. The runner binary
`src/bin/etna.rs` dispatches on `<tool> <property>`.

| Framework   | Adapter signature                                  |
|-------------|----------------------------------------------------|
| proptest    | closure over `(String, u8, bool)` / `String`       |
| quickcheck  | `fn(u8, u8, bool)` / `fn(u8)` (avoids `String::Arbitrary`, which panics when `g.size() == 0`) |
| crabcheck   | `fn((usize, usize, usize))` / `fn(usize)` (same reason as quickcheck) |
| hegel       | `TestCase::draw` of `integers::<u8>()` / `text()`  |

Each adapter calls the corresponding `property_<name>` function directly, so
there is no re-implementation of the invariant inside any framework.

## Bug Details

### `grapheme_next_boundary_unwrap_0f55f70_1`

The fix in commit `0f55f70b` (PR #137, "Fix unwrap panic in next_boundary()")
replaced `chars().next().unwrap()` with a `match` that returns
`Err(GraphemeIncomplete::NextChunk)` when the caller hands the cursor an
empty chunk at the cursor's offset:

```rust
let mut iter = chunk[self.offset.saturating_sub(chunk_start)..].chars();
let mut ch = match iter.next() {
    Some(ch) => ch,
    None => return Err(GraphemeIncomplete::NextChunk),
};
```

The patch reverts to the pre-fix form. On any `chunk_start == offset < len`
input the iterator is empty and `.unwrap()` panics.

**Property.** `property_grapheme_next_boundary_empty_chunk_no_panic(s,
offset, is_extended)` constructs a `GraphemeCursor` at `offset`, passes it
`&s[offset..offset]` (an empty suffix chunk) and expects either
`Err(NextChunk)` or `Ok(_)`; any panic counts as a failure.

### `grapheme_prev_boundary_chunk_start_fb5d7b6_1`

The fix in commit `fb5d7b6` ("fix crashes on prev_boundary", issues #38/#39)
added an early-return when the chunk exactly starts at the cursor offset:

```rust
if self.offset == chunk_start {
    return Err(GraphemeIncomplete::PrevChunk);
}
```

The patch removes that guard. The next line slices `chunk[..0]` and calls
`chars().rev().next().unwrap()`, which panics.

**Property.** `property_grapheme_prev_boundary_chunk_start_no_panic(s,
offset, is_extended)` constructs a `GraphemeCursor` at `offset` and passes
`&s[offset..]` as the chunk with `chunk_start == offset`. Must return
`Err(PrevChunk)`; any panic or `Ok(_)` is a failure.

### `ascii_word_bound_drop_apostrophe_af87c8d_1`

Commit `af87c8d` (PR #147) added an ASCII fast path for word-bound indices
iteration. The fast path classifies `b'\''` as MidNumLetQ between two
alphabetic bytes, matching UAX#29 WB6/WB7 so that words like `"can't"` are
one segment.

The patch drops `b'\''` from the `is_infix` arm, leaving only `b'.'` and
`b':'`. The Unicode path (`split_word_bound_indices`) is unchanged. For
input `"can't"` the fast path yields `[(0, "can"), (3, "'"), (4, "t")]`
while the Unicode path returns `[(0, "can't")]`.

**Property.** `property_ascii_word_bound_indices_match(s)` requires that
for any `s.is_ascii()`, the fast path iterator and `split_word_bound_indices`
produce identical `(usize, &str)` sequences.
