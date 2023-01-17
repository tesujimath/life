#[cfg(test)]
use super::*;

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

// #[test]
// fn test_bits_to_bytes() {
//     use std::vec::IntoIter;

//     let rows: Vec<Vec<u8>> = vec![
//         vec![0b00111011, 0b11110001],
//         vec![0b10101010, 0b10001001],
//         vec![0b00001000, 0b10000001],
//     ];

//     let mut double_row = rows.chunks(2);

//     let r0 = double_row.next().unwrap();
//     let p = PairwiseOrDefault::<IntoIter<u8>>::from(&r0);

//     // assert_eq!(
//     //     p.collect::<Vec<(u8, u8)>>(),
//     //     vec![(0b00111011, 0b10101010), (0b11110001, 0b10001001),]
//     // );

//     // let p = PairwiseOrDefault::<IntoIter<u8>>::from(double_row.next());
//     // assert_eq!(
//     //     p.collect::<Vec<(u8, u8)>>(),
//     //     vec![(0b00001000, 0), (0b10000001, 0)]
//     // );
// }
