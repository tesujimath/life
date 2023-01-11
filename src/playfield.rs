// TODO remove suppression for dead code warning
#![allow(dead_code)]

use num::FromPrimitive;
use num::One;
use num::ToPrimitive;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::ops::Add;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Sub;
use std::ops::SubAssign;

/// a trait like Ord<RHS>
/// see https://github.com/rust-lang/rfcs/issues/2511
trait Ord2<RHS> {
    fn cmp(&self, other: &RHS) -> Ordering;
}

/// a block of contiguous items
#[derive(Debug, Eq, PartialEq)]
struct Contig<Idx, T>
where
    Idx: Copy,
{
    /// position of leftmost item
    origin: Idx,
    items: VecDeque<T>,
}

impl<Idx, T> Contig<Idx, T>
where
    Idx: Copy
        + One
        + FromPrimitive
        + ToPrimitive
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + SubAssign,
{
    fn new(i: Idx, item: T) -> Contig<Idx, T> {
        Contig {
            origin: i,
            items: VecDeque::from(vec![item]),
        }
    }

    // provide a reference to the indexed item
    fn get(&self, i: Idx) -> Option<&T> {
        if i >= self.origin {
            self.items.get(Idx::to_usize(&(i - self.origin)).unwrap())
        } else {
            None
        }
    }

    // provide a mutable reference to the indexed item
    fn get_mut(&mut self, i: Idx) -> Option<&mut T> {
        if i >= self.origin {
            self.items
                .get_mut(Idx::to_usize(&(i - self.origin)).unwrap())
        } else {
            None
        }
    }

    fn adjoins_left(&self, i: Idx) -> bool {
        i == self.origin - Idx::one()
    }

    fn adjoins_right(&self, i: Idx) -> bool {
        i == self.origin + Idx::from_usize(self.items.len()).unwrap()
    }

    fn push_front(&mut self, item: T) {
        self.items.push_front(item);
        self.origin -= Idx::one();
    }

    fn push_back(&mut self, item: T) {
        self.items.push_back(item);
    }

    fn append(&mut self, other: &mut Contig<Idx, T>) {
        self.items.append(&mut other.items);
    }

    fn enumerate_from(&self, i: Idx) -> impl Iterator<Item = (Idx, &T)> {
        let enumerator = self.items.iter().enumerate().skip(if i > self.origin {
            Idx::to_usize(&(i - self.origin)).unwrap()
        } else {
            0
        });
        enumerator.map(|(u, item)| (self.origin + Idx::from_usize(u).unwrap(), item))
    }
}

impl<Idx, T> Index<Idx> for Contig<Idx, T>
where
    Idx: Copy
        + One
        + FromPrimitive
        + ToPrimitive
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + SubAssign,
{
    type Output = T;

    fn index(&self, i: Idx) -> &Self::Output {
        &self.items[Idx::to_usize(&(i - self.origin)).unwrap()]
    }
}

impl<Idx, T> IndexMut<Idx> for Contig<Idx, T>
where
    Idx: Copy
        + One
        + FromPrimitive
        + ToPrimitive
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + SubAssign,
{
    fn index_mut(&mut self, i: Idx) -> &mut Self::Output {
        &mut self.items[Idx::to_usize(&(i - self.origin)).unwrap()]
    }
}

impl<Idx, T> Ord2<Idx> for Contig<Idx, T>
where
    Idx: Copy + FromPrimitive + Add<Output = Idx> + PartialOrd,
{
    fn cmp(&self, i: &Idx) -> Ordering {
        if *i < self.origin {
            Ordering::Greater
        } else if *i < self.origin + Idx::from_usize(self.items.len()).unwrap() {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }
}

/// an ordered list of contigs, ordered by `origin`, and coelesced opportunistically
#[derive(Debug, Eq, PartialEq)]
struct OrderedContigs<Idx, T>
where
    Idx: Copy,
{
    contigs: VecDeque<Contig<Idx, T>>,
}

enum OrderedContigsUpdate {
    Set(usize),
    PushFront(usize),
    PushFrontAndCoelesce(usize),
    PushBack(usize),
    Insert(usize),
}

impl<Idx, T> OrderedContigs<Idx, T>
where
    Idx: Copy
        + One
        + FromPrimitive
        + ToPrimitive
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + SubAssign,
{
    fn new() -> OrderedContigs<Idx, T> {
        OrderedContigs {
            contigs: VecDeque::new(),
        }
    }

    fn determine_update(&self, i: Idx) -> OrderedContigsUpdate {
        use OrderedContigsUpdate::*;

        match self.contigs.binary_search_by(|c| c.cmp(&i)) {
            Ok(i_c) => Set(i_c),
            Err(i_c) => {
                let c_left_o = if i_c > 0 {
                    self.contigs.get(i_c - 1)
                } else {
                    self.contigs.get(i_c)
                };

                match (c_left_o, self.contigs.get(i_c)) {
                    (Some(c_left), Some(c_i)) if c_i.adjoins_left(i) => {
                        if c_left.adjoins_right(i) {
                            PushFrontAndCoelesce(i_c)
                        } else {
                            PushFront(i_c)
                        }
                    }

                    (None, Some(c_i)) if c_i.adjoins_left(i) => PushFront(i_c),

                    (Some(c_left), _) if c_left.adjoins_right(i) => PushBack(i_c - 1),

                    (_, _) => Insert(i_c),
                }
            }
        }
    }

    // provide a reference to the indexed item
    fn get(&self, i: Idx) -> Option<&T> {
        if let Ok(i_c) = self.contigs.binary_search_by(|c| c.cmp(&i)) {
            self.contigs[i_c].get(i)
        } else {
            None
        }
    }

    // provide a mutable reference to the indexed item
    fn get_mut(&mut self, i: Idx) -> Option<&mut T> {
        if let Ok(i_c) = self.contigs.binary_search_by(|c| c.cmp(&i)) {
            self.contigs[i_c].get_mut(i)
        } else {
            None
        }
    }

    fn set(&mut self, i: Idx, item: T) {
        use OrderedContigsUpdate::*;

        match self.determine_update(i) {
            Set(i_c) => self.contigs[i_c][i] = item,

            PushFront(i_c) => self.contigs[i_c].push_front(item),

            PushFrontAndCoelesce(i_c) => {
                self.contigs[i_c].push_front(item);
                self.coelesce_left(i_c);
            }

            PushBack(i_c) => self.contigs[i_c].push_back(item),

            Insert(i_c) => self.contigs.insert(i_c, Contig::new(i, item)),
        }
    }

    fn coelesce_left(&mut self, i_c: usize) {
        if let Some(mut removed_c) = self.contigs.remove(i_c) {
            self.contigs[i_c - 1].append(&mut removed_c);
        }
    }
}

// 2D array of Contigs, organised in rows
struct CartesianContigs<Idx, T>(OrderedContigs<Idx, OrderedContigs<Idx, T>>)
where
    Idx: Copy;

impl<Idx, T> CartesianContigs<Idx, T>
where
    Idx: Copy
        + One
        + FromPrimitive
        + ToPrimitive
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + SubAssign,
{
    fn new() -> CartesianContigs<Idx, T> {
        CartesianContigs(OrderedContigs::new())
    }

    fn get(&self, x: Idx, y: Idx) -> Option<&T> {
        self.0.get(y).and_then(|c| c.get(x))
    }

    fn set(&mut self, x: Idx, y: Idx, item: T) {
        match self.0.get_mut(y) {
            Some(c) => {
                c.set(x, item);
            }

            None => {
                let mut row = OrderedContigs::new();
                row.set(x, item);
                self.0.set(y, row);
            }
        }
    }
}

mod tests;
