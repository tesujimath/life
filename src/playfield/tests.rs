#[cfg(test)]
use super::*;
use std::vec::IntoIter;

#[test]
fn test_pairwise_or_default() {
    assert_eq!(
        PairwiseOrDefault::<IntoIter<u8>>::from(vec![vec![1u8, 2u8, 3u8], vec![11u8]])
            .collect::<Vec<(u8, u8)>>(),
        vec![(1u8, 11u8), (2u8, 0u8), (3u8, 0u8)]
    );

    assert_eq!(
        PairwiseOrDefault::<IntoIter<u8>>::from(vec![vec![1u8, 2u8, 3u8]])
            .collect::<Vec<(u8, u8)>>(),
        vec![(1u8, 0u8), (2u8, 0u8), (3u8, 0u8)]
    );
}
