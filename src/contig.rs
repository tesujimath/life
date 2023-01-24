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

/// a span of contiguous items
#[derive(Debug, Eq, PartialEq)]
struct Span<Idx, T>
where
    Idx: Copy,
{
    /// position of leftmost item
    origin: Idx,
    items: VecDeque<T>,
}

impl<Idx, T> Span<Idx, T>
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
    fn new(i: Idx, item: T) -> Span<Idx, T> {
        Span {
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

    fn append(&mut self, other: &mut Span<Idx, T>) {
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
        let item = &self.items[u];
        let right = self.items.get(u + 1);

        Neighbourhood {
            i,
            items: [left, Some(item), right],
        }
    }
}

impl<Idx, T> Index<Idx> for Span<Idx, T>
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

impl<Idx, T> IndexMut<Idx> for Span<Idx, T>
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

impl<Idx, T> Ord2<Idx> for Span<Idx, T>
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
    items: [Option<T>; 3],
}

/// an ordered list of spans, ordered by `origin`, and coelesced opportunistically
#[derive(Debug, Eq, PartialEq)]
pub struct Contig<Idx, T>
where
    Idx: Copy,
{
    spans: VecDeque<Span<Idx, T>>,
}

enum ContigUpdate {
    Set(usize),
    PushFront(usize),
    PushFrontAndCoelesce(usize),
    PushBack(usize),
    Insert(usize),
}

impl<Idx, T> Contig<Idx, T>
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
    fn new(i: Idx, item: T) -> Contig<Idx, T> {
        let s = Span::new(i, item);
        let mut spans = VecDeque::new();
        spans.push_front(s);
        Contig { spans }
    }

    fn from<I>(into_it: I) -> Option<Contig<Idx, T>>
    where
        I: IntoIterator<Item = (Idx, T)>,
    {
        let mut it = into_it.into_iter();

        match it.by_ref().next() {
            Some((i, item)) => {
                let c0 = Contig::new(i, item);
                Some(it.fold(c0, |mut c, (i, item)| {
                    c.set(i, item);
                    c
                }))
            }
            None => None,
        }
    }

    fn origin(&self) -> Idx {
        self.spans[0].origin
    }

    fn determine_update(&self, i: Idx) -> ContigUpdate {
        use ContigUpdate::*;

        match self.spans.binary_search_by(|c| c.cmp(&i)) {
            Ok(u) => Set(u),
            Err(u) => {
                let c_left_o = if u > 0 {
                    self.spans.get(u - 1)
                } else {
                    self.spans.get(u)
                };

                match (c_left_o, self.spans.get(u)) {
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
            self.spans[u - 1].get(i)
        } else {
            None
        }
    }

    /// provide a reference to the indexed item
    fn get(&self, i: Idx) -> Option<&T> {
        if let Ok(u) = self.spans.binary_search_by(|c| c.cmp(&i)) {
            self.spans[u].get(i)
        } else {
            None
        }
    }

    /// provide a mutable reference to the indexed item
    fn get_mut(&mut self, i: Idx) -> Option<&mut T> {
        if let Ok(u) = self.spans.binary_search_by(|c| c.cmp(&i)) {
            self.spans[u].get_mut(i)
        } else {
            None
        }
    }

    fn set(&mut self, i: Idx, item: T) {
        use ContigUpdate::*;

        match self.determine_update(i) {
            Set(u) => self.spans[u][i] = item,

            PushFront(u) => self.spans[u].push_front(item),

            PushFrontAndCoelesce(u) => {
                self.spans[u].push_front(item);
                self.coelesce_left(u);
            }

            PushBack(u) => self.spans[u].push_back(item),

            Insert(u) => self.spans.insert(u, Span::new(i, item)),
        }
    }

    fn coelesce_left(&mut self, u: usize) {
        if let Some(mut removed_c) = self.spans.remove(u) {
            self.spans[u - 1].append(&mut removed_c);
        }
    }

    pub fn enumerator(&self) -> ContigEnumerator<Idx, T> {
        let next_i = self.spans[0].origin;

        ContigEnumerator::new(self, 0, next_i)
    }

    pub fn neighbourhood_enumerator(&self) -> ContigNeighbourhoodEnumerator<Idx, T> {
        let next_i = self.spans[0].origin;

        ContigNeighbourhoodEnumerator::new(self, 0, next_i)
    }

    fn find(&self, i: Idx) -> (usize, Idx) {
        match self.spans.binary_search_by(|c| c.cmp(&i)) {
            Ok(u) => (u, i),
            Err(u) => {
                if u < self.spans.len() {
                    (u, self.spans[u].origin)
                } else {
                    (u, i)
                }
            }
        }
    }

    pub fn neighbourhood_enumerator_from(&self, i: Idx) -> ContigNeighbourhoodEnumerator<Idx, T> {
        let (next_u, next_i) = self.find(i);

        ContigNeighbourhoodEnumerator::new(self, next_u, next_i)
    }
}

pub struct ContigNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy,
{
    c: &'a Contig<Idx, T>,
    u_next: usize,
    i_next: Idx,
}

impl<'a, Idx, T> ContigNeighbourhoodEnumerator<'a, Idx, T>
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
        c: &'a Contig<Idx, T>,
        u_next: usize,
        i_next: Idx,
    ) -> ContigNeighbourhoodEnumerator<'a, Idx, T> {
        ContigNeighbourhoodEnumerator { c, u_next, i_next }
    }

    /// advance the enumerator
    fn advance(&mut self) {
        self.i_next += Idx::one();
        if !self.c.spans[self.u_next].contains(self.i_next) {
            self.u_next += 1;
            if self.u_next < self.c.spans.len() {
                self.i_next = self.c.spans[self.u_next].origin;
            }
        }
    }

    /// return the neighbourhood for `i`,
    /// positioning the iterator after the returned item, which may be backwards
    fn get(&mut self, i: Idx) -> [Option<&'a T>; 3] {
        let i_left = i - Idx::from_usize(1).unwrap();
        let i_right = i + Idx::from_usize(1).unwrap();

        if i < self.i_next {
            (self.u_next, self.i_next) = self.c.find(i);
        }

        // skip contig
        while self.u_next < self.c.spans.len()
            && self.c.spans[self.u_next].cmp(&i) == Ordering::Less
        {
            self.u_next += 1;
        }

        if self.u_next < self.c.spans.len() {
            let c = &self.c.spans[self.u_next];
            let item_this = c.get(i);
            let item_left = match item_this {
                Some(_) => c.get(i_left),
                // this is the tricky case where we have to look in the contig to the left, if any:
                None => c
                    .get(i_left)
                    .or_else(|| self.c.get_in_left(self.u_next, i_left)),
            };
            let item_right = c.get(i_right);
            [item_left, item_this, item_right]
        } else {
            // again, we need to look in the contig to the left, if any
            let item_left = self.c.get_in_left(self.u_next, i_left);
            [item_left, None, None]
        }
    }
}

impl<'a, Idx, T> Iterator for ContigNeighbourhoodEnumerator<'a, Idx, T>
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
        if self.u_next < self.c.spans.len() {
            let nbh = self.c.spans[self.u_next].get_neighbourhood(self.i_next);
            self.advance();
            Some(nbh)
        } else {
            None
        }
    }
}

/// simple enumerator without the neighbourhood
pub struct ContigEnumerator<'a, Idx, T>
where
    Idx: Copy,
{
    c: &'a Contig<Idx, T>,
    u_next: usize,
    i_next: Idx,
}

impl<'a, Idx, T> ContigEnumerator<'a, Idx, T>
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
    fn new(c: &'a Contig<Idx, T>, u_next: usize, i_next: Idx) -> ContigEnumerator<'a, Idx, T> {
        ContigEnumerator { c, u_next, i_next }
    }

    /// advance the enumerator
    fn advance(&mut self) {
        self.i_next += Idx::one();
        if !self.c.spans[self.u_next].contains(self.i_next) {
            self.u_next += 1;
            if self.u_next < self.c.spans.len() {
                self.i_next = self.c.spans[self.u_next].origin;
            }
        }
    }
}

impl<'a, Idx, T> Iterator for ContigEnumerator<'a, Idx, T>
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
        if self.u_next < self.c.spans.len() {
            let i = self.i_next;
            let item = &self.c.spans[self.u_next][i];
            self.advance();
            Some((i, item))
        } else {
            None
        }
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
pub struct CartesianContig<Idx, T>(Contig<Idx, Contig<Idx, T>>)
where
    Idx: Copy;

impl<Idx, T> CartesianContig<Idx, T>
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
    pub fn new(x: Idx, y: Idx, item: T) -> CartesianContig<Idx, T> {
        CartesianContig(Contig::new(y, Contig::new(x, item)))
    }

    pub fn get(&self, x: Idx, y: Idx) -> Option<&T> {
        self.0.get(y).and_then(|row| row.get(x))
    }

    pub fn set(&mut self, x: Idx, y: Idx, item: T) {
        match self.0.get_mut(y) {
            Some(row) => row.set(x, item),
            None => self.0.set(y, Contig::new(x, item)),
        }
    }

    pub fn origin(&self) -> Coordinate<Idx> {
        Coordinate {
            x: self
                .0
                .spans
                .iter()
                .min_by(|lhs, rhs| lhs.origin.cmp(&rhs.origin))
                .unwrap()
                .origin,
            y: self.0.origin(),
        }
    }

    pub fn rows_enumerator(&self) -> ContigEnumerator<Idx, Contig<Idx, T>> {
        self.0.enumerator()
    }

    pub fn neighbourhood_enumerator(&self) -> CartesianContigNeighbourhoodEnumerator<Idx, T> {
        CartesianContigNeighbourhoodEnumerator::new(self)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct CartesianNeighbourhood<Idx, T> {
    i_row: Idx,
    i_col: Idx,
    items: [[Option<T>; 3]; 3], // first index is row
}

pub struct CartesianContigNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy,
{
    row_enumerator: ContigNeighbourhoodEnumerator<'a, Idx, Contig<Idx, T>>,
    row_nbh: Option<Neighbourhood<Idx, &'a Contig<Idx, T>>>,

    column_enumerators: Option<Neighbourhood<Idx, ContigNeighbourhoodEnumerator<'a, Idx, T>>>,
}

impl<'a, Idx, T> CartesianContigNeighbourhoodEnumerator<'a, Idx, T>
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
    fn new(c: &'a CartesianContig<Idx, T>) -> CartesianContigNeighbourhoodEnumerator<'a, Idx, T> {
        let row_enumerator = c.0.neighbourhood_enumerator();
        let mut result = CartesianContigNeighbourhoodEnumerator {
            row_enumerator,
            row_nbh: None,
            column_enumerators: None,
        };

        result.advance_row();

        result
    }

    /// advance to the next non-empty row, if any
    fn advance_row(&mut self) {
        loop {
            self.row_nbh = self.row_enumerator.next();
            match self.row_nbh {
                Some(ref row_nbh) => {
                    if row_nbh.items[1].is_some() {
                        break;
                    }
                }
                None => break,
            }
        }
        if let Some(ref mut row_nbh) = self.row_nbh {
            self.column_enumerators = {
                Some(Neighbourhood {
                    i: row_nbh.i,
                    items: row_nbh
                        .items
                        .map(|item| item.map(|c| c.neighbourhood_enumerator())),
                })
            };
        }
    }

    /// return next column if any
    fn next_col(&mut self) -> Option<CartesianNeighbourhood<Idx, &'a T>> {
        self.column_enumerators
            .as_mut()
            .and_then(|rows| match &mut rows.items[1] {
                Some(ref mut row_1_e) => match row_1_e.next() {
                    Some(row_1_nbh) => {
                        let i_col = row_1_nbh.i;
                        let absent: [Option<&'a T>; 3] = [None, None, None];
                        Some(CartesianNeighbourhood {
                            i_row: rows.i,
                            i_col,
                            items: [
                                rows.items[0].as_mut().map_or(absent, |c| c.get(i_col)),
                                row_1_nbh.items,
                                rows.items[2].as_mut().map_or(absent, |c| c.get(i_col)),
                            ],
                        })
                    }
                    None => None,
                },
                None => None,
            })
    }
}

impl<'a, Idx, T> Iterator for CartesianContigNeighbourhoodEnumerator<'a, Idx, T>
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
