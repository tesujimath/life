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
    i1: I,
    i2o: Option<I>,
}

impl<I> PairwiseOrDefault<I> {
    fn new<Outer, Inner>(rows: Outer) -> PairwiseOrDefault<<Inner as IntoIterator>::IntoIter>
    where
        Outer: IntoIterator<Item = Inner>,
        Inner: IntoIterator<Item = u8>,
    {
        let mut rows_iter = rows.into_iter();
        let i1 = rows_iter.by_ref().next().unwrap().into_iter();
        let i2o = rows_iter.next().map(|i| i.into_iter());

        PairwiseOrDefault { i1, i2o }
    }
}

impl<I> Iterator for PairwiseOrDefault<I>
where
    I: Iterator<Item = u8>,
{
    type Item = (u8, u8);

    fn next(&mut self) -> Option<(u8, u8)> {
        match &mut self.i2o {
            Some(i2) => match (self.i1.next(), i2.next()) {
                (Some(x1), Some(x2)) => Some((x1, x2)),
                (Some(x1), None) => Some((x1, 0u8)),
                (None, Some(x2)) => Some((0u8, x2)),
                (None, None) => None,
            },
            None => self.i1.next().map(|x1| (x1, 0u8)),
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
