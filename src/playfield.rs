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

    fn get(&self, i: Idx) -> Option<&T> {
        if i >= self.origin {
            self.items.get(Idx::to_usize(&(i - self.origin)).unwrap())
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

    // return the indexed item
    fn get(&self, i: Idx) -> Option<&T> {
        if let Ok(i_c) = self.contigs.binary_search_by(|c| c.cmp(&i)) {
            self.contigs[i_c].get(i)
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

/// a vertical block of contiguous rows
#[derive(Debug)]
struct Drop {
    bottom: i32,
    rows: VecDeque<OrderedContigs<i32, u8>>,
}

/// ordered list of drops, ordered by `bottom`, and coelesced opportunistically
type Playfield = VecDeque<Drop>;

mod tests;
