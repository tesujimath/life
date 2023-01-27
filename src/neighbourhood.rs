// TODO remove suppression for dead code warning
#![allow(dead_code)]

use std::marker::PhantomData;

use super::indexed::Indexed;
use super::seekable::SeekableIterator;

pub const N_SIZE: usize = 3;

/// An item and its siblings, any of which may be missing.
///
/// # Intended Usage
///
/// Neighbourhoods may wrap value or reference types, and are expected  to be used as
/// iterator items.  In this they are unusual, since the neighbourhoods are created during
/// iteration, so cannot themselves be references, but may contain references to items in
/// the underlying collection.
///
/// The phantom lifetime addresses exactly this, exposing the lifetime of any reference in `T`
/// into the neighbourhood itself.
#[derive(Eq, PartialEq, Debug)]
pub struct Neighbourhood<'a, Idx, T> {
    pub i: Idx,
    pub items: [Option<T>; N_SIZE],
    phantom: PhantomData<&'a ()>,
}

impl<'a, Idx, T> Neighbourhood<'a, Idx, T> {
    pub fn new(i: Idx, items: [Option<T>; N_SIZE]) -> Neighbourhood<'a, Idx, T> {
        Neighbourhood {
            i,
            items,
            phantom: PhantomData,
        }
    }

    pub fn empty(i: Idx) -> Neighbourhood<'a, Idx, T> {
        Neighbourhood {
            i,
            items: [None, None, None], // to shortcut as [None; N_SIZE] would require T: Copy
            phantom: PhantomData,
        }
    }
}

impl<'a, Idx, T> Indexed<Idx> for Neighbourhood<'a, Idx, T>
where
    Idx: Copy,
{
    fn index(&self) -> Idx {
        self.i
    }
}

/// An iterator over a neighbourhood of seekable iterators.
// TODO this isn't neighbourhood specific, surely, just any collection of iterators?
pub struct NeighbourhoodIterator<'a, Idx, T, S>
where
    T: SeekableIterator<Idx, S>,
    S: Indexed<Idx>,
{
    n: Neighbourhood<'a, Idx, T>,
    drivers: [bool; N_SIZE],
    phantom: PhantomData<S>,
}

impl<'a, Idx, T, S> NeighbourhoodIterator<'a, Idx, T, S>
where
    T: SeekableIterator<Idx, S>,
    S: Indexed<Idx>,
    Idx: Copy + PartialOrd,
{
    pub fn new(
        n: Neighbourhood<Idx, T>,
        drivers: [bool; N_SIZE],
    ) -> NeighbourhoodIterator<Idx, T, S> {
        NeighbourhoodIterator {
            n,
            drivers,
            phantom: PhantomData,
        }
    }

    /// return index of next item
    fn determine_next(&mut self) -> Option<Idx> {
        let mut min_o: Option<Idx> = None;
        for (u, driver) in self.drivers.iter().enumerate() {
            if *driver {
                let next_o = self.n.items[u]
                    .as_mut()
                    .and_then(|ref mut it| it.peek().map(|s| s.index()));

                match (min_o, next_o) {
                    (None, _) => {
                        min_o = next_o;
                    }
                    (Some(min), Some(next)) if next < min => min_o = next_o,
                    _ => (),
                }
            }
        }

        min_o
    }

    /// consume the next item
    fn consume_next(&mut self, i: Idx) -> Option<(Idx, Neighbourhood<'a, Idx, &'a T>)> {
        let mut items: Vec<Option<S>> = Vec::with_capacity(N_SIZE);
        for p_o in self.n.items.iter_mut() {
            let item = if let Some(p) = p_o { p.seek(i) } else { None };
            items.push(item);
        }

        None
    }
}

impl<'a, Idx, T, S> Iterator for NeighbourhoodIterator<'a, Idx, T, S>
where
    T: SeekableIterator<Idx, S> + 'a,
    S: Indexed<Idx>,
    Idx: Copy + PartialOrd,
{
    type Item = (Idx, Neighbourhood<'a, Idx, &'a T>);

    fn next(&mut self) -> Option<(Idx, Neighbourhood<'a, Idx, &'a T>)> {
        match self.determine_next() {
            Some(i) => self.consume_next(i),
            None => None,
        }
    }
}
