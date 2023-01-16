// TODO remove suppression for dead code warning
#![allow(dead_code, unused_variables)]

use super::contig::CartesianContigs;
use std::iter::Iterator;
use std::slice::Iter;

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

struct PairwiseOrZeroes<'a> {
    i1: Iter<'a, u8>,
    i2o: Option<Iter<'a, u8>>,
}

impl<'a> PairwiseOrZeroes<'a> {
    fn new(rows: &'a [&'a [u8]]) -> PairwiseOrZeroes<'a> {
        PairwiseOrZeroes {
            i1: rows[0].iter(),
            i2o: rows.get(1).map(|r| r.iter()),
        }
    }
}

impl<'a> Iterator for PairwiseOrZeroes<'a> {
    type Item = (&'a u8, &'a u8);

    fn next(&mut self) -> Option<(&'a u8, &'a u8)> {
        match &mut self.i2o {
            Some(i2) => match (self.i1.next(), i2.next()) {
                (Some(x1), Some(x2)) => Some((x1, x2)),
                (Some(x1), None) => Some((x1, &0u8)),
                (None, Some(x2)) => Some((&0u8, x2)),
                (None, None) => None,
            },
            None => self.i1.next().map(|x1| (x1, &0u8)),
        }
    }
}

impl Playfield {
    fn new() -> Playfield {
        Playfield {
            contigs: CartesianContigs::new(),
        }
    }

    fn from<I>(rows_of_bytes: &[&[u8]]) -> Playfield {
        for chunk in rows_of_bytes.chunks(2) {
            for p in PairwiseOrZeroes::new(chunk) {
                println!("pair {:?} {:?}", p.0, p.1)
            }
        }

        Playfield::new()
    }
}

mod tests;
