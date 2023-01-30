#![cfg(test)]
use super::*;

// a simple seekable iterator for testing
struct VecSeekableIterator<'a> {
    vec: &'a Vec<Option<i32>>,
    i_next: usize, // points to an element which exists, or past the end
}

impl<'a> VecSeekableIterator<'a> {
    fn from(vec: &'a Vec<Option<i32>>) -> VecSeekableIterator<'a> {
        let mut this = VecSeekableIterator { vec, i_next: 0 };
        this.skip_missing();

        this
    }

    fn skip_missing(&mut self) {
        while self.i_next < self.vec.len() && self.vec[self.i_next].is_none() {
            self.i_next += 1;
        }
    }

    fn advance(&mut self) {
        if self.i_next < self.vec.len() {
            self.i_next += 1;
        }
        self.skip_missing();
    }

    fn get_current(&self) -> Option<IndexedItem> {
        if self.i_next < self.vec.len() {
            self.vec[self.i_next].map(|item| IndexedItem::new(self.i_next, item))
        } else {
            None
        }
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
        if self.i_next < self.vec.len() {
            let item = self.get_current();
            self.advance();
            item
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
        self.skip_missing();
        if self.i_next == i {
            let item = self.get_current();
            self.advance();
            item
        } else {
            None
        }
    }

    fn peek(&self) -> Option<IndexedItem> {
        self.get_current()
    }
}

#[test]
fn test_multi_iterator() {
    fn multi_iterate(
        mi: MultiIterator<usize, VecSeekableIterator, IndexedItem>,
    ) -> Vec<Vec<Option<i32>>> {
        mi.map(|(_, iios)| {
            iios.iter()
                .map(|iio| iio.as_ref().map(|ii| ii.item))
                .collect::<Vec<Option<i32>>>()
        })
        .collect::<Vec<Vec<Option<i32>>>>()
    }

    let v1 = vec![None, Some(12)];
    let v2 = vec![Some(21), Some(22)];
    let v3 = vec![Some(31), None, Some(33)];

    assert_eq![
        VecSeekableIterator::from(&v1).collect::<Vec<IndexedItem>>(),
        vec![IndexedItem::new(1, 12)]
    ];

    assert_eq![
        VecSeekableIterator::from(&v2).collect::<Vec<IndexedItem>>(),
        vec![IndexedItem::new(0, 21), IndexedItem::new(1, 22)]
    ];

    assert_eq![
        VecSeekableIterator::from(&v3).collect::<Vec<IndexedItem>>(),
        vec![IndexedItem::new(0, 31), IndexedItem::new(2, 33)]
    ];

    assert_eq!(
        multi_iterate(MultiIterator::new(
            vec![
                Some(VecSeekableIterator::from(&v1)),
                Some(VecSeekableIterator::from(&v2)),
                Some(VecSeekableIterator::from(&v3)),
            ],
            vec![false, true, false],
        )),
        vec![
            vec![None, Some(21), Some(31)],
            vec![Some(12), Some(22), None],
        ]
    );

    assert_eq!(
        multi_iterate(MultiIterator::new(
            vec![
                Some(VecSeekableIterator::from(&v1)),
                Some(VecSeekableIterator::from(&v2)),
                Some(VecSeekableIterator::from(&v3)),
            ],
            vec![true, true, false],
        )),
        vec![
            vec![None, Some(21), Some(31)],
            vec![Some(12), Some(22), None],
        ]
    );

    assert_eq!(
        multi_iterate(MultiIterator::new(
            vec![
                Some(VecSeekableIterator::from(&v1)),
                Some(VecSeekableIterator::from(&v2)),
                Some(VecSeekableIterator::from(&v3)),
            ],
            vec![false, false, true],
        )),
        vec![vec![None, Some(21), Some(31)], vec![None, None, Some(33)],]
    );

    assert_eq!(
        multi_iterate(MultiIterator::new(
            vec![
                Some(VecSeekableIterator::from(&v1)),
                Some(VecSeekableIterator::from(&v2)),
                Some(VecSeekableIterator::from(&v3)),
            ],
            vec![true, false, true],
        )),
        vec![
            vec![None, Some(21), Some(31)],
            vec![Some(12), Some(22), None],
            vec![None, None, Some(33)],
        ]
    );
}
