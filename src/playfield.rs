// TODO remove suppression for dead code warning
#![allow(dead_code, unused_variables)]

use super::contig::CartesianContigs;
use std::iter::Iterator;

/// turns separate iterators into iterator of pairs
struct PairwiseOrDefault<I> {
    i0: I,
    i1: Option<I>,
}

impl<I> PairwiseOrDefault<I> {
    fn from<'a, Outer, Inner, X>(
        rows: Outer,
    ) -> PairwiseOrDefault<<Inner as IntoIterator>::IntoIter>
    where
        Outer: IntoIterator<Item = Inner>,
        Inner: IntoIterator<Item = &'a X>,
        X: 'a,
    {
        let mut rows_iter = rows.into_iter();
        let i0 = rows_iter.by_ref().next().unwrap().into_iter();
        let i1 = rows_iter.by_ref().next().map(|i| i.into_iter());
        assert!(rows_iter.next().is_none(), "too many rows");

        PairwiseOrDefault { i0, i1 }
    }
}

impl<'a, I, X> Iterator for PairwiseOrDefault<I>
where
    I: Iterator<Item = &'a X>,
    X: 'a + Copy + Default,
{
    type Item = (X, X);

    fn next(&mut self) -> Option<(X, X)> {
        match &mut self.i1 {
            Some(i2) => match (self.i0.next(), i2.next()) {
                (Some(x1), Some(x2)) => Some((*x1, *x2)),
                (Some(x1), None) => Some((*x1, X::default())),
                (None, Some(x2)) => Some((X::default(), *x2)),
                (None, None) => None,
            },
            None => self.i0.next().map(|x1| (*x1, X::default())),
        }
    }
}

/// Array of bits, organised as double rows.
/// Each item represents two rows of bits of width `BLOCKSIZE`.
///
/// The reason for double rows is so that non-contiguous rows cannot both influence
/// new cell birth in the intermediate row, which would otherwise be possible.
pub struct Playfield {
    contigs: CartesianContigs<i32, u16>,
}

#[derive(Default)]
pub struct Coordinate<T>
where
    T: Default,
{
    pub x: T,
    pub y: T,
}

impl Playfield {
    fn new() -> Playfield {
        Playfield {
            contigs: CartesianContigs::new(),
        }
    }

    pub fn from(rows_of_bytes: &Vec<Vec<u8>>, origin: Coordinate<i32>) -> Playfield {
        use std::vec::IntoIter;

        let mut playfield = Playfield::new();

        for (y, chunk) in rows_of_bytes.chunks(2).enumerate() {
            for (x, p) in PairwiseOrDefault::<IntoIter<u8>>::from(chunk).enumerate() {
                let merged_pair = p.0 as u16 | (p.1 as u16) << 8;
                println!(
                    "pair ({}, {}) {:02x}{:02x}\n----------- {:04x}",
                    x, y, p.1, p.0, merged_pair
                );
                if merged_pair != 0 {
                    playfield
                        .contigs
                        .set(x as i32 + origin.x, y as i32 + origin.y, merged_pair);
                }
            }
        }

        playfield
    }

    pub fn to_rows_of_bytes(&self) -> (Vec<Vec<u8>>, Coordinate<i32>) {
        if self.contigs.is_empty() {
            (Vec::new(), Coordinate::default())
        } else {
            (Vec::new(), Coordinate::default())
            //let origin = Coordinate{ x: 0, y: self.contigs}
            //      (vec![vec![]], Coordinate { x: 0, y: 0 })
        }
    }
}

mod tests;
