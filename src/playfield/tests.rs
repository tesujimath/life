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
