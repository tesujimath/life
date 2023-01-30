use std::fmt::Debug;
use std::marker::PhantomData;

/// something which has an index
pub trait Indexed<Idx> {
    fn index(&self) -> Idx;
}

pub trait SeekableIterator<Idx, T>: Iterator<Item = T> {
    /// seek to first item at or past `i`, consuming it only if it is exactly at `i`
    fn seek(&mut self, i: Idx) -> Option<T>;

    // this isn't quite like standard library peek(), since usage here is with a vector which is created as the iterator progresses
    // so we return by value not reference
    // TODO think if there's a better way, or consider renaming
    fn peek(&self) -> Option<T>;
}

/// An iterator over multiple seekable iterators.
pub struct MultiIterator<Idx, I, T>
where
    I: SeekableIterator<Idx, T>,
    T: Indexed<Idx>,
{
    iterators: Vec<Option<I>>,
    drivers: Vec<bool>,
    phantom_idx: PhantomData<Idx>,
    phantom_t: PhantomData<T>,
}

impl<Idx, I, T> MultiIterator<Idx, I, T>
where
    I: SeekableIterator<Idx, T>,
    T: Indexed<Idx> + Debug,
    Idx: Copy + PartialOrd + Debug,
{
    pub fn new(iterators: Vec<Option<I>>, drivers: Vec<bool>) -> MultiIterator<Idx, I, T> {
        MultiIterator {
            iterators,
            drivers,
            phantom_idx: PhantomData,
            phantom_t: PhantomData,
        }
    }

    /// return index of next item
    fn determine_next(&mut self) -> Option<Idx> {
        let mut min_o: Option<Idx> = None;
        for (u, driver) in self.drivers.iter().enumerate() {
            if *driver {
                let next_o = self.iterators[u]
                    .as_mut()
                    .and_then(|ref mut it| it.peek().map(|item| item.index()));

                match (min_o, next_o) {
                    (None, _) => {
                        min_o = next_o;
                    }
                    (Some(min), Some(next)) if next < min => min_o = next_o,
                    _ => (),
                }
            }
        }

        min_o
    }

    /// consume the next item
    fn consume_next(&mut self, i: Idx) -> Option<(Idx, Vec<Option<T>>)> {
        let mut items: Vec<Option<T>> = Vec::with_capacity(self.iterators.len());
        for p_o in self.iterators.iter_mut() {
            let item = if let Some(p) = p_o { p.seek(i) } else { None };
            items.push(item);
        }

        Some((i, items))
    }
}

impl<Idx, I, T> Iterator for MultiIterator<Idx, I, T>
where
    I: SeekableIterator<Idx, T>,
    T: Indexed<Idx> + Debug,
    Idx: Copy + PartialOrd + Debug,
{
    type Item = (Idx, Vec<Option<T>>);

    fn next(&mut self) -> Option<(Idx, Vec<Option<T>>)> {
        match self.determine_next() {
            Some(i) => self.consume_next(i),
            None => None,
        }
    }
}

mod tests;
