// TODO remove suppression for dead code warning
#![allow(dead_code)]

use super::multi_iterator::Indexed;
use std::marker::PhantomData;

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
#[derive(Debug)]
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

impl<'a, Idx, T> PartialEq for Neighbourhood<'a, Idx, T>
where
    Idx: PartialEq,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.i == other.i && self.items == other.items
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
