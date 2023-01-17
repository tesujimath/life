// TODO remove suppression for dead code warning
#![allow(dead_code, unused_variables)]

use super::contig::CartesianContigs;
use std::iter::Iterator;

/// turns separate iterators into iterator of pairs
struct PairwiseOrDefault<I, X> {
    i0: I,
    i1: Option<I>,
    default: X,
}

impl<I, X> PairwiseOrDefault<I, X> {
    fn from<Outer, Inner>(
        rows: Outer,
        default: X,
    ) -> PairwiseOrDefault<<Inner as IntoIterator>::IntoIter, X>
    where
        Outer: IntoIterator<Item = Inner>,
        Inner: IntoIterator<Item = X>,
    {
        let mut rows_iter = rows.into_iter();
        let i0 = rows_iter.by_ref().next().unwrap().into_iter();
        let i1 = rows_iter.by_ref().next().map(|i| i.into_iter());
        assert!(rows_iter.next().is_none(), "too many rows");

        PairwiseOrDefault { i0, i1, default }
    }
}

impl<I, X> Iterator for PairwiseOrDefault<I, X>
where
    I: Iterator<Item = X>,
    X: Copy,
{
    type Item = (X, X);

    fn next(&mut self) -> Option<(X, X)> {
        match &mut self.i1 {
            Some(i2) => match (self.i0.next(), i2.next()) {
                (Some(x1), Some(x2)) => Some((x1, x2)),
                (Some(x1), None) => Some((x1, self.default)),
                (None, Some(x2)) => Some((self.default, x2)),
                (None, None) => None,
            },
            None => self.i0.next().map(|x1| (x1, self.default)),
        }
    }
}

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

impl Playfield {
    fn new() -> Playfield {
        Playfield {
            contigs: CartesianContigs::new(),
        }
    }

    // fn from<I>(rows_of_bytes: &[&[u8]]) -> Playfield {
    //     use std::vec::IntoIter;
    //     for chunk in rows_of_bytes.chunks(2) {
    //         for p in PairwiseOrDefault::<IntoIter<u8>>::from(chunk) {
    //             println!("pair {:?} {:?}", p.0, p.1)
    //         }
    //     }

    //     Playfield::new()
    // }
}

mod tests;
