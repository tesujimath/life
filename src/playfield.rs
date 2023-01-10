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
    T: Copy,
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
    T: Copy,
{
    fn new(i: Idx, item: T) -> Contig<Idx, T> {
        Contig {
            origin: i,
            items: VecDeque::from(vec![item]),
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
    T: Copy,
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
    T: Copy,
{
    fn index_mut(&mut self, i: Idx) -> &mut Self::Output {
        &mut self.items[Idx::to_usize(&(i - self.origin)).unwrap()]
    }
}

impl<Idx, T> Ord2<Idx> for Contig<Idx, T>
where
    Idx: Copy + FromPrimitive + Add<Output = Idx> + PartialOrd,
    T: Copy,
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
    T: Copy,
{
    contigs: VecDeque<Contig<Idx, T>>,
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
    T: Copy,
{
    fn new() -> OrderedContigs<Idx, T> {
        OrderedContigs {
            contigs: VecDeque::new(),
        }
    }

    fn set(&mut self, i: Idx, item: T) {
        match self.contigs.binary_search_by(|c| c.cmp(&i)) {
            Ok(i_c) => {
                self.contigs[i_c][i] = item;
            }
            Err(i_c) => {
                if i_c < self.contigs.len() {
                    let c_i = &mut self.contigs[i_c];
                    if c_i.adjoins_left(i) {
                        c_i.push_front(item);

                        if i_c > 0 && self.contigs[i_c - 1].adjoins_right(i) {
                            self.coelesce_left(i_c);
                        }
                    } else if i_c > 0 {
                        let c_left = &mut self.contigs[i_c - 1];
                        if c_left.adjoins_right(i) {
                            c_left.push_back(item);
                        } else {
                            self.contigs.insert(i_c, Contig::new(i, item));
                        }
                    } else {
                        self.contigs.push_front(Contig::new(i, item));
                    }
                } else {
                    match self.contigs.back_mut() {
                        Some(c_left) => {
                            // TODO extract out common code with above
                            if c_left.adjoins_right(i) {
                                c_left.push_back(item);
                            } else {
                                self.contigs.insert(i_c, Contig::new(i, item));
                            }
                        }
                        None => self.contigs.push_back(Contig::new(i, item)),
                    }
                }
            }
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
