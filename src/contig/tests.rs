#[cfg(test)]
use super::*;

#[test]
fn test_span_cmp() {
    assert_eq!(
        Span {
            origin: 1,
            items: VecDeque::<u8>::new()
        }
        .cmp(&0),
        Ordering::Greater
    );

    assert_eq!(
        Span {
            origin: 1,
            items: VecDeque::<u8>::new()
        }
        .cmp(&1),
        Ordering::Less
    );

    assert_eq!(
        Span {
            origin: 1,
            items: VecDeque::<u8>::new()
        }
        .cmp(&2),
        Ordering::Less
    );

    assert_eq!(
        Span {
            origin: 1,
            items: VecDeque::from(vec![7u8])
        }
        .cmp(&0),
        Ordering::Greater
    );

    assert_eq!(
        Span {
            origin: 1,
            items: VecDeque::from(vec![7u8])
        }
        .cmp(&1),
        Ordering::Equal
    );

    assert_eq!(
        Span {
            origin: 1,
            items: VecDeque::from(vec![7u8])
        }
        .cmp(&2),
        Ordering::Less
    );

    assert_eq!(
        Span {
            origin: 1,
            items: VecDeque::from(vec![5u8, 7u8])
        }
        .cmp(&2),
        Ordering::Equal
    );
}

#[test]
fn test_spans_binary_search() {
    let spans = VecDeque::from(vec![
        Span {
            origin: 0,
            items: VecDeque::from(vec![5u8, 7u8]),
        },
        Span {
            origin: 3,
            items: VecDeque::from(vec![3u8]),
        },
    ]);

    assert_eq!(spans.binary_search_by(|span| span.cmp(&-1)), Err(0));
    assert_eq!(spans.binary_search_by(|span| span.cmp(&1)), Ok(0));
    assert_eq!(spans.binary_search_by(|span| span.cmp(&2)), Err(1));
    assert_eq!(spans.binary_search_by(|span| span.cmp(&3)), Ok(1));
    assert_eq!(spans.binary_search_by(|span| span.cmp(&4)), Err(2));
}

#[test]
fn test_contig_set() {
    let c = &mut Contig::new(10, 10u8);
    assert_eq!(
        *c,
        Contig {
            spans: VecDeque::from(vec![Span {
                origin: 10,
                items: VecDeque::from(vec![10u8])
            },])
        }
    );

    c.set(11, 11u8);
    assert_eq!(
        *c,
        Contig {
            spans: VecDeque::from(vec![Span {
                origin: 10,
                items: VecDeque::from(vec![10u8, 11u8])
            },])
        }
    );

    c.set(9, 9u8);
    assert_eq!(
        *c,
        Contig {
            spans: VecDeque::from(vec![Span {
                origin: 9,
                items: VecDeque::from(vec![9u8, 10u8, 11u8])
            }])
        }
    );

    c.set(7, 7u8);
    assert_eq!(
        *c,
        Contig {
            spans: VecDeque::from(vec![
                Span {
                    origin: 7,
                    items: VecDeque::from(vec![7u8])
                },
                Span {
                    origin: 9,
                    items: VecDeque::from(vec![9u8, 10u8, 11u8])
                }
            ])
        }
    );

    c.set(8, 8u8);
    assert_eq!(
        *c,
        Contig {
            spans: VecDeque::from(vec![Span {
                origin: 7,
                items: VecDeque::from(vec![7u8, 8u8, 9u8, 10u8, 11u8])
            }])
        }
    );
}

#[test]
fn test_contig_get() {
    let mut c = Contig::new(10, 10u8);

    c.set(11, 11u8);
    c.set(13, 13u8);

    assert_eq!(c.get(9), None);
    assert_eq!(c.get(10), Some(&10u8));
    assert_eq!(c.get(11), Some(&11u8));
    assert_eq!(c.get(12), None);
    assert_eq!(c.get(13), Some(&13u8));
    assert_eq!(c.get(14), None);
}

#[test]
fn test_contig_find_with_adjacent() {
    let c = Contig::from(vec![
        (10, 10u8),
        (11, 11u8),
        (13, 13u8),
        (14, 14u8),
        (20, 20u8),
    ])
    .unwrap();

    assert_eq!(c.find_with_adjacent(-1), (0, 9));
    assert_eq!(c.find_with_adjacent(9), (0, 9));
    assert_eq!(c.find_with_adjacent(10), (0, 10));
    assert_eq!(c.find_with_adjacent(11), (0, 11));
    assert_eq!(c.find_with_adjacent(12), (1, 12));
    assert_eq!(c.find_with_adjacent(13), (1, 13));
    assert_eq!(c.find_with_adjacent(14), (1, 14));
    assert_eq!(c.find_with_adjacent(15), (1, 15));
    assert_eq!(c.find_with_adjacent(16), (2, 19));
    assert_eq!(c.find_with_adjacent(20), (2, 20));
    assert_eq!(c.find_with_adjacent(21), (2, 21));
    assert_eq!(c.find_with_adjacent(100), (3, 100));
}

#[test]
fn test_contig_neighbourhood_enumerator() {
    fn enumerator_as_vec(c: &Contig<i32, u8>) -> Vec<Neighbourhood<i32, &u8>> {
        c.neighbourhood_enumerator()
            //.map(|nbh| (nbh.i, nbh.left.copied(), *nbh.this, nbh.right.copied()))
            .collect()
    }

    let c = Contig::from(vec![(10, 10), (11, 11), (13, 13), (14, 14), (20, 20)]).unwrap();

    assert_eq!(
        enumerator_as_vec(&c),
        vec![
            Neighbourhood {
                i: 9,
                items: [None, None, Some(&10)]
            },
            Neighbourhood {
                i: 10,
                items: [None, Some(&10), Some(&11)]
            },
            Neighbourhood {
                i: 11,
                items: [Some(&10), Some(&11), None]
            },
            Neighbourhood {
                i: 12,
                items: [Some(&11), None, Some(&13)]
            },
            Neighbourhood {
                i: 13,
                items: [None, Some(&13), Some(&14)]
            },
            Neighbourhood {
                i: 14,
                items: [Some(&13), Some(&14), None]
            },
            Neighbourhood {
                i: 15,
                items: [Some(&14), None, None]
            },
            Neighbourhood {
                i: 19,
                items: [None, None, Some(&20)]
            },
            Neighbourhood {
                i: 20,
                items: [None, Some(&20), None]
            },
            Neighbourhood {
                i: 21,
                items: [Some(&20), None, None]
            },
        ]
    );
}

#[test]
fn test_contig_neighbourhood_enumerator_seek() {
    let c = Contig::from(vec![
        (10, 10u8),
        (11, 11u8),
        (13, 13u8),
        (14, 114u8),
        (20, 120u8),
        (30, 130u8),
    ])
    .unwrap();
    let mut e = c.neighbourhood_enumerator();

    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((9, None)));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((10, Some(&10))));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((11, Some(&11))));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((12, None)));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((13, Some(&13))));

    e.seek(10);
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((10, Some(&10))));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((11, Some(&11))));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((12, None)));

    e.seek(-1);
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((9, None)));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((10, Some(&10))));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((11, Some(&11))));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((12, None)));

    e.seek(11);
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((11, Some(&11))));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((12, None)));

    e.seek(12);
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((12, None)));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((13, Some(&13))));

    e.seek(11);
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((11, Some(&11))));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((12, None)));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((13, Some(&13))));

    e.seek(19);
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((19, None)));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((20, Some(&120))));

    e.seek(20);
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((20, Some(&120))));

    e.seek(21);
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((21, None)));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((29, None)));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((30, Some(&130))));

    e.seek(30);
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((30, Some(&130))));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((31, None)));

    e.seek(31);
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((31, None)));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), None);

    e.seek(31);
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((31, None)));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), None);

    e.seek(100);
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), None);

    e.seek(12);
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((12, None)));
    assert_eq!(e.next().map(|n| (n.i, n.items[1])), Some((13, Some(&13))));
}

#[test]
fn test_contig_neighbourhood_enumerator_get() {
    let c = Contig::from(vec![(10, 10u8), (11, 11u8), (13, 13u8)]).unwrap();

    let mut e = c.neighbourhood_enumerator();
    assert_eq!(e.get(9), [None, None, Some(&10u8)]);
    assert_eq!(e.get(10), [None, Some(&10u8), Some(&11u8)]);
    assert_eq!(e.get(11), [Some(&10u8), Some(&11u8), None]);
    assert_eq!(e.get(12), [Some(&11u8), None, Some(&13u8)]);
    assert_eq!(e.get(13), [None, Some(&13u8), None]);
    assert_eq!(e.get(14), [Some(&13u8), None, None]);
}

#[test]
fn test_cartesian_contig_set() {
    let mut cc = CartesianContig::new(0, 0, 0u8);
    cc.set(1, 1, 11u8);
    cc.set(4, 2, 42u8);

    assert_eq!(cc.get(1, 1), Some(&11u8));
    assert_eq!(cc.get(1, 2), None);
    assert_eq!(cc.get(4, 1), None);
    assert_eq!(cc.get(4, 2), Some(&42u8));
}

#[test]
fn test_cartesian_contig_neighbourhood_enumerator() {
    fn enumerator_as_vec(cc: &CartesianContig<i32, u8>) -> Vec<CartesianNeighbourhood<i32, &u8>> {
        cc.neighbourhood_enumerator().collect()
    }

    let mut cc = CartesianContig::new(0, 0, 0u8);
    cc.set(1, 0, 1u8);
    cc.set(2, 0, 2u8);
    cc.set(1, 1, 11u8);

    assert_eq!(
        enumerator_as_vec(&cc),
        vec![
            CartesianNeighbourhood {
                i_row: 0,
                i_col: 0,
                items: [
                    [None, None, None],
                    [None, Some(&0), Some(&1)],
                    [None, None, Some(&11)]
                ]
            },
            CartesianNeighbourhood {
                i_row: 0,
                i_col: 1,
                items: [
                    [None, None, None],
                    [Some(&0), Some(&1), Some(&2)],
                    [None, Some(&11), None]
                ]
            },
            CartesianNeighbourhood {
                i_row: 0,
                i_col: 2,
                items: [
                    [None, None, None],
                    [Some(&1), Some(&2), None],
                    [Some(&11), None, None]
                ]
            },
            CartesianNeighbourhood {
                i_row: 1,
                i_col: 1,
                items: [
                    [Some(&0), Some(&1), Some(&2)],
                    [None, Some(&11), None],
                    [None, None, None]
                ]
            },
        ]
    );
}
