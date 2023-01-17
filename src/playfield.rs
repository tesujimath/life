// TODO remove suppression for dead code warning
#![allow(dead_code, unused_variables)]

use super::contig::CartesianContigs;
use std::iter::Iterator;

/// Array of bits, organised as double rows.
/// Each item represents two rows of bits of width `BLOCKSIZE`.
///
/// The reason for double rows is so that non-contiguous rows cannot both influence
/// new cell birth in the intermediate row, which would otherwise be possible.
struct Playfield {
    contigs: CartesianContigs<i32, u8>,
}

/// half of the width in bits of the playfield storage type
const BLOCKSIZE: usize = 4;
const ZEROBLOCK: u8 = 0;

struct PairwiseOrDefault<I> {
    i0: I,
    i1: Option<I>,
}

impl<I> PairwiseOrDefault<I> {
    fn from<Outer, Inner, X>(rows: Outer) -> PairwiseOrDefault<<Inner as IntoIterator>::IntoIter>
    where
        Outer: IntoIterator<Item = Inner>,
        Inner: IntoIterator<Item = X>,
    {
        let mut rows_iter = rows.into_iter();
        let i0 = rows_iter.by_ref().next().unwrap().into_iter();
        let i1o = rows_iter.next().map(|i| i.into_iter());

        PairwiseOrDefault { i0, i1: i1o }
    }
}

impl<I, X> Iterator for PairwiseOrDefault<I>
where
    I: Iterator<Item = X>,
    X: Default,
{
    type Item = (X, X);

    fn next(&mut self) -> Option<(X, X)> {
        match &mut self.i1 {
            Some(i2) => match (self.i0.next(), i2.next()) {
                (Some(x1), Some(x2)) => Some((x1, x2)),
                (Some(x1), None) => Some((x1, X::default())),
                (None, Some(x2)) => Some((X::default(), x2)),
                (None, None) => None,
            },
            None => self.i0.next().map(|x1| (x1, X::default())),
        }
    }
}

impl Playfield {
    fn new() -> Playfield {
        Playfield {
            contigs: CartesianContigs::new(),
        }
    }

    // fn from<I>(rows_of_bytes: &[&[u8]]) -> Playfield {
    //     for chunk in rows_of_bytes.chunks(2) {
    //         for p in PairwiseOrZeroes::new(chunk) {
    //             println!("pair {:?} {:?}", p.0, p.1)
    //         }
    //     }

    //     Playfield::new()
    // }
}

mod tests;
