#[cfg(test)]
use super::*;

#[test]
fn test_span_ord2() {
    assert_eq!(
        Span {
            left: 1,
            cells: VecDeque::from(vec![])
        }
        .cmp(&0),
        Ordering::Greater
    );

    assert_eq!(
        Span {
            left: 1,
            cells: VecDeque::from(vec![])
        }
        .cmp(&1),
        Ordering::Less
    );

    assert_eq!(
        Span {
            left: 1,
            cells: VecDeque::from(vec![])
        }
        .cmp(&2),
        Ordering::Less
    );

    assert_eq!(
        Span {
            left: 1,
            cells: VecDeque::from(vec![7u8])
        }
        .cmp(&0),
        Ordering::Greater
    );

    assert_eq!(
        Span {
            left: 1,
            cells: VecDeque::from(vec![7u8])
        }
        .cmp(&1),
        Ordering::Equal
    );

    assert_eq!(
        Span {
            left: 1,
            cells: VecDeque::from(vec![7u8])
        }
        .cmp(&2),
        Ordering::Less
    );

    assert_eq!(
        Span {
            left: 1,
            cells: VecDeque::from(vec![5u8, 7u8])
        }
        .cmp(&2),
        Ordering::Equal
    );
}

#[test]
fn test_spans_binary_search() {
    let spans = VecDeque::from(vec![
        Span {
            left: 0,
            cells: VecDeque::from(vec![5u8, 7u8]),
        },
        Span {
            left: 3,
            cells: VecDeque::from(vec![3u8]),
        },
    ]);

    assert_eq!(spans.binary_search_by(|span| span.cmp(&-1)), Err(0));
    assert_eq!(spans.binary_search_by(|span| span.cmp(&1)), Ok(0));
    assert_eq!(spans.binary_search_by(|span| span.cmp(&2)), Err(1));
    assert_eq!(spans.binary_search_by(|span| span.cmp(&3)), Ok(1));
    assert_eq!(spans.binary_search_by(|span| span.cmp(&4)), Err(2));
}

#[test]
fn test_row_set() {
    let row = &mut Row::new();

    row.set(10, 10u8);
    assert_eq!(
        *row,
        Row {
            spans: VecDeque::from(vec![Span {
                left: 10,
                cells: VecDeque::from(vec![10u8])
            },])
        }
    );

    row.set(11, 11u8);
    assert_eq!(
        *row,
        Row {
            spans: VecDeque::from(vec![Span {
                left: 10,
                cells: VecDeque::from(vec![10u8, 11u8])
            },])
        }
    );

    row.set(9, 9u8);
    assert_eq!(
        *row,
        Row {
            spans: VecDeque::from(vec![Span {
                left: 9,
                cells: VecDeque::from(vec![9u8, 10u8, 11u8])
            }])
        }
    );

    row.set(7, 7u8);
    assert_eq!(
        *row,
        Row {
            spans: VecDeque::from(vec![
                Span {
                    left: 7,
                    cells: VecDeque::from(vec![7u8])
                },
                Span {
                    left: 9,
                    cells: VecDeque::from(vec![9u8, 10u8, 11u8])
                }
            ])
        }
    );

    row.set(8, 8u8);
    assert_eq!(
        *row,
        Row {
            spans: VecDeque::from(vec![Span {
                left: 7,
                cells: VecDeque::from(vec![7u8, 8u8, 9u8, 10u8, 11u8])
            }])
        }
    );
}
