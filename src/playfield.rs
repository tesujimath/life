// TODO remove suppression for dead code warning
#![allow(dead_code)]

use std::cmp::Ordering;
use std::collections::VecDeque;

/// a trait like Ord<RHS>
/// see https://github.com/rust-lang/rfcs/issues/2511
trait Ord2<RHS> {
    fn cmp(&self, other: &RHS) -> Ordering;
}

/// a horizontal block of contiguous cells
#[derive(Debug, Eq, PartialEq)]
struct Span {
    /// position of leftmost cell, in cell units
    left: i32,
    cells: VecDeque<u8>,
}

impl Span {
    fn new(i_cell: i32, cell: u8) -> Span {
        Span {
            left: i_cell,
            cells: VecDeque::from(vec![cell]),
        }
    }

    fn contains(&self, i_cell: i32) -> bool {
        i_cell >= self.left && i_cell < self.left + self.cells.len() as i32
    }

    fn adjoins_left(&self, i_cell: i32) -> bool {
        i_cell == self.left - 1
    }

    fn adjoins_right(&self, i_cell: i32) -> bool {
        i_cell == self.left + self.cells.len() as i32
    }

    // TODO implement via Index and IndexMut traits
    fn set(&mut self, i_cell: i32, cell: u8) {
        assert!(self.contains(i_cell));
        self.cells[(i_cell - self.left) as usize] = cell;
    }

    fn push_front(&mut self, cell: u8) {
        self.cells.push_front(cell);
        self.left -= 1;
    }

    fn push_back(&mut self, cell: u8) {
        self.cells.push_back(cell);
    }

    fn append(&mut self, other: &mut Span) {
        self.cells.append(&mut other.cells);
    }
}

impl Ord2<i32> for Span {
    fn cmp(&self, i_cell: &i32) -> Ordering {
        if *i_cell < self.left {
            Ordering::Greater
        } else if *i_cell < self.left + self.cells.len() as i32 {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }
}

/// an ordered list of spans, ordered by `left`, and coelesced opportunistically
#[derive(Debug, Eq, PartialEq)]
struct Row {
    spans: VecDeque<Span>,
}

impl Row {
    fn new() -> Row {
        Row {
            spans: VecDeque::new(),
        }
    }

    fn set(&mut self, i_cell: i32, cell: u8) {
        match self.spans.binary_search_by(|span| span.cmp(&i_cell)) {
            Ok(i_span) => {
                self.spans[i_span].set(i_cell, cell);
            }
            Err(i_span) => {
                if i_span < self.spans.len() {
                    let span_i = &mut self.spans[i_span];
                    if span_i.adjoins_left(i_cell) {
                        span_i.push_front(cell);

                        if i_span > 0 && self.spans[i_span - 1].adjoins_right(i_cell) {
                            self.coelesce_left(i_span);
                        }
                    } else if i_span > 0 {
                        let span_left = &mut self.spans[i_span - 1];
                        if span_left.adjoins_right(i_cell) {
                            span_left.push_back(cell);
                        } else {
                            self.spans.insert(i_span, Span::new(i_cell, cell));
                        }
                    } else {
                        self.spans.push_front(Span::new(i_cell, cell));
                    }
                } else {
                    match self.spans.back_mut() {
                        Some(span_left) => {
                            // TODO extract out common code with above
                            if span_left.adjoins_right(i_cell) {
                                span_left.push_back(cell);
                            } else {
                                self.spans.insert(i_span, Span::new(i_cell, cell));
                            }
                        }
                        None => self.spans.push_back(Span::new(i_cell, cell)),
                    }
                }
            }
        }
    }

    fn coelesce_left(&mut self, i_span: usize) {
        // need somewhere to move cells out of the span to be removed,
        // to avoid repeated mutable borrow from seperate VecDeque indices
        let mut cells: VecDeque<u8> = VecDeque::new();
        cells.append(&mut self.spans[i_span].cells);
        self.spans.remove(i_span);
        self.spans[i_span - 1].cells.append(&mut cells);
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
