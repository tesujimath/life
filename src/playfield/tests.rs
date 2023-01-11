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
fn test_ordered_contigs_get_or_default() {
    let oc = &mut OrderedContigs::new();

    oc.set(10, 10u8);
    oc.set(11, 11u8);
    oc.set(13, 13u8);

    assert_eq!(oc.get_or_default(9), 0u8);
    assert_eq!(oc.get_or_default(10), 10u8);
    assert_eq!(oc.get_or_default(11), 11u8);
    assert_eq!(oc.get_or_default(12), 0u8);
    assert_eq!(oc.get_or_default(13), 13u8);
    assert_eq!(oc.get_or_default(14), 0u8);
}
