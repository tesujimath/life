#![cfg(test)]
use super::*;

// a simple seekable iterator for testing
struct VecSeekableIterator<'a> {
    vec: &'a Vec<Option<i32>>,
    i_next: usize,
}

impl<'a> VecSeekableIterator<'a> {
    fn from(vec: &'a Vec<Option<i32>>) -> VecSeekableIterator<'a> {
        VecSeekableIterator { vec, i_next: 0 }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct IndexedItem {
    i: usize,
    item: i32,
}

impl IndexedItem {
    fn new(i: usize, item: i32) -> IndexedItem {
        IndexedItem { i, item }
    }
}

impl<'a> Iterator for VecSeekableIterator<'a> {
    type Item = IndexedItem;

    fn next(&mut self) -> Option<IndexedItem> {
        while self.i_next < self.vec.len() && self.vec[self.i_next].is_none() {
            self.i_next += 1;
        }

        if self.i_next < self.vec.len() {
            let result = IndexedItem::new(self.i_next, self.vec[self.i_next].unwrap());
            Some(result)
        } else {
            None
        }
    }
}

impl Indexed<usize> for IndexedItem {
    fn index(&self) -> usize {
        self.i
    }
}

impl<'a> SeekableIterator<usize, IndexedItem> for VecSeekableIterator<'a> {
    fn seek(&mut self, i: usize) -> Option<IndexedItem> {
        self.i_next = i;
        self.next()
    }

    fn peek(&self) -> Option<IndexedItem> {
        if self.i_next < self.vec.len() {
            self.vec[self.i_next].map(|item| IndexedItem::new(self.i_next, item))
        } else {
            None
        }
    }
}

#[test]
fn test_multi_iterator() {
    let v1 = vec![None, Some(12)];
    let v2 = vec![Some(21), Some(22)];
    let v3 = vec![Some(31), None, Some(33)];

    let v1i = VecSeekableIterator::from(&v1);
    let v2i = VecSeekableIterator::from(&v2);
    let v3i = VecSeekableIterator::from(&v3);

    let mi = MultiIterator::new(
        vec![Some(v1i), Some(v2i), Some(v3i)],
        vec![false, true, false],
    );

    let simplified_result = mi
        .map(|(_, iios)| {
            iios.iter()
                .map(|iio| iio.as_ref().map(|ii| ii.item))
                .collect::<Vec<Option<i32>>>()
        })
        .collect::<Vec<Vec<Option<i32>>>>();
    let expected = vec![vec![None, None, Some(21i32)]];

    assert_eq!(simplified_result, expected);
}
