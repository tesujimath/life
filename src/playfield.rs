// TODO remove suppression for dead code warning
#![allow(dead_code, unused_variables)]

use super::contig::{CartesianContigs, Coordinate};
use num::cast::AsPrimitive;
use num::FromPrimitive;
use num::One;
use num::Zero;
use std::cmp::PartialOrd;
use std::iter::Iterator;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::Shl;
use std::ops::Shr;
use std::ops::Sub;
use std::ops::SubAssign;

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
pub struct Playfield<Idx, T>
where
    Idx: Copy
        + Default
        + One
        + FromPrimitive
        + AsPrimitive<usize>
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + AddAssign
        + SubAssign,
{
    contigs: CartesianContigs<Idx, T>,
}

impl<Idx, T> Playfield<Idx, T>
where
    Idx: Copy
        + Default
        + One
        + FromPrimitive
        + AsPrimitive<usize>
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + AddAssign
        + SubAssign
        + Zero
        + Ord,
    T: Zero,
{
    fn new() -> Playfield<Idx, T> {
        Playfield {
            contigs: CartesianContigs::new(Idx::zero(), Idx::zero(), T::zero()),
        }
    }

    /// pack multiple bytes into playfield item - TODO make this work for any T
    fn pack(pair: (u8, u8)) -> T
    where
        T: FromPrimitive + Shl<u8> + BitOr<<T as Shl<u8>>::Output, Output = T>,
    {
        let b0 = T::from_u8(pair.0).unwrap();
        let b1 = T::from_u8(pair.1).unwrap();

        b0 | b1 << 8
    }

    /// unpack multiple bytes from playfield item - TODO make this work for any T
    fn unpack(packed: T) -> (u8, u8)
    where
        T: AsPrimitive<u16> + Shr<u8> + BitAnd<<T as Shr<u8>>::Output, Output = T>,
    {
        (
            (T::as_(packed) & 0xff) as u8,
            ((T::as_(packed) & 0xff00) >> 8) as u8,
        )
    }

    pub fn from(rows_of_bytes: &[Vec<u8>], origin: Coordinate<Idx>) -> Playfield<Idx, T>
    where
        T: FromPrimitive + Shl<u8> + BitOr<<T as Shl<u8>>::Output, Output = T> + std::fmt::LowerHex,
    {
        use std::vec::IntoIter;

        let mut playfield = Playfield::<Idx, T>::new();

        for (y_u, chunk) in rows_of_bytes.chunks(2).enumerate() {
            for (x_u, p) in PairwiseOrDefault::<IntoIter<u8>>::from(chunk).enumerate() {
                let merged_pair = Self::pack(p);
                println!(
                    "pair ({}, {}) {:02x}{:02x}\n----------- {:04x}",
                    x_u, y_u, p.1, p.0, merged_pair
                );
                if !T::is_zero(&merged_pair) {
                    let x = Idx::from_usize(x_u).unwrap() + origin.x;
                    let y = Idx::from_usize(y_u).unwrap() + origin.y;
                    playfield.contigs.set(x, y, merged_pair);
                }
            }
        }

        playfield
    }

    // space wasting conversion into packed vectors
    pub fn to_rows_of_bytes(&self) -> (Vec<Vec<u8>>, Coordinate<Idx>)
    where
        T: AsPrimitive<u16> + Shr<u8> + BitAnd<<T as Shr<u8>>::Output, Output = T> + Copy,
        Idx: FromPrimitive,
    {
        let origin = self.contigs.origin();
        let mut rows: Vec<Vec<u8>> = Vec::new();

        for (y, row) in self.contigs.rows_enumerator() {
            let mut lower_items = Vec::new();
            let mut upper_items = Vec::new();
            for (x, merged_item) in row.enumerator() {
                while origin.x < x - Idx::from_usize(lower_items.len()).unwrap() {
                    lower_items.push(0u8);
                    upper_items.push(0u8);
                }

                let (lower, upper) = Self::unpack(*merged_item);

                lower_items.push(lower);
                upper_items.push(upper);
            }
            rows.push(lower_items);
            rows.push(upper_items);
        }
        (rows, origin)
    }
}

mod tests;
