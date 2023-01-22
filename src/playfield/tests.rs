use super::*;
#[cfg(test)]
use assert_hex::assert_eq_hex;

#[test]
fn test_pairwise_or_default() {
    use std::vec::IntoIter;

    let r0 = vec![vec![1u8, 2u8, 3u8], vec![11u8]];

    let vec_ref = PairwiseOrDefault::<IntoIter<u8>>::from(&r0);
    assert_eq!(
        vec_ref.collect::<Vec<(u8, u8)>>(),
        vec![(1u8, 11u8), (2u8, 0u8), (3u8, 0u8)]
    );

    let p0 = PairwiseOrDefault::<IntoIter<u8>>::from(&r0);
    assert_eq!(
        p0.collect::<Vec<(u8, u8)>>(),
        vec![(1u8, 11u8), (2u8, 0u8), (3u8, 0u8)]
    );

    let r1 = vec![vec![1u8, 2u8, 3u8]];
    let p1 = PairwiseOrDefault::<IntoIter<u8>>::from(&r1);
    assert_eq!(
        p1.collect::<Vec<(u8, u8)>>(),
        vec![(1u8, 0u8), (2u8, 0u8), (3u8, 0u8)]
    );
}

#[test]
fn test_chunks() {
    use std::vec::IntoIter;

    let rows: Vec<Vec<u8>> = vec![vec![1, 2], vec![3, 4], vec![5, 6]];

    let mut double_row = rows.chunks(2);

    let r0 = double_row.next().unwrap();
    let p0 = PairwiseOrDefault::<IntoIter<u8>>::from(r0);
    assert_eq!(p0.collect::<Vec<(u8, u8)>>(), vec![(1, 3), (2, 4),]);

    let r1 = double_row.next().unwrap();
    let p1 = PairwiseOrDefault::<IntoIter<u8>>::from(r1);
    assert_eq!(p1.collect::<Vec<(u8, u8)>>(), vec![(5, 0), (6, 0)]);
}

#[test]
fn test_bits_to_bytes() {
    use std::vec::IntoIter;

    let rows: Vec<Vec<u8>> = vec![
        vec![0b00111011, 0b11110001],
        vec![0b10101010, 0b10001001],
        vec![0b00001000, 0b10000001],
    ];

    let mut double_row = rows.chunks(2);

    let r0 = double_row.next().unwrap();
    let p0 = PairwiseOrDefault::<IntoIter<u8>>::from(r0);
    assert_eq!(
        p0.collect::<Vec<(u8, u8)>>(),
        vec![(0b00111011, 0b10101010), (0b11110001, 0b10001001),]
    );

    let r1 = double_row.next().unwrap();
    let p1 = PairwiseOrDefault::<IntoIter<u8>>::from(r1);
    assert_eq!(
        p1.collect::<Vec<(u8, u8)>>(),
        vec![(0b00001000, 0), (0b10000001, 0)]
    );
}

#[test]
fn test_pack_unpack_roundtrip() {
    type P = Playfield<i32, u16>;

    assert_eq_hex!(P::pack((3, 4)), 0x0403);
    assert_eq_hex!(P::unpack(0x0605), (5, 6));

    assert_eq_hex!(P::unpack(P::pack((1, 2))), (1, 2));
}

#[test]
fn test_to_from_bytes_roundtrip() {
    let bytes0: Vec<Vec<u8>> = vec![vec![0x01, 0x02], vec![0x13, 0x14], vec![0x25, 0x26]];
    let origin0 = Coordinate { x: 0, y: 0 };

    // roundtrip
    let p = Playfield::<i32, u16>::from(&bytes0, origin0);
    let (bytes1, origin1) = p.to_rows_of_bytes();

    // always comes back as even number of rows
    let mut expected = bytes0.clone();
    expected.push(vec![0, 0]);

    assert_eq_hex!(origin1, origin0);
    assert_eq_hex!(bytes1, expected);
}
