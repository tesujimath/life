// TODO remove suppression for dead code warning
#![allow(dead_code)]

use std::{iter::Peekable, marker::PhantomData};

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
}

/// An iterator over a neighbourhood of seekable iterators.
// TODO this isn't neighbourhood specific, surely, just any collection of iterators?
pub struct NeighbourhoodIterator<'a, Idx, T, S>
where
    T: SeekableIterator<Idx, S>,
    S: Indexed<Idx>,
{
    n: Neighbourhood<'a, Idx, Peekable<T>>,
    drivers: [bool; N_SIZE],
    phantom: PhantomData<S>,
}

impl<'a, Idx, T, S> NeighbourhoodIterator<'a, Idx, T, S>
where
    T: SeekableIterator<Idx, S>,
    S: Indexed<Idx>,
    Idx: Copy + PartialOrd,
{
    fn new(n: Neighbourhood<Idx, T>, drivers: [bool; N_SIZE]) -> NeighbourhoodIterator<Idx, T, S> {
        NeighbourhoodIterator {
            n: Self::make_peekable(n),
            drivers,
            phantom: PhantomData,
        }
    }

    fn make_peekable(n: Neighbourhood<Idx, T>) -> Neighbourhood<Idx, Peekable<T>> {
        Neighbourhood::new(n.i, n.items.map(|it_o| it_o.map(|it| it.peekable())))
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
    fn consume_next(&mut self, i: Idx) -> Option<(Idx, Neighbourhood<'a, Idx, S>)> {
        let providers = self
            .n
            .items
            .iter()
            .enumerate()
            .filter_map(|(u, p_o)| match p_o {
                Some(p) => Some((u, p)),
                None => None,
            });
        // for (u, provider) in providers {
        //     provider.peek().and_then(|x| x)
        // }

        None
    }
}

impl<'a, Idx, T, S> Iterator for NeighbourhoodIterator<'a, Idx, T, S>
where
    T: SeekableIterator<Idx, S>,
    S: Indexed<Idx>,
    Idx: Copy + PartialOrd,
{
    type Item = (Idx, Neighbourhood<'a, Idx, S>);

    fn next(&mut self) -> Option<(Idx, Neighbourhood<'a, Idx, S>)> {
        match self.determine_next() {
            Some(i) => {
                let result = self.consume_next(i);
                result
            }
            None => None,
        }
    }
}
