// TODO remove suppression for dead code warning
#![allow(dead_code)]

use std::convert::identity;
use std::{iter::Peekable, marker::PhantomData};

use super::indexed::Indexed;
use super::seekable::SeekableIterator;

pub const N_SIZE: usize = 3;

/// an item and its siblings, any of which may be missing
#[derive(Eq, PartialEq, Debug)]
pub struct Neighbourhood<Idx, T> {
    pub i: Idx,
    pub items: [Option<T>; N_SIZE],
}

pub struct NeighbourhoodIterator<Idx, T, S>
where
    T: SeekableIterator<Idx, S>,
    S: Indexed<Idx>,
{
    n: Neighbourhood<Idx, Peekable<T>>,
    drivers: [bool; N_SIZE],
    phantom: PhantomData<S>,
}

impl<Idx, T, S> NeighbourhoodIterator<Idx, T, S>
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
        Neighbourhood {
            i: n.i,
            items: n.items.map(|it_o| it_o.map(|it| it.peekable())),
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
    fn consume_next(&mut self, i: Idx) -> Option<(Idx, Neighbourhood<Idx, S>)> {
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

impl<Idx, T, S> Iterator for NeighbourhoodIterator<Idx, T, S>
where
    T: SeekableIterator<Idx, S>,
    S: Indexed<Idx>,
    Idx: Copy + PartialOrd,
{
    type Item = (Idx, Neighbourhood<Idx, S>);

    fn next(&mut self) -> Option<(Idx, Neighbourhood<Idx, S>)> {
        self.determine_next().and_then(|i| self.consume_next(i))
    }
}
