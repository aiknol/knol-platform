//! Tests for the Reciprocal Rank Fusion algorithm.
//! NOTE: This tests the algorithm logic extracted from the service.

use std::collections::HashMap;
use uuid::Uuid;

/// RRF fusion: score(m) = Σ w_i / (k + rank_i(m))
fn rrf_fusion(ranked_lists: &[(&[Uuid], f64)], k: f64) -> Vec<(Uuid, f64)> {
    let mut scores: HashMap<Uuid, f64> = HashMap::new();
    for (ids, weight) in ranked_lists {
        for (rank, id) in ids.iter().enumerate() {
            *scores.entry(*id).or_default() += weight / (k + rank as f64 + 1.0);
        }
    }
    let mut sorted: Vec<(Uuid, f64)> = scores.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sorted
}

#[test]
fn test_rrf_single_list() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let ids = vec![id1, id2];
    let result = rrf_fusion(&[(&ids, 1.0)], 60.0);
    assert_eq!(result.len(), 2);
    // First item should score higher
    assert!(result[0].1 > result[1].1);
    assert_eq!(result[0].0, id1);
}

#[test]
fn test_rrf_multiple_lists_boost() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();

    // id2 appears in both lists, so should be boosted
    let list1 = vec![id1, id2, id3];
    let list2 = vec![id2, id3];

    let result = rrf_fusion(&[(&list1, 1.0), (&list2, 1.0)], 60.0);
    // id2 should score highest because it appears in both lists
    assert_eq!(result[0].0, id2);
}

#[test]
fn test_rrf_empty_lists() {
    let empty: Vec<Uuid> = vec![];
    let result = rrf_fusion(&[(&empty, 1.0)], 60.0);
    assert!(result.is_empty());
}

#[test]
fn test_rrf_weighted_lists() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    // id1 is first in high-weight list, id2 is first in low-weight list
    let list1 = vec![id1];
    let list2 = vec![id2];

    let result = rrf_fusion(&[(&list1, 10.0), (&list2, 0.1)], 60.0);
    assert_eq!(result[0].0, id1); // Higher weight list wins
    assert!(result[0].1 > result[1].1 * 10.0); // Score ratio reflects weight ratio
}

#[test]
fn test_rrf_k_parameter_effect() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let ids = vec![id1, id2];

    // Small k = more spread between ranks
    let result_small_k = rrf_fusion(&[(&ids, 1.0)], 1.0);
    let spread_small = result_small_k[0].1 - result_small_k[1].1;

    // Large k = less spread between ranks
    let result_large_k = rrf_fusion(&[(&ids, 1.0)], 1000.0);
    let spread_large = result_large_k[0].1 - result_large_k[1].1;

    assert!(spread_small > spread_large);
}
