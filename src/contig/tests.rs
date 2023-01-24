#[cfg(test)]
use super::*;

#[test]
fn test_span_ord2() {
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
    let c = &mut Contig::new(10, 10u8);

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
fn test_contig_neighbourhood_enumerator() {
    fn enumerator_as_vec(c: &Contig<i32, u8>) -> Vec<Neighbourhood<i32, &u8>> {
        c.neighbourhood_enumerator()
            //.map(|nbh| (nbh.i, nbh.left.copied(), *nbh.this, nbh.right.copied()))
            .collect()
    }

    let c = &mut Contig::new(10, 10u8);

    c.set(11, 11u8);
    c.set(13, 13u8);

    assert_eq!(
        enumerator_as_vec(c),
        vec![
            Neighbourhood {
                i: 10,
                items: [None, Some(&10u8), Some(&11u8)]
            },
            Neighbourhood {
                i: 11,
                items: [Some(&10u8), Some(&11u8), None]
            },
            Neighbourhood {
                i: 13,
                items: [None, Some(&13u8), None]
            },
        ]
    );
}

#[test]
fn test_contig_neighbourhood_enumerator_get() {
    let c = Contig::from(vec![(10, 10u8), (11, 11u8), (13, 13u8)]).unwrap();

    let mut e0 = c.neighbourhood_enumerator();
    assert_eq!(e0.get(9), [None, None, Some(&10u8)]);
    assert_eq!(e0.get(10), [None, Some(&10u8), Some(&11u8)]);
    assert_eq!(e0.get(11), [Some(&10u8), Some(&11u8), None]);
    assert_eq!(e0.get(12), [Some(&11u8), None, Some(&13u8)]);
    assert_eq!(e0.get(13), [None, Some(&13u8), None]);
    assert_eq!(e0.get(14), [Some(&13u8), None, None]);

    let mut e1 = c.neighbourhood_enumerator_from(11);
    assert_eq!(e1.get(10), [None, Some(&10u8), Some(&11u8)]);
    assert_eq!(e1.get(11), [Some(&10u8), Some(&11u8), None]);
    assert_eq!(e1.get(12), [Some(&11), None, Some(&13)]);
    assert_eq!(e1.get(13), [None, Some(&13u8), None]);

    let mut e2 = c.neighbourhood_enumerator_from(12);
    assert_eq!(e2.get(10), [None, Some(&10u8), Some(&11u8)]);
    assert_eq!(e2.get(11), [Some(&10u8), Some(&11u8), None]);
    assert_eq!(e2.get(12), [Some(&11), None, Some(&13)]);
    assert_eq!(e2.get(13), [None, Some(&13u8), None]);
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
