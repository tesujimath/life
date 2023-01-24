// TODO remove suppression for dead code warning
#![allow(dead_code)]

use num::cast::AsPrimitive;
use num::FromPrimitive;
use num::One;
use std::cmp::Ordering;
use std::cmp::PartialOrd;
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
        + AsPrimitive<usize>
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
            self.items.get(Idx::as_(i - self.origin))
        } else {
            None
        }
    }

    /// provide a mutable reference to the indexed item
    fn get_mut(&mut self, i: Idx) -> Option<&mut T> {
        if i >= self.origin {
            self.items.get_mut(Idx::as_(i - self.origin))
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
    fn get_neighbourhood(&self, i: Idx) -> Neighbourhood<Idx, &T> {
        let u = Idx::as_(i - self.origin);
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
        + AsPrimitive<usize>
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + SubAssign,
{
    type Output = T;

    fn index(&self, i: Idx) -> &Self::Output {
        &self.items[Idx::as_(i - self.origin)]
    }
}

impl<Idx, T> IndexMut<Idx> for Contig<Idx, T>
where
    Idx: Copy
        + One
        + FromPrimitive
        + AsPrimitive<usize>
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + SubAssign,
{
    fn index_mut(&mut self, i: Idx) -> &mut Self::Output {
        &mut self.items[Idx::as_(i - self.origin)]
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

#[derive(Eq, PartialEq, Debug)]
pub struct Neighbourhood<Idx, T> {
    i: Idx,
    left: Option<T>,
    this: T,
    right: Option<T>,
}

/// an ordered list of contigs, ordered by `origin`, and coelesced opportunistically
#[derive(Debug, Eq, PartialEq)]
pub struct Contigs<Idx, T>
where
    Idx: Copy,
{
    contigs: VecDeque<Contig<Idx, T>>,
}

enum ContigsUpdate {
    Set(usize),
    PushFront(usize),
    PushFrontAndCoelesce(usize),
    PushBack(usize),
    Insert(usize),
}

impl<Idx, T> Contigs<Idx, T>
where
    Idx: Copy
        + One
        + Default
        + FromPrimitive
        + AsPrimitive<usize>
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + AddAssign
        + SubAssign,
{
    fn new(i: Idx, item: T) -> Contigs<Idx, T> {
        let c = Contig::new(i, item);
        let mut contigs = VecDeque::new();
        contigs.push_front(c);
        Contigs { contigs }
    }

    fn from<I>(into_it: I) -> Option<Contigs<Idx, T>>
    where
        I: IntoIterator<Item = (Idx, T)>,
    {
        let mut it = into_it.into_iter();

        match it.by_ref().next() {
            Some((i, item)) => {
                let oc0 = Contigs::new(i, item);
                Some(it.fold(oc0, |mut oc, (i, item)| {
                    oc.set(i, item);
                    oc
                }))
            }
            None => None,
        }
    }

    fn origin(&self) -> Idx {
        self.contigs[0].origin
    }

    fn determine_update(&self, i: Idx) -> ContigsUpdate {
        use ContigsUpdate::*;

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

    /// provide a reference to the indexed item, if any
    fn get_in_left(&self, u: usize, i: Idx) -> Option<&T> {
        if u > 0 {
            self.contigs[u - 1].get(i)
        } else {
            None
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
        use ContigsUpdate::*;

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

    pub fn enumerator(&self) -> ContigsEnumerator<Idx, T> {
        ContigsEnumerator::new(self)
    }

    pub fn neighbourhood_enumerator(&self) -> ContigsNeighbourhoodEnumerator<Idx, T> {
        let next_i = self.contigs[0].origin;

        ContigsNeighbourhoodEnumerator::new(self, 0, next_i)
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

    pub fn neighbourhood_enumerator_from(&self, i: Idx) -> ContigsNeighbourhoodEnumerator<Idx, T> {
        let (next_u, next_i) = self.find(i);

        ContigsNeighbourhoodEnumerator::new(self, next_u, next_i)
    }
}

pub struct ContigsNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy,
{
    oc: &'a Contigs<Idx, T>,
    u_c: usize,
    next_i: Idx,
}

impl<'a, Idx, T> ContigsNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy
        + Default
        + One
        + FromPrimitive
        + AsPrimitive<usize>
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + AddAssign
        + SubAssign,
{
    fn new(
        oc: &'a Contigs<Idx, T>,
        u_c: usize,
        next_i: Idx,
    ) -> ContigsNeighbourhoodEnumerator<'a, Idx, T> {
        ContigsNeighbourhoodEnumerator { oc, u_c, next_i }
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
    fn get(&mut self, i: Idx) -> (Option<&'a T>, Option<&'a T>, Option<&'a T>) {
        let i_left = i - Idx::from_usize(1).unwrap();
        let i_right = i + Idx::from_usize(1).unwrap();

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
            let item_this = c.get(i);
            let item_left = match item_this {
                Some(_) => c.get(i_left),
                // this is the tricky case where we have to look in the contig to the left, if any:
                None => c
                    .get(i_left)
                    .or_else(|| self.oc.get_in_left(self.u_c, i_left)),
            };
            let item_right = c.get(i_right);
            (item_left, item_this, item_right)
        } else {
            // again, we need to look in the contig to the left, if any
            let item_left = self.oc.get_in_left(self.u_c, i_left);
            (item_left, None, None)
        }
    }
}

impl<'a, Idx, T> Iterator for ContigsNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy
        + Default
        + One
        + FromPrimitive
        + AsPrimitive<usize>
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + AddAssign
        + SubAssign,
{
    type Item = Neighbourhood<Idx, &'a T>;

    fn next(&mut self) -> Option<Neighbourhood<Idx, &'a T>> {
        if self.u_c < self.oc.contigs.len() {
            let nbh = self.oc.contigs[self.u_c].get_neighbourhood(self.next_i);
            self.advance();
            Some(nbh)
        } else {
            None
        }
    }
}

/// simple enumerator without the neighbourhood
pub struct ContigsEnumerator<'a, Idx, T>(ContigsNeighbourhoodEnumerator<'a, Idx, T>)
where
    Idx: Copy;

impl<'a, Idx, T> ContigsEnumerator<'a, Idx, T>
where
    Idx: Copy
        + Default
        + One
        + FromPrimitive
        + AsPrimitive<usize>
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + AddAssign
        + SubAssign,
{
    fn new(oc: &'a Contigs<Idx, T>) -> ContigsEnumerator<'a, Idx, T> {
        ContigsEnumerator(oc.neighbourhood_enumerator())
    }
}

impl<'a, Idx, T> Iterator for ContigsEnumerator<'a, Idx, T>
where
    Idx: Copy
        + Default
        + One
        + FromPrimitive
        + AsPrimitive<usize>
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + AddAssign
        + SubAssign,
{
    type Item = (Idx, &'a T);

    fn next(&mut self) -> Option<(Idx, &'a T)> {
        self.0.next().map(|ref nbh| (nbh.i, nbh.this))
    }
}

#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub struct Coordinate<T>
where
    T: Default + PartialEq + Eq,
{
    pub x: T,
    pub y: T,
}

/// nonempty 2D array of Contigs, organised in rows, or None
#[derive(Debug)]
pub struct CartesianContigs<Idx, T>(Contigs<Idx, Contigs<Idx, T>>)
where
    Idx: Copy;

impl<Idx, T> CartesianContigs<Idx, T>
where
    Idx: Copy
        + Default
        + One
        + FromPrimitive
        + AsPrimitive<usize>
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + Ord
        + AddAssign
        + SubAssign,
{
    /// create almost empty, with a single cell
    pub fn new(x: Idx, y: Idx, item: T) -> CartesianContigs<Idx, T> {
        CartesianContigs(Contigs::new(y, Contigs::new(x, item)))
    }

    pub fn get(&self, x: Idx, y: Idx) -> Option<&T> {
        self.0.get(y).and_then(|row| row.get(x))
    }

    pub fn set(&mut self, x: Idx, y: Idx, item: T) {
        match self.0.get_mut(y) {
            Some(row) => row.set(x, item),
            None => self.0.set(y, Contigs::new(x, item)),
        }
    }

    pub fn origin(&self) -> Coordinate<Idx> {
        Coordinate {
            x: self
                .0
                .contigs
                .iter()
                .min_by(|lhs, rhs| lhs.origin.cmp(&rhs.origin))
                .unwrap()
                .origin,
            y: self.0.origin(),
        }
    }

    pub fn rows_enumerator(&self) -> ContigsEnumerator<Idx, Contigs<Idx, T>> {
        self.0.enumerator()
    }

    pub fn neighbourhood_enumerator(&self) -> CartesianContigsNeighbourhoodEnumerator<Idx, T> {
        CartesianContigsNeighbourhoodEnumerator::new(self)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct CartesianNeighbourhood<Idx, T> {
    i_row: Idx,
    below: (Option<T>, Option<T>, Option<T>),
    this: Neighbourhood<Idx, T>,
    above: (Option<T>, Option<T>, Option<T>),
}

pub struct CartesianContigsNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy,
{
    row_enumerator: ContigsNeighbourhoodEnumerator<'a, Idx, Contigs<Idx, T>>,
    row_nbh: Option<Neighbourhood<Idx, &'a Contigs<Idx, T>>>,

    column_enumerators: Option<Neighbourhood<Idx, ContigsNeighbourhoodEnumerator<'a, Idx, T>>>,
}

impl<'a, Idx, T> CartesianContigsNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy
        + Default
        + One
        + FromPrimitive
        + AsPrimitive<usize>
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + AddAssign
        + SubAssign,
{
    fn new(c: &'a CartesianContigs<Idx, T>) -> CartesianContigsNeighbourhoodEnumerator<'a, Idx, T> {
        let row_enumerator = c.0.neighbourhood_enumerator();
        let mut result = CartesianContigsNeighbourhoodEnumerator {
            row_enumerator,
            row_nbh: None,
            column_enumerators: None,
        };

        result.advance_row();

        result
    }

    /// advance the row
    fn advance_row(&mut self) {
        self.row_nbh = self.row_enumerator.next();
        self.column_enumerators = self.row_nbh.as_ref().map(|row_nbh| {
            let row_origin = row_nbh.this.origin();

            Neighbourhood {
                i: row_nbh.i,
                left: row_nbh
                    .left
                    .map(|oc| oc.neighbourhood_enumerator_from(row_origin)),
                this: row_nbh.this.neighbourhood_enumerator(),
                right: row_nbh
                    .right
                    .map(|oc| oc.neighbourhood_enumerator_from(row_origin)),
            }
        });
    }

    /// return next column if any
    fn next_col(&mut self) -> Option<CartesianNeighbourhood<Idx, &'a T>> {
        self.column_enumerators
            .as_mut()
            .and_then(|nbh| match nbh.this.next() {
                Some(this) => {
                    let i_this = this.i;
                    Some(CartesianNeighbourhood {
                        i_row: nbh.i,
                        below: nbh
                            .left
                            .as_mut()
                            .map_or((None, None, None), |oc| oc.get(i_this)),
                        this,
                        above: nbh
                            .right
                            .as_mut()
                            .map_or((None, None, None), |oc| oc.get(i_this)),
                    })
                }
                None => None,
            })
    }
}

impl<'a, Idx, T> Iterator for CartesianContigsNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy
        + Default
        + One
        + FromPrimitive
        + AsPrimitive<usize>
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + PartialOrd
        + AddAssign
        + SubAssign,
{
    type Item = CartesianNeighbourhood<Idx, &'a T>;

    fn next(&mut self) -> Option<CartesianNeighbourhood<Idx, &'a T>> {
        self.next_col().or_else(|| {
            self.advance_row();
            self.next_col()
        })
    }
}

mod tests;
