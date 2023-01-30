// TODO remove suppression for dead code warning
#![allow(dead_code)]

use super::contig::{Contig, ContigEnumerator, ContigNeighbourhoodEnumerator};
use super::multi_iterator::MultiIterator;
use super::neighbourhood::{Neighbourhood, N_SIZE};
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
    items: [[Option<T>; N_SIZE]; N_SIZE], // first index is row
}

impl<Idx, T> CartesianNeighbourhood<Idx, T> {
    fn new(
        i_row: Idx,
        i_col: Idx,
        nbhs: Vec<Option<Neighbourhood<Idx, T>>>,
    ) -> CartesianNeighbourhood<Idx, T>
    where
        T: Debug,
    {
        CartesianNeighbourhood {
            i_row,
            i_col,
            items: nbhs
                .into_iter()
                .map(|nbho| match nbho {
                    Some(nbh) => nbh.items,
                    None => [None, None, None],
                })
                .collect::<Vec<[Option<T>; N_SIZE]>>()
                .try_into()
                .unwrap(),
        }
    }
}

/// the MultiIterator instantiated for CarteisianContigNeighbourhoodEnumerator
type CartesianContigNeighborhoodMultiIterator<'a, Idx, T> =
    MultiIterator<Idx, ContigNeighbourhoodEnumerator<'a, Idx, T>, Neighbourhood<'a, Idx, &'a T>>;

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
    i_row: Option<Idx>,
    column_enumerator: CartesianContigNeighborhoodMultiIterator<'a, Idx, T>,
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
        let mut row_enumerator = c.0.neighbourhood_enumerator();
        let (i_row, column_enumerator) =
            CartesianContigNeighbourhoodEnumerator::get_next_row(&mut row_enumerator);

        CartesianContigNeighbourhoodEnumerator {
            row_enumerator,
            i_row,
            column_enumerator,
        }
    }

    fn get_next_row(
        row_enumerator: &mut ContigNeighbourhoodEnumerator<'a, Idx, Contig<Idx, T>>,
    ) -> (
        Option<Idx>,
        CartesianContigNeighborhoodMultiIterator<'a, Idx, T>,
    ) {
        let row = row_enumerator.next();
        let i_row = row.as_ref().map(|n| n.i);
        let column_enumerator =
            CartesianContigNeighbourhoodEnumerator::multi_iterator_for_row_neighbourhood(row);
        (i_row, column_enumerator)
    }

    fn multi_iterator_for_row_neighbourhood(
        row_nbh_o: Option<Neighbourhood<Idx, &'a Contig<Idx, T>>>,
    ) -> CartesianContigNeighborhoodMultiIterator<'a, Idx, T> {
        match row_nbh_o {
            Some(row_nbh) => {
                // if the focused row is present, that drives the enumerator,
                // otherwise whichever or both of the above/below rows
                let drivers = match row_nbh.items {
                    [_, Some(_), _] => vec![false, true, false],
                    [Some(_), None, Some(_)] => vec![true, false, true],
                    [Some(_), None, None] => vec![true, false, false],
                    [None, None, Some(_)] => vec![false, false, true],
                    [None, None, None] => vec![false, false, false],
                };

                let iterators = row_nbh
                    .items
                    .iter()
                    .map(|c_o| c_o.map(|c| c.neighbourhood_enumerator()))
                    .collect::<Vec<Option<ContigNeighbourhoodEnumerator<Idx, T>>>>();

                MultiIterator::new(iterators, drivers)
            }
            None => MultiIterator::new(vec![None, None, None], vec![false, false, false]),
        }
    }

    /// advance to the next non-empty row, if any
    fn advance_row(&mut self) {
        (self.i_row, self.column_enumerator) =
            CartesianContigNeighbourhoodEnumerator::get_next_row(&mut self.row_enumerator);
    }

    /// return next column if any
    fn next_col(&mut self) -> Option<CartesianNeighbourhood<Idx, &'a T>> {
        match self.column_enumerator.next() {
            Some((i_col, items)) => Some(CartesianNeighbourhood::new(
                self.i_row.unwrap(),
                i_col,
                items,
            )),
            None => None,
        }

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
