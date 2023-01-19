// TODO remove suppression for dead code warning
#![allow(dead_code)]

use num::FromPrimitive;
use num::One;
use num::ToPrimitive;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Sub;
use std::ops::SubAssign;

/// a trait like Ord<RHS>
/// see [Rust RFC issue 2511](https://github.com/rust-lang/rfcs/issues/2511)
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

    /// return whether `i` is contained
    fn contains(&self, i: Idx) -> bool {
        i >= self.origin && i < self.origin + Idx::from_usize(self.items.len()).unwrap()
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

    /// get the neighbourhood for `i`, which must be in range
    fn get_neighbourhood(&self, i: Idx) -> Neighbourhood<Idx, T> {
        let u = Idx::to_usize(&(i - self.origin)).unwrap();
        let left = if u > 0 {
            Some(&self.items[u - 1])
        } else {
            None
        };
        let this = &self.items[u];
        let right = self.items.get(u + 1);

        Neighbourhood {
            i,
            left,
            this,
            right,
        }
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

#[derive(PartialEq, Debug)]
struct Neighbourhood<'a, Idx, T> {
    i: Idx,
    left: Option<&'a T>,
    this: &'a T,
    right: Option<&'a T>,
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
        + Default
        + FromPrimitive
        + ToPrimitive
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + AddAssign
        + SubAssign,
{
    fn new() -> OrderedContigs<Idx, T> {
        OrderedContigs {
            contigs: VecDeque::new(),
        }
    }

    fn from<I>(it: I) -> OrderedContigs<Idx, T>
    where
        I: IntoIterator<Item = (Idx, T)>,
    {
        let oc0 = OrderedContigs::new();
        it.into_iter().fold(oc0, |mut oc, (i, item)| {
            oc.set(i, item);
            oc
        })
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

    fn neighbourhood_enumerator(&self) -> OrderedContigsNeighbourhoodEnumerator<Idx, T> {
        let next_i = if !self.contigs.is_empty() {
            self.contigs[0].origin
        } else {
            Idx::default()
        };

        OrderedContigsNeighbourhoodEnumerator::new(self, 0, next_i)
    }

    fn find(&self, i: Idx) -> (usize, Idx) {
        match self.contigs.binary_search_by(|c| c.cmp(&i)) {
            Ok(u) => (u, i),
            Err(u) => {
                if u < self.contigs.len() {
                    (u, self.contigs[u].origin)
                } else {
                    (u, i)
                }
            }
        }
    }

    fn neighbourhood_enumerator_from(
        &self,
        i: Idx,
    ) -> OrderedContigsNeighbourhoodEnumerator<Idx, T> {
        let (next_u, next_i) = self.find(i);

        OrderedContigsNeighbourhoodEnumerator::new(self, next_u, next_i)
    }
}

struct OrderedContigsNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy,
{
    oc: &'a OrderedContigs<Idx, T>,
    u_c: usize,
    next_i: Idx,
}

impl<'a, Idx, T> OrderedContigsNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy
        + Default
        + One
        + FromPrimitive
        + ToPrimitive
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + AddAssign
        + SubAssign,
{
    fn new(
        oc: &'a OrderedContigs<Idx, T>,
        u_c: usize,
        next_i: Idx,
    ) -> OrderedContigsNeighbourhoodEnumerator<'a, Idx, T> {
        OrderedContigsNeighbourhoodEnumerator { oc, u_c, next_i }
    }

    /// advance the enumerator
    fn advance(&mut self) {
        self.next_i += Idx::one();
        if !self.oc.contigs[self.u_c].contains(self.next_i) {
            self.u_c += 1;
            if self.u_c < self.oc.contigs.len() {
                self.next_i = self.oc.contigs[self.u_c].origin;
            }
        }
    }

    /// return the neighbourhood for `i` or None,
    /// positioning the iterator after the returned item, which may be backwards
    fn get(&mut self, i: Idx) -> Option<Neighbourhood<Idx, T>> {
        if i < self.next_i {
            (self.u_c, self.next_i) = self.oc.find(i);
        }

        // skip contig
        while self.u_c < self.oc.contigs.len()
            && self.oc.contigs[self.u_c].cmp(&i) == Ordering::Less
        {
            self.u_c += 1;
        }

        if self.u_c < self.oc.contigs.len() {
            let c = &self.oc.contigs[self.u_c];
            if c.contains(i) {
                let nbh = c.get_neighbourhood(i);
                self.next_i = i;
                self.advance();
                Some(nbh)
            } else {
                self.next_i = c.origin;
                None
            }
        } else {
            None
        }
    }
}

impl<'a, Idx, T> Iterator for OrderedContigsNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy
        + Default
        + One
        + FromPrimitive
        + ToPrimitive
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + AddAssign
        + SubAssign,
{
    type Item = Neighbourhood<'a, Idx, T>;

    fn next(&mut self) -> Option<Neighbourhood<'a, Idx, T>> {
        if self.u_c < self.oc.contigs.len() {
            let nbh = self.oc.contigs[self.u_c].get_neighbourhood(self.next_i);
            self.advance();
            Some(nbh)
        } else {
            None
        }
    }
}

/// nonempty 2D array of Contigs, organised in rows, or None
pub struct CartesianContigs<Idx, T>(Option<OrderedContigs<Idx, OrderedContigs<Idx, T>>>)
where
    Idx: Copy;

impl<Idx, T> CartesianContigs<Idx, T>
where
    Idx: Copy
        + Default
        + One
        + FromPrimitive
        + ToPrimitive
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + AddAssign
        + SubAssign,
{
    pub fn new() -> CartesianContigs<Idx, T> {
        CartesianContigs(None)
    }

    pub fn is_empty(&self) -> bool {
        (&self.0).is_some()
    }

    pub fn get(&self, x: Idx, y: Idx) -> Option<&T> {
        match &self.0 {
            None => None,
            Some(rows) => match rows.get(y) {
                None => None,
                Some(row) => row.get(x),
            },
        }
    }

    // TODO rewrite get() using and_then
    //pub fn get2(&self, x: Idx, y: Idx) -> Option<&T> {
    //    (&self.0).and_then(|ref rows| rows.get(y).and_then(|row| row.get(x)))
    //}

    pub fn set(&mut self, x: Idx, y: Idx, item: T) {
        let rows = match self.0 {
            Some(ref mut rows) => rows,
            None => self.0.insert(OrderedContigs::new()),
        };
        let row = match rows.get_mut(y) {
            Some(row) => row,
            None => {
                let row = OrderedContigs::new();
                rows.set(y, row);
                // TODO should set() return a &mut to avoid this additional query?
                rows.get_mut(y).unwrap()
            }
        };
        row.set(x, item);
    }
}

mod tests;
