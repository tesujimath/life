// TODO remove suppression for dead code warning
#![allow(dead_code)]

use std::marker::PhantomData;

use super::seekable::SeekableIterator;

pub const NBH_SIZE: usize = 3;

/// an item and its siblings, any of which may be missing
#[derive(Eq, PartialEq, Debug)]
pub struct Neighbourhood<Idx, T> {
    pub i: Idx,
    pub items: [Option<T>; NBH_SIZE],
}

pub struct NeighbourhoodIterator<Idx, T, S> {
    sis: Neighbourhood<Idx, T>,
    drivers: [bool; NBH_SIZE],
    phantom: PhantomData<S>,
}

impl<Idx, T, S> NeighbourhoodIterator<Idx, T, S>
where
    T: SeekableIterator<Idx, S>,
{
    fn new(
        sis: Neighbourhood<Idx, T>,
        drivers: [bool; NBH_SIZE],
    ) -> NeighbourhoodIterator<Idx, T, S> {
        NeighbourhoodIterator {
            sis,
            drivers,
            phantom: PhantomData,
        }
    }
}

impl<Idx, T, S> Iterator for NeighbourhoodIterator<Idx, T, S>
where
    T: SeekableIterator<Idx, S>,
{
    type Item = (Idx, Neighbourhood<Idx, S>);

    fn next(&mut self) -> Option<(Idx, Neighbourhood<Idx, S>)> {
        None
    }
}
