use std::cmp::Ordering;
use std::collections::VecDeque;

/// a trait like Ord<RHS>
/// see https://github.com/rust-lang/rfcs/issues/2511
trait Ord2<RHS> {
    fn cmp(&self, other: &RHS) -> Ordering;
}

/// a horizontal block of contiguous cells
#[derive(Debug)]
struct Span {
    /// position of leftmost cell, in cell units
    left: i32,
    cells: VecDeque<u8>,
}

impl PartialEq<i32> for Span {
    fn eq(&self, x: &i32) -> bool {
        *x >= self.left && *x < self.left + self.cells.len() as i32
    }
}

impl PartialOrd<i32> for Span {
    fn partial_cmp(&self, x: &i32) -> Option<Ordering> {
        if *x < self.left {
            Some(Ordering::Greater)
        } else if *x < self.left + self.cells.len() as i32 {
            Some(Ordering::Equal)
        } else {
            Some(Ordering::Less)
        }
    }
}

impl Ord2<i32> for Span {
    fn cmp(&self, x: &i32) -> Ordering {
        if *x < self.left {
            Ordering::Greater
        } else if *x < self.left + self.cells.len() as i32 {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }
}

/// an ordered list of spans, ordered by `left`, and coelesced opportunistically
#[derive(Debug)]
struct Row {
    spans: VecDeque<Span>,
}

impl Row {
    fn set(&mut self, x: i32, val: u8) {
        match self
            .spans
            .binary_search_by(|span| Ordering::Greater /* span.partial_cmp(&x)*/)
        {
            Ok(i) => (),
            Err(i) => (),
        }
    }
}

/// a vertical block of contiguous rows
#[derive(Debug)]
struct Drop {
    bottom: i32,
    rows: VecDeque<Row>,
}

/// ordered list of drops, ordered by `bottom`, and coelesced opportunistically
type Playfield = VecDeque<Drop>;

mod tests;
