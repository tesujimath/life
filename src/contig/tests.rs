#[cfg(test)]
use super::*;

#[test]
fn test_contig_ord2() {
    assert_eq!(
        Contig {
            origin: 1,
            items: VecDeque::<u8>::new()
        }
        .cmp(&0),
        Ordering::Greater
    );

    assert_eq!(
        Contig {
            origin: 1,
            items: VecDeque::<u8>::new()
        }
        .cmp(&1),
        Ordering::Less
    );

    assert_eq!(
        Contig {
            origin: 1,
            items: VecDeque::<u8>::new()
        }
        .cmp(&2),
        Ordering::Less
    );

    assert_eq!(
        Contig {
            origin: 1,
            items: VecDeque::from(vec![7u8])
        }
        .cmp(&0),
        Ordering::Greater
    );

    assert_eq!(
        Contig {
            origin: 1,
            items: VecDeque::from(vec![7u8])
        }
        .cmp(&1),
        Ordering::Equal
    );

    assert_eq!(
        Contig {
            origin: 1,
            items: VecDeque::from(vec![7u8])
        }
        .cmp(&2),
        Ordering::Less
    );

    assert_eq!(
        Contig {
            origin: 1,
            items: VecDeque::from(vec![5u8, 7u8])
        }
        .cmp(&2),
        Ordering::Equal
    );
}

#[test]
fn test_contigs_binary_search() {
    let contigs = VecDeque::from(vec![
        Contig {
            origin: 0,
            items: VecDeque::from(vec![5u8, 7u8]),
        },
        Contig {
            origin: 3,
            items: VecDeque::from(vec![3u8]),
        },
    ]);

    assert_eq!(contigs.binary_search_by(|span| span.cmp(&-1)), Err(0));
    assert_eq!(contigs.binary_search_by(|span| span.cmp(&1)), Ok(0));
    assert_eq!(contigs.binary_search_by(|span| span.cmp(&2)), Err(1));
    assert_eq!(contigs.binary_search_by(|span| span.cmp(&3)), Ok(1));
    assert_eq!(contigs.binary_search_by(|span| span.cmp(&4)), Err(2));
}

#[test]
fn test_ordered_contigs_set() {
    let oc = &mut OrderedContigs::new(10, 10u8);
    assert_eq!(
        *oc,
        OrderedContigs {
            contigs: VecDeque::from(vec![Contig {
                origin: 10,
                items: VecDeque::from(vec![10u8])
            },])
        }
    );

    oc.set(11, 11u8);
    assert_eq!(
        *oc,
        OrderedContigs {
            contigs: VecDeque::from(vec![Contig {
                origin: 10,
                items: VecDeque::from(vec![10u8, 11u8])
            },])
        }
    );

    oc.set(9, 9u8);
    assert_eq!(
        *oc,
        OrderedContigs {
            contigs: VecDeque::from(vec![Contig {
                origin: 9,
                items: VecDeque::from(vec![9u8, 10u8, 11u8])
            }])
        }
    );

    oc.set(7, 7u8);
    assert_eq!(
        *oc,
        OrderedContigs {
            contigs: VecDeque::from(vec![
                Contig {
                    origin: 7,
                    items: VecDeque::from(vec![7u8])
                },
                Contig {
                    origin: 9,
                    items: VecDeque::from(vec![9u8, 10u8, 11u8])
                }
            ])
        }
    );

    oc.set(8, 8u8);
    assert_eq!(
        *oc,
        OrderedContigs {
            contigs: VecDeque::from(vec![Contig {
                origin: 7,
                items: VecDeque::from(vec![7u8, 8u8, 9u8, 10u8, 11u8])
            }])
        }
    );
}

#[test]
fn test_ordered_contigs_get() {
    let oc = &mut OrderedContigs::new(10, 10u8);

    oc.set(11, 11u8);
    oc.set(13, 13u8);

    assert_eq!(oc.get(9), None);
    assert_eq!(oc.get(10), Some(&10u8));
    assert_eq!(oc.get(11), Some(&11u8));
    assert_eq!(oc.get(12), None);
    assert_eq!(oc.get(13), Some(&13u8));
    assert_eq!(oc.get(14), None);
}

#[test]
fn test_ordered_contigs_enumerator() {
    fn enumerator_as_vec(oc: &OrderedContigs<i32, u8>) -> Vec<(i32, Option<u8>, u8, Option<u8>)> {
        oc.neighbourhood_enumerator()
            .map(|nbh| (nbh.i, nbh.left.copied(), *nbh.this, nbh.right.copied()))
            .collect::<Vec<(i32, Option<u8>, u8, Option<u8>)>>()
    }

    let oc = &mut OrderedContigs::new(10, 10u8);

    oc.set(11, 11u8);
    oc.set(13, 13u8);

    assert_eq!(
        enumerator_as_vec(oc),
        vec![
            (10, None, 10u8, Some(11u8)),
            (11, Some(10u8), 11u8, None),
            (13, None, 13u8, None),
        ]
    );
}

#[test]
fn test_ordered_contigs_enumerator_get() {
    let oc = OrderedContigs::from(vec![(10, 10u8), (11, 11u8), (13, 13u8)]).unwrap();

    let mut e0 = oc.neighbourhood_enumerator();
    assert_eq!(e0.get(9), (None, None, Some(&10u8)));
    assert_eq!(e0.get(10), (None, Some(&10u8), Some(&11u8)));
    assert_eq!(e0.get(11), (Some(&10u8), Some(&11u8), None));
    assert_eq!(e0.get(12), (Some(&11u8), None, Some(&13u8)));
    assert_eq!(e0.get(13), (None, Some(&13u8), None));
    assert_eq!(e0.get(14), (Some(&13u8), None, None));

    let mut e1 = oc.neighbourhood_enumerator_from(11);
    assert_eq!(e1.get(10), (None, Some(&10u8), Some(&11u8)));
    assert_eq!(e1.get(11), (Some(&10u8), Some(&11u8), None));
    assert_eq!(e1.get(12), (Some(&11), None, Some(&13)));
    assert_eq!(e1.get(13), (None, Some(&13u8), None));

    let mut e2 = oc.neighbourhood_enumerator_from(12);
    assert_eq!(e2.get(10), (None, Some(&10u8), Some(&11u8)));
    assert_eq!(e2.get(11), (Some(&10u8), Some(&11u8), None));
    assert_eq!(e2.get(12), (Some(&11), None, Some(&13)));
    assert_eq!(e2.get(13), (None, Some(&13u8), None));
}

#[test]
fn test_cartesian_contigs_set() {
    let mut c = CartesianContigs::new(0, 0, 0u8);
    c.set(1, 1, 11u8);
    c.set(4, 2, 42u8);

    assert_eq!(c.get(1, 1), Some(&11u8));
    assert_eq!(c.get(1, 2), None);
    assert_eq!(c.get(4, 1), None);
    assert_eq!(c.get(4, 2), Some(&42u8));
}

#[test]
fn test_cartesian_contigs_enumerator() {
    fn enumerator_as_vec(
        cc: &CartesianContigs<i32, u8>,
    ) -> Vec<(
        i32,
        i32,
        Option<u8>,
        Option<u8>,
        Option<u8>,
        Option<u8>,
        u8,
        Option<u8>,
        Option<u8>,
        Option<u8>,
        Option<u8>,
    )> {
        cc.neighbourhood_enumerator()
            .map(|row| {
                (
                    row.this.i,
                    row.i_row,
                    row.below.0.copied(),
                    row.below.1.copied(),
                    row.below.2.copied(),
                    row.this.left.copied(),
                    *row.this.this,
                    row.this.right.copied(),
                    row.above.0.copied(),
                    row.above.1.copied(),
                    row.above.2.copied(),
                )
            })
            .collect::<Vec<(
                i32,
                i32,
                Option<u8>,
                Option<u8>,
                Option<u8>,
                Option<u8>,
                u8,
                Option<u8>,
                Option<u8>,
                Option<u8>,
                Option<u8>,
            )>>()
    }

    let mut cc = CartesianContigs::new(0, 0, 0u8);
    cc.set(1, 0, 1u8);
    cc.set(2, 0, 2u8);
    cc.set(1, 1, 11u8);

    assert_eq!(
        enumerator_as_vec(&cc),
        vec![
            (
                0,
                0,
                None,
                None,
                None,
                None,
                0,
                Some(1),
                None,
                None,
                Some(11),
            ),
            (
                1,
                0,
                None,
                None,
                None,
                Some(0),
                1,
                Some(2),
                None,
                Some(11),
                None,
            ),
            (
                2,
                0,
                None,
                None,
                None,
                Some(1),
                2,
                None,
                Some(11),
                None,
                None,
            ),
            (
                1,
                1,
                Some(0),
                Some(1),
                Some(2),
                None,
                11,
                None,
                None,
                None,
                None,
            ),
        ]
    );
}
