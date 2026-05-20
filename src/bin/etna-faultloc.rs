use std::fmt;

use crabcheck::profiling::quickcheck;
use crabcheck::quickcheck::{Arbitrary, Mutate};
use rand::Rng;
use unicode_segmentation::etna::{
    property_ascii_word_bound_indices_match, property_grapheme_next_boundary_empty_chunk_no_panic,
    property_grapheme_prev_boundary_chunk_start_no_panic, PropertyResult,
};

#[derive(Clone)]
struct AnyText(String);
impl fmt::Debug for AnyText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone)]
struct AsciiText(String);
impl fmt::Debug for AsciiText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

fn draw_char<R: Rng>(rng: &mut R) -> char {
    loop {
        let cp = rng.random_range(0u32..=0x10FFFFu32);
        if let Some(c) = char::from_u32(cp) {
            return c;
        }
    }
}

impl<R: Rng> Arbitrary<R> for AnyText {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = rng.random_range(0..16u32) as usize;
        let mut s = String::with_capacity(len * 4);
        for _ in 0..len {
            s.push(draw_char(rng));
        }
        AnyText(s)
    }
}

impl<R: Rng> Arbitrary<R> for AsciiText {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = rng.random_range(0..32u32) as usize;
        let mut s = String::with_capacity(len);
        for _ in 0..len {
            s.push(rng.random_range(0u8..=127u8) as char);
        }
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
            _ if !chars.is_empty() => {
                chars.pop();
            }
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
            _ if !bytes.is_empty() => {
                bytes.pop();
            }
            _ => {}
        }
        AsciiText(String::from_utf8(bytes).unwrap_or_default())
    }
}

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

fn to_opt(r: PropertyResult) -> Option<bool> {
    match r {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        return;
    }
    let result = match (args[1].as_str(), args[2].as_str()) {
        ("crabcheck", "GraphemeNextBoundaryEmptyChunk") => {
            quickcheck(|(AnyText(s), tag, ext): (AnyText, usize, bool)| {
                let off = pick_offset(&s, tag as u8);
                {
                    to_opt(property_grapheme_next_boundary_empty_chunk_no_panic(s, off, ext))
                }
            })
        }
        ("crabcheck", "GraphemePrevBoundaryChunkStart") => {
            quickcheck(|(AnyText(s), tag, ext): (AnyText, usize, bool)| {
                let off = pick_offset(&s, tag as u8);
                {
                    to_opt(property_grapheme_prev_boundary_chunk_start_no_panic(s, off, ext))
                }
            })
        }
        ("crabcheck", "AsciiWordBoundIndicesMatch") => quickcheck(|AsciiText(s)| {
            to_opt(property_ascii_word_bound_indices_match(s))
        }),
        (a, b) => panic!("Unknown: {a} {b}"),
    };
    println!("Result: {:?}", result);
}
