// TODO remove suppression for dead code warning
#![allow(dead_code, unused_variables)]

use super::contig::{CartesianContig, Coordinate};
use num::cast::AsPrimitive;
use num::FromPrimitive;
use num::One;
use num::Zero;
use std::cmp::PartialOrd;
use std::iter::Iterator;
use std::ops::Add;
use std::ops::AddAssign;
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
    cc: CartesianContig<Idx, T>,
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
            cc: CartesianContig::new(Idx::zero(), Idx::zero(), T::zero()),
        }
    }

    /// pack a pair of halfblocks into a block
    /// the halfblock type H must be half the size of the block type T
    fn pack<H>(pair: (H, H)) -> T {
        use std::mem::size_of;
        assert!(size_of::<H>() * 2 == size_of::<T>());

        let packed: T = T::zero();
        unsafe {
            let p = &packed as *const T as *mut H;
            *p = pair.0;
            *(p.offset(1)) = pair.1
        }
        packed
    }

    /// unpack a pair of halfblocks from a block
    /// the halfblock type H must be half the size of the block type T
    fn unpack<H>(packed: T) -> (H, H)
    where
        H: Zero + Copy,
    {
        use std::mem::size_of;
        assert!(size_of::<H>() * 2 == size_of::<T>());

        let p0: H;
        let p1: H;

        unsafe {
            let p = &packed as *const T as *mut H;
            p0 = *p;
            p1 = *(p.offset(1));
        }

        (p0, p1)
    }

    pub fn from_rows<H>(rows_of_bytes: &[Vec<H>], origin: Coordinate<Idx>) -> Playfield<Idx, T>
    where
        T: std::fmt::LowerHex,
        H: Copy + Default + std::fmt::LowerHex,
    {
        use std::vec::IntoIter;

        let mut playfield = Playfield::<Idx, T>::new();

        for (y_u, chunk) in rows_of_bytes.chunks(2).enumerate() {
            for (x_u, p) in PairwiseOrDefault::<IntoIter<H>>::from(chunk).enumerate() {
                let merged_pair = Self::pack::<H>(p);
                println!(
                    "pair ({}, {}) {:02x}{:02x}\n----------- {:04x}",
                    x_u, y_u, p.1, p.0, merged_pair
                );
                if !T::is_zero(&merged_pair) {
                    let x = Idx::from_usize(x_u).unwrap() + origin.x;
                    let y = Idx::from_usize(y_u).unwrap() + origin.y;
                    playfield.cc.set(x, y, merged_pair);
                }
            }
        }

        playfield
    }

    // space wasting conversion into packed vectors
    pub fn to_rows<H>(&self) -> (Vec<Vec<H>>, Coordinate<Idx>)
    where
        Idx: FromPrimitive,
        T: Copy,
        H: Zero + Copy,
    {
        let origin = self.cc.origin();
        let mut rows: Vec<Vec<H>> = Vec::new();

        for (y, row) in self.cc.rows_enumerator() {
            let mut lower_items = Vec::new();
            let mut upper_items = Vec::new();
            for (x, merged_item) in row.enumerator() {
                while origin.x < x - Idx::from_usize(lower_items.len()).unwrap() {
                    lower_items.push(H::zero());
                    upper_items.push(H::zero());
                }

                let (lower, upper) = Self::unpack::<H>(*merged_item);

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
