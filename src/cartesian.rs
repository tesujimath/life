// TODO remove suppression for dead code warning
#![allow(dead_code)]

use super::contig::{Contig, ContigEnumerator, ContigNeighbourhoodEnumerator};
use super::multi_iterator::MultiIterator;
use super::neighbourhood::Neighbourhood;
use num::cast::AsPrimitive;
use num::FromPrimitive;
use num::One;
use std::cmp::PartialOrd;
use std::fmt::Debug;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::SubAssign;

#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub struct Coordinate<T>
where
    T: Default + PartialEq + Eq,
{
    pub x: T,
    pub y: T,
}

/// nonempty 2D array of Contigs, organised in rows, or None
#[derive(Debug)]
pub struct CartesianContig<Idx, T>(Contig<Idx, Contig<Idx, T>>)
where
    Idx: Copy + Debug;

impl<Idx, T> CartesianContig<Idx, T>
where
    T: Debug,
    Idx: Copy
        + Default
        + One
        + FromPrimitive
        + AsPrimitive<usize>
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + Ord
        + AddAssign
        + SubAssign
        + Debug,
{
    /// create almost empty, with a single cell
    pub fn new(x: Idx, y: Idx, item: T) -> CartesianContig<Idx, T> {
        CartesianContig(Contig::new(y, Contig::new(x, item)))
    }

    pub fn get(&self, x: Idx, y: Idx) -> Option<&T> {
        self.0.get(y).and_then(|row| row.get(x))
    }

    pub fn set(&mut self, x: Idx, y: Idx, item: T) {
        match self.0.get_mut(y) {
            Some(row) => row.set(x, item),
            None => self.0.set(y, Contig::new(x, item)),
        }
    }

    pub fn origin(&self) -> Coordinate<Idx> {
        Coordinate {
            x: self
                .0
                .enumerator()
                .min_by(|(_, lhs), (_, rhs)| lhs.origin().cmp(&rhs.origin()))
                .unwrap()
                .1
                .origin(),
            y: self.0.origin(),
        }
    }

    pub fn rows_enumerator(&self) -> ContigEnumerator<Idx, Contig<Idx, T>> {
        self.0.enumerator()
    }

    pub fn neighbourhood_enumerator(&self) -> CartesianContigNeighbourhoodEnumerator<Idx, T> {
        CartesianContigNeighbourhoodEnumerator::new(self)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct CartesianNeighbourhood<Idx, T> {
    i_row: Idx,
    i_col: Idx,
    items: [[Option<T>; 3]; 3], // first index is row
}

pub struct CartesianContigNeighbourhoodEnumerator<'a, Idx, T>
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
        + Debug,
{
    row_enumerator: ContigNeighbourhoodEnumerator<'a, Idx, Contig<Idx, T>>,
    column_enumerator: Option<
        MultiIterator<
            Idx,
            ContigNeighbourhoodEnumerator<'a, Idx, T>,
            Neighbourhood<'a, Idx, &'a T>,
        >,
    >,
}

impl<'a, Idx, T> CartesianContigNeighbourhoodEnumerator<'a, Idx, T>
where
    T: Debug,
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
        + Debug,
{
    fn new(c: &'a CartesianContig<Idx, T>) -> CartesianContigNeighbourhoodEnumerator<'a, Idx, T> {
        let row_enumerator = c.0.neighbourhood_enumerator();
        let mut result = CartesianContigNeighbourhoodEnumerator {
            row_enumerator,
            column_enumerator: None,
        };

        result.advance_row();

        result
    }

    /// advance to the next non-empty row, if any
    fn advance_row(&mut self) {
        let mut row_nbh_o: Option<Neighbourhood<Idx, &Contig<Idx, T>>>;
        loop {
            row_nbh_o = self.row_enumerator.next();
            match row_nbh_o {
                Some(ref row_nbh) => {
                    if row_nbh.items[1].is_some() {
                        break;
                    }
                }
                None => break,
            }
        }

        if let Some(ref mut row_nbh) = row_nbh_o {
            // if the focused row is present, that drives the enumerator,
            // otherwise whichever or both of the above/below rows
            let drivers = match row_nbh.items {
                [_, Some(_), _] => vec![false, true, false],
                [Some(_), None, Some(_)] => vec![true, false, true],
                [Some(_), None, None] => vec![true, false, false],
                [None, None, Some(_)] => vec![false, false, true],
                [None, None, None] => vec![false, false, false],
            };

            println!("contig drivers for row {:?}: {:?}", row_nbh.i, drivers);

            let iterators = row_nbh
                .items
                .iter()
                .map(|c_o| c_o.map(|c| c.neighbourhood_enumerator()))
                .collect::<Vec<Option<ContigNeighbourhoodEnumerator<Idx, T>>>>();

            self.column_enumerator = Some(MultiIterator::new(iterators, drivers));
        }
    }

    /// return next column if any
    fn next_col(&mut self) -> Option<CartesianNeighbourhood<Idx, &'a T>> {
        None
        // self.column_enumerator
        //     .as_mut()
        //     .and_then(|rows| match &mut rows.items[1] {
        //         Some(ref mut row_1_e) => match row_1_e.next() {
        //             Some(row_1_nbh) => {
        //                 let i_col = row_1_nbh.i;
        //                 let absent: [Option<&'a T>; 3] = [None, None, None];
        //                 Some(CartesianNeighbourhood {
        //                     i_row: rows.i,
        //                     i_col,
        //                     items: [
        //                         rows.items[0].as_mut().map_or(absent, |c| c.get(i_col)),
        //                         row_1_nbh.items,
        //                         rows.items[2].as_mut().map_or(absent, |c| c.get(i_col)),
        //                     ],
        //                 })
        //             }
        //             None => None,
        //         },
        //         None => None,
        //     })
    }
}

impl<'a, Idx, T> Iterator for CartesianContigNeighbourhoodEnumerator<'a, Idx, T>
where
    T: Debug,
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
        + Debug,
{
    type Item = CartesianNeighbourhood<Idx, &'a T>;

    fn next(&mut self) -> Option<CartesianNeighbourhood<Idx, &'a T>> {
        self.next_col().or_else(|| {
            self.advance_row();
            self.next_col()
        })
    }
}

mod tests;
