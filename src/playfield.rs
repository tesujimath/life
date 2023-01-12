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

    /// provide a reference to the indexed item
    fn get(&self, i: Idx) -> Option<&T> {
        if i >= self.origin {
            self.items.get(Idx::to_usize(&(i - self.origin)).unwrap())
        } else {
            None
        }
    }

    /// provide a mutable reference to the indexed item
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

    // TODO maybe remove this?
    fn enumerate_from(&self, i: Idx) -> impl Iterator<Item = (Idx, &T)> {
        let enumerator = self.items.iter().enumerate().skip(if i > self.origin {
            Idx::to_usize(&(i - self.origin)).unwrap()
        } else {
            0
        });
        enumerator.map(|(u, item)| (self.origin + Idx::from_usize(u).unwrap(), item))
    }

    fn neighbourhood_iter(&self, from: Idx) -> ContigNeighbourhoodEnumerator<Idx, T> {
        ContigNeighbourhoodEnumerator::new(self, from)
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

struct Neighbourhood<'a, Idx, T> {
    i: Idx,
    left: Option<&'a T>,
    this: &'a T,
    right: Option<&'a T>,
}

struct ContigNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy,
{
    c: &'a Contig<Idx, T>,
    next_u: usize,
}

impl<'a, Idx, T> ContigNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy + FromPrimitive + ToPrimitive + Add<Output = Idx> + Sub<Output = Idx> + PartialOrd,
{
    fn new(c: &'a Contig<Idx, T>, from: Idx) -> ContigNeighbourhoodEnumerator<'a, Idx, T> {
        ContigNeighbourhoodEnumerator {
            c,
            next_u: if from < c.origin {
                0
            } else {
                Idx::to_usize(&(from - c.origin)).unwrap()
            },
        }
    }
}

impl<'a, Idx, T> Iterator for ContigNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy + FromPrimitive + ToPrimitive + Add<Output = Idx> + Sub<Output = Idx> + PartialOrd,
{
    type Item = Neighbourhood<'a, Idx, T>;

    fn next(&mut self) -> Option<Neighbourhood<'a, Idx, T>> {
        if self.next_u < self.c.items.len() {
            let i = self.c.origin + Idx::from_usize(self.next_u).unwrap();
            let left = if self.next_u > 0 {
                Some(&self.c.items[self.next_u - 1])
            } else {
                None
            };
            let this = &self.c.items[self.next_u];
            let right = self.c.items.get(self.next_u + 1);
            self.next_u += 1;

            Some(Neighbourhood {
                i,
                left,
                this,
                right,
            })
        } else {
            None
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
            Ok(u) => Set(u),
            Err(u) => {
                let c_left_o = if u > 0 {
                    self.contigs.get(u - 1)
                } else {
                    self.contigs.get(u)
                };

                match (c_left_o, self.contigs.get(u)) {
                    (Some(c_left), Some(c_i)) if c_i.adjoins_left(i) => {
                        if c_left.adjoins_right(i) {
                            PushFrontAndCoelesce(u)
                        } else {
                            PushFront(u)
                        }
                    }

                    (None, Some(c_i)) if c_i.adjoins_left(i) => PushFront(u),

                    (Some(c_left), _) if c_left.adjoins_right(i) => PushBack(u - 1),

                    (_, _) => Insert(u),
                }
            }
        }
    }

    /// provide a reference to the indexed item
    fn get(&self, i: Idx) -> Option<&T> {
        if let Ok(u) = self.contigs.binary_search_by(|c| c.cmp(&i)) {
            self.contigs[u].get(i)
        } else {
            None
        }
    }

    /// provide a mutable reference to the indexed item
    fn get_mut(&mut self, i: Idx) -> Option<&mut T> {
        if let Ok(u) = self.contigs.binary_search_by(|c| c.cmp(&i)) {
            self.contigs[u].get_mut(i)
        } else {
            None
        }
    }

    fn set(&mut self, i: Idx, item: T) {
        use OrderedContigsUpdate::*;

        match self.determine_update(i) {
            Set(u) => self.contigs[u][i] = item,

            PushFront(u) => self.contigs[u].push_front(item),

            PushFrontAndCoelesce(u) => {
                self.contigs[u].push_front(item);
                self.coelesce_left(u);
            }

            PushBack(u) => self.contigs[u].push_back(item),

            Insert(u) => self.contigs.insert(u, Contig::new(i, item)),
        }
    }

    fn coelesce_left(&mut self, u: usize) {
        if let Some(mut removed_c) = self.contigs.remove(u) {
            self.contigs[u - 1].append(&mut removed_c);
        }
    }

    fn neighbourhood_enumerator(&self, from: Idx) -> OrderedContigsNeighbourhoodEnumerator<Idx, T> {
        let next_u = match self.contigs.binary_search_by(|c| c.cmp(&from)) {
            Ok(u) => u,
            Err(u) => u,
        };

        OrderedContigsNeighbourhoodEnumerator::new(&self, next_u, from)
    }

    fn neighbourhood_enumerator_for(
        &self,
        u: usize,
        from: Idx,
    ) -> Option<ContigNeighbourhoodEnumerator<Idx, T>> {
        self.contigs.get(u).map(|c| c.neighbourhood_iter(from))
    }
}

struct OrderedContigsNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy,
{
    oc: &'a OrderedContigs<Idx, T>,
    next_u: usize,
    next_i: Idx,
    enumerator: Option<ContigNeighbourhoodEnumerator<'a, Idx, T>>,
}

impl<'a, Idx, T> OrderedContigsNeighbourhoodEnumerator<'a, Idx, T>
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
    fn new(
        oc: &'a OrderedContigs<Idx, T>,
        next_u: usize,
        next_i: Idx,
    ) -> OrderedContigsNeighbourhoodEnumerator<'a, Idx, T> {
        OrderedContigsNeighbourhoodEnumerator {
            oc,
            next_u,
            next_i,
            enumerator: oc.contigs.get(next_u).map(|c| c.neighbourhood_iter(next_i)),
        }
    }
}

impl<'a, Idx, T> Iterator for OrderedContigsNeighbourhoodEnumerator<'a, Idx, T>
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
    type Item = Neighbourhood<'a, Idx, T>;

    fn next(&mut self) -> Option<Neighbourhood<'a, Idx, T>> {
        match &mut self.enumerator {
            Some(e) => match e.next() {
                Some(nbh) => Some(nbh),
                None => {
                    self.next_u += 1;
                    self.enumerator = self
                        .oc
                        .neighbourhood_enumerator_for(self.next_u, self.next_i);

                    self.enumerator.as_mut().and_then(|e| e.next())
                }
            },

            None => None,
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
