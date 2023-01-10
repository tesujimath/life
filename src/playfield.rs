// TODO remove suppression for dead code warning
#![allow(dead_code)]

use num::FromPrimitive;
use num::One;
use num::ToPrimitive;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::ops::Add;
use std::ops::Sub;
use std::ops::SubAssign;

/// a trait like Ord<RHS>
/// see https://github.com/rust-lang/rfcs/issues/2511
trait Ord2<RHS> {
    fn cmp(&self, other: &RHS) -> Ordering;
}

/// a block of contiguous items
#[derive(Debug, Eq, PartialEq)]
struct Contig<I>
where
    I: Copy,
{
    /// position of leftmost item
    origin: I,
    items: VecDeque<u8>,
}

impl<I> Contig<I>
where
    I: Copy
        + One
        + FromPrimitive
        + ToPrimitive
        + Add<Output = I>
        + Sub<Output = I>
        + PartialOrd
        + SubAssign,
{
    fn new(i: I, item: u8) -> Contig<I> {
        Contig {
            origin: i,
            items: VecDeque::from(vec![item]),
        }
    }

    fn contains(&self, i: I) -> bool {
        i >= self.origin && i < self.origin + I::from_usize(self.items.len()).unwrap()
    }

    fn adjoins_left(&self, i: I) -> bool {
        i == self.origin - I::one()
    }

    fn adjoins_right(&self, i: I) -> bool {
        i == self.origin + I::from_usize(self.items.len()).unwrap()
    }

    // TODO implement via Index and IndexMut traits
    fn set(&mut self, i: I, item: u8) {
        assert!(self.contains(i));
        self.items[I::to_usize(&(i - self.origin)).unwrap()] = item;
    }

    fn push_front(&mut self, item: u8) {
        self.items.push_front(item);
        self.origin -= I::one();
    }

    fn push_back(&mut self, item: u8) {
        self.items.push_back(item);
    }

    fn append(&mut self, other: &mut Contig<I>) {
        self.items.append(&mut other.items);
    }
}

impl<I> Ord2<I> for Contig<I>
where
    I: Copy + FromPrimitive + Add<Output = I> + PartialOrd,
{
    fn cmp(&self, i: &I) -> Ordering {
        if *i < self.origin {
            Ordering::Greater
        } else if *i < self.origin + I::from_usize(self.items.len()).unwrap() {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }
}

/// an ordered list of contigs, ordered by `origin`, and coelesced opportunistically
#[derive(Debug, Eq, PartialEq)]
struct OrderedContigs<I>
where
    I: Copy,
{
    contigs: VecDeque<Contig<I>>, // TODO make generic
}

impl<I> OrderedContigs<I>
where
    I: Copy
        + One
        + FromPrimitive
        + ToPrimitive
        + Add<Output = I>
        + Sub<Output = I>
        + PartialOrd
        + SubAssign,
{
    fn new() -> OrderedContigs<I> {
        OrderedContigs {
            contigs: VecDeque::new(),
        }
    }

    fn set(&mut self, i: I, item: u8) {
        match self.contigs.binary_search_by(|c| c.cmp(&i)) {
            Ok(i_c) => {
                self.contigs[i_c].set(i, item);
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
    rows: VecDeque<OrderedContigs<i32>>,
}

/// ordered list of drops, ordered by `bottom`, and coelesced opportunistically
type Playfield = VecDeque<Drop>;

mod tests;
