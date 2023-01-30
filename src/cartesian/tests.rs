#![cfg(test)]
use super::*;

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
    cc.set(2, 3, 22u8);

    assert_eq!(
        enumerator_as_vec(&cc),
        vec![
            CartesianNeighbourhood {
                i_row: -1,
                i_col: -1,
                items: [
                    [None, None, None],
                    [None, None, None],
                    [None, None, Some(&0)],
                ]
            },
            CartesianNeighbourhood {
                i_row: -1,
                i_col: 0,
                items: [
                    [None, None, None],
                    [None, None, None],
                    [None, Some(&0), Some(&1)],
                ]
            },
            CartesianNeighbourhood {
                i_row: -1,
                i_col: 1,
                items: [
                    [None, None, None],
                    [None, None, None],
                    [Some(&0), Some(&1), Some(&2)],
                ]
            },
            CartesianNeighbourhood {
                i_row: -1,
                i_col: 2,
                items: [
                    [None, None, None],
                    [None, None, None],
                    [Some(&1), Some(&2), None],
                ]
            },
            CartesianNeighbourhood {
                i_row: -1,
                i_col: 3,
                items: [
                    [None, None, None],
                    [None, None, None],
                    [Some(&2), None, None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 0,
                i_col: -1,
                items: [
                    [None, None, None],
                    [None, None, Some(&0)],
                    [None, None, None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 0,
                i_col: 0,
                items: [
                    [None, None, None],
                    [None, Some(&0), Some(&1)],
                    [None, None, Some(&11)],
                ]
            },
            CartesianNeighbourhood {
                i_row: 0,
                i_col: 1,
                items: [
                    [None, None, None],
                    [Some(&0), Some(&1), Some(&2)],
                    [None, Some(&11), None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 0,
                i_col: 2,
                items: [
                    [None, None, None],
                    [Some(&1), Some(&2), None],
                    [Some(&11), None, None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 0,
                i_col: 3,
                items: [
                    [None, None, None],
                    [Some(&2), None, None],
                    [None, None, None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 1,
                i_col: 0,
                items: [
                    [None, Some(&0), Some(&1)],
                    [None, None, Some(&11)],
                    [None, None, None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 1,
                i_col: 1,
                items: [
                    [Some(&0), Some(&1), Some(&2)],
                    [None, Some(&11), None],
                    [None, None, None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 1,
                i_col: 2,
                items: [
                    [Some(&1), Some(&2), None],
                    [Some(&11), None, None],
                    [None, None, None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 2,
                i_col: 0,
                items: [
                    [None, None, Some(&11)],
                    [None, None, None],
                    [None, None, None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 2,
                i_col: 1,
                items: [
                    [None, Some(&11), None],
                    [None, None, None],
                    [None, None, Some(&22)],
                ]
            },
            CartesianNeighbourhood {
                i_row: 2,
                i_col: 2,
                items: [
                    [Some(&11), None, None],
                    [None, None, None],
                    [None, Some(&22), None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 2,
                i_col: 3,
                items: [
                    [None, None, None],
                    [None, None, None],
                    [Some(&22), None, None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 3,
                i_col: 1,
                items: [
                    [None, None, None],
                    [None, None, Some(&22)],
                    [None, None, None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 3,
                i_col: 2,
                items: [
                    [None, None, None],
                    [None, Some(&22), None],
                    [None, None, None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 3,
                i_col: 3,
                items: [
                    [None, None, None],
                    [Some(&22), None, None],
                    [None, None, None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 4,
                i_col: 1,
                items: [
                    [None, None, Some(&22)],
                    [None, None, None],
                    [None, None, None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 4,
                i_col: 2,
                items: [
                    [None, Some(&22), None],
                    [None, None, None],
                    [None, None, None],
                ]
            },
            CartesianNeighbourhood {
                i_row: 4,
                i_col: 3,
                items: [
                    [Some(&22), None, None],
                    [None, None, None],
                    [None, None, None],
                ]
            },
        ]
    );
}
