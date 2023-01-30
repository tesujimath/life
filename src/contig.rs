// TODO remove suppression for dead code warning
#![allow(dead_code)]

use super::multi_iterator::SeekableIterator;
use super::neighbourhood::Neighbourhood;
use num::cast::AsPrimitive;
use num::FromPrimitive;
use num::One;
use std::cmp::Ordering;
use std::cmp::PartialOrd;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Sub;
use std::ops::SubAssign;

/// a span of contiguous items
#[derive(Debug, Eq, PartialEq)]
struct Span<Idx, T>
where
    Idx: Copy + Debug,
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
        + SubAssign
        + Debug,
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

    /// return whether `i` is contained or adjoining
    fn contains_or_adjoins(&self, i: Idx) -> bool {
        i >= self.origin - Idx::one()
            && i <= self.origin + Idx::from_usize(self.items.len()).unwrap()
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
        let left = (u > 0).then(|| &self.items[u - 1]);
        let item = &self.items[u];
        let right = self.items.get(u + 1);

        Neighbourhood::new(i, [left, Some(item), right])
    }

    fn cmp(&self, i: &Idx) -> Ordering {
        if *i < self.origin {
            Ordering::Greater
        } else if *i < self.origin + Idx::from_usize(self.items.len()).unwrap() {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }

    fn cmp_with_adjacent(&self, i: &Idx) -> Ordering {
        if *i < self.origin - Idx::one() {
            Ordering::Greater
        } else if *i <= self.origin + Idx::from_usize(self.items.len()).unwrap() {
            Ordering::Equal
        } else {
            Ordering::Less
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
        + SubAssign
        + Debug,
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
        + SubAssign
        + Debug,
{
    fn index_mut(&mut self, i: Idx) -> &mut Self::Output {
        &mut self.items[Idx::as_(i - self.origin)]
    }
}

/// an ordered list of spans, ordered by `origin`, and coelesced opportunistically
#[derive(Debug, Eq, PartialEq)]
pub struct Contig<Idx, T>
where
    Idx: Copy + Debug,
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
        + SubAssign
        + Debug,
{
    pub fn new(i: Idx, item: T) -> Contig<Idx, T> {
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

    pub fn origin(&self) -> Idx {
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

    /// provide a reference to the span one to the left, if any
    fn get_left_of(&self, u: usize) -> Option<&Span<Idx, T>> {
        if u > 0 {
            Some(&self.spans[u - 1])
        } else {
            None
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
    pub fn get(&self, i: Idx) -> Option<&T> {
        if let Ok(u) = self.spans.binary_search_by(|c| c.cmp(&i)) {
            self.spans[u].get(i)
        } else {
            None
        }
    }

    /// provide a mutable reference to the indexed item
    pub fn get_mut(&mut self, i: Idx) -> Option<&mut T> {
        if let Ok(u) = self.spans.binary_search_by(|c| c.cmp(&i)) {
            self.spans[u].get_mut(i)
        } else {
            None
        }
    }

    pub fn set(&mut self, i: Idx, item: T) {
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
        ContigNeighbourhoodEnumerator::new(self)
    }

    /// in case of a gap of one, prefer left adjoining of the right span over the right adjoining of the left span
    pub fn normalised(&self, u: usize, i: Idx) -> usize {
        match self.spans.get(u + 1) {
            Some(span_right) if span_right.adjoins_left(i) => u + 1,
            _ => u,
        }
    }

    /// find the first item at or past the given index, including adjacent siblings,
    fn find_with_adjacent(&self, i: Idx) -> (usize, Idx) {
        match self.spans.binary_search_by(|c| c.cmp_with_adjacent(&i)) {
            Ok(u) => (self.normalised(u, i), i),
            Err(u) => {
                if u < self.spans.len() {
                    (u, self.spans[u].origin - Idx::one())
                } else {
                    (u, i)
                }
            }
        }
    }
}

/// simple enumerator without the neighbourhood
pub struct ContigEnumerator<'a, Idx, T>
where
    Idx: Copy + Debug,
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
        + SubAssign
        + Debug,
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
        + SubAssign
        + Debug,
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

/// an iterator which returns neighbourhoods for all items and their adjacent siblings, with indices
pub struct ContigNeighbourhoodEnumerator<'a, Idx, T>
where
    Idx: Copy + Debug,
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
        + SubAssign
        + Debug,
{
    fn new(c: &'a Contig<Idx, T>) -> ContigNeighbourhoodEnumerator<'a, Idx, T> {
        let u_next = 0;
        let i_next = c.spans[u_next].origin - Idx::one();

        ContigNeighbourhoodEnumerator { c, u_next, i_next }
    }

    /// return the current neighbourhood without advancing
    fn get_current(&self) -> Option<Neighbourhood<'a, Idx, &'a T>> {
        // TODO tidy this up
        if self.u_next < self.c.spans.len() {
            let span = &self.c.spans[self.u_next];

            let nbh = if span.contains(self.i_next) {
                self.c.spans[self.u_next].get_neighbourhood(self.i_next)
            } else if span.adjoins_left(self.i_next) {
                // the item before this span, which could be in the gap between spans if that's a gap of one
                let item_left = self
                    .c
                    .get_left_of(self.u_next)
                    .and_then(|span_left| span_left.get(self.i_next - Idx::one()));
                Neighbourhood::new(self.i_next, [item_left, None, span.get(span.origin)])
            } else {
                assert!(span.adjoins_right(self.i_next));

                // there'll never be a gap of one situation here, since in that case we would have advanced onto the next span
                Neighbourhood::new(
                    self.i_next,
                    [span.get(self.i_next - Idx::one()), None, None],
                )
            };

            Some(nbh)
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if self.u_next < self.c.spans.len() {
            if self.u_next < self.c.spans.len() {
                self.i_next += Idx::one();
                if !self.c.spans[self.u_next].contains_or_adjoins(self.i_next) {
                    self.u_next += 1;
                    if self.u_next < self.c.spans.len() {
                        self.i_next = self.c.spans[self.u_next].origin - Idx::one();
                    }
                }

                self.u_next = self.c.normalised(self.u_next, self.i_next);
            }
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
        + SubAssign
        + Debug,
{
    type Item = Neighbourhood<'a, Idx, &'a T>;

    fn next(&mut self) -> Option<Neighbourhood<'a, Idx, &'a T>> {
        let result = self.get_current();
        self.advance();
        result
    }
}

impl<'a, Idx, T> SeekableIterator<Idx, Neighbourhood<'a, Idx, &'a T>>
    for ContigNeighbourhoodEnumerator<'a, Idx, T>
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
        + SubAssign
        + Debug,
{
    /// seek to any index including adjacent locations
    fn seek(&mut self, i_from: Idx) -> Option<Neighbourhood<'a, Idx, &'a T>> {
        // look in current and adjacent spans before falling back to find
        if self.u_next < self.c.spans.len() {
            let span = &self.c.spans[self.u_next];
            let span_left_o = self.c.get_left_of(self.u_next);
            let span_right_o = self.c.spans.get(self.u_next + 1);

            if span.contains(i_from)
                || span.adjoins_left(i_from)
                || (span.adjoins_right(i_from)
                    && !span_right_o.map_or(false, |span_right| span_right.adjoins_left(i_from)))
            {
                self.i_next = i_from;
            } else if span_right_o.map_or(false, |span_right| {
                span_right.contains(i_from) || span_right.adjoins_left(i_from)
            }) {
                // the adjoins right case is handled by find fallback, to avoid checking the right of right span
                self.u_next += 1;
                self.i_next = i_from;
            } else if span_left_o.map_or(false, |span_left| {
                span_left.contains(i_from)
                    || span_left.adjoins_left(i_from)
                    || span_left.adjoins_right(i_from)
            }) {
                self.u_next -= 1;
                self.i_next = i_from;
            } else {
                (self.u_next, self.i_next) = self.c.find_with_adjacent(i_from);
            }
        } else {
            (self.u_next, self.i_next) = self.c.find_with_adjacent(i_from);
        }

        if self.i_next == i_from {
            self.next()
        } else {
            None
        }
    }

    fn peek(&self) -> Option<Neighbourhood<'a, Idx, &'a T>> {
        self.get_current()
    }
}

mod tests;
