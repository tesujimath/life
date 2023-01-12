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
    let oc = &mut OrderedContigs::new();

    oc.set(10, 10u8);
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
    let oc = &mut OrderedContigs::new();

    oc.set(10, 10u8);
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
fn test_ordered_contigs_neighbourhood_iter() {
    fn vec_from(oc: &OrderedContigs<i32, u8>, i: i32) -> Vec<(i32, Option<u8>, u8, Option<u8>)> {
        oc.neighbourhood_enumerator(i)
            .map(|nbh| (nbh.i, nbh.left.copied(), *nbh.this, nbh.right.copied()))
            .collect::<Vec<(i32, Option<u8>, u8, Option<u8>)>>()
    }

    let oc = &mut OrderedContigs::new();

    oc.set(10, 10u8);
    oc.set(11, 11u8);
    oc.set(13, 13u8);

    assert_eq!(
        vec_from(oc, 0),
        vec![
            (10, None, 10u8, Some(11u8)),
            (11, Some(10u8), 11u8, None),
            (13, None, 13u8, None),
        ]
    );
}

#[test]
fn test_cartesian_contigs_set() {
    let mut c = CartesianContigs::new();
    c.set(1, 1, 11u8);
    c.set(4, 2, 42u8);

    assert_eq!(c.get(1, 1), Some(&11u8));
    assert_eq!(c.get(1, 2), None);
    assert_eq!(c.get(4, 1), None);
    assert_eq!(c.get(4, 2), Some(&42u8));
}
