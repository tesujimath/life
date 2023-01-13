// TODO remove suppression for dead code warning
#![allow(dead_code)]

use super::contig::CartesianContigs;
use std::iter::repeat;
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

impl Playfield {
    fn new() -> Playfield {
        Playfield {
            contigs: CartesianContigs::new(),
        }
    }

    fn from<I>(rows_of_bytes: &[&[u8]]) -> Playfield {
        for double_row in rows_of_bytes.chunks(2) {
            let r0 = double_row[0usize].iter();
            let zeros = &repeat(ZEROBLOCK) as &dyn Iterator<Item = u8>;
            let r1 = double_row
                .get(1)
                .copied()
                .map(|r| r.iter() as &dyn Iterator<Item = u8>)
                .unwrap_or(zeros);
            //for block_pair in r0.zip(r1).collect::<Vec<(&u8, &u8)>>().chunks(BLOCKSIZE) {
            //    println!("{:?}", block_pair)
            //}
        }

        Playfield::new()
    }
}
