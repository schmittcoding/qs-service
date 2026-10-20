//! Unit tests for leaderboard route helper functions.
//!
//! Tests the `is_cache_stale()`, pagination, and filtering logic
//! used in the LevelCapRace leaderboard handler.

use chrono::NaiveDateTime;
use ran_models::events::level_cap_race_score::LevelCapRaceScore;
use std::time::Duration;
use tokio::time::Instant;
use uuid::Uuid;

// ============================================================================
// Helper function implementations (copied from handler for testing)
// ============================================================================

/// Check if cached data is stale
fn is_cache_stale(cached_at: Instant, ttl: Duration) -> bool {
    cached_at.elapsed() > ttl
}

/// Calculate pagination for in-memory results
fn apply_pagination(
    items: Vec<LevelCapRaceScore>,
    page: u32,
    page_size: u32,
) -> Vec<LevelCapRaceScore> {
    let offset = (page.saturating_sub(1)) * page_size;
    items
        .into_iter()
        .skip(offset as usize)
        .take(page_size as usize)
        .collect()
}

/// Apply class filter to scores
fn apply_class_filter(scores: Vec<LevelCapRaceScore>, cha_class: Option<i32>) -> Vec<LevelCapRaceScore> {
    scores
        .into_iter()
        .filter(|s| cha_class.is_none() || s.cha_class == cha_class.unwrap())
        .collect()
}

fn create_test_score(rank: i64, cha_class: i32) -> LevelCapRaceScore {
    LevelCapRaceScore {
        rank,
        event_id: Uuid::nil(),
        cha_num: 1000 + rank as i32,
        cha_name: format!("Char{}", rank),
        cha_class,
        cha_school: None,
        guild_num: None,
        guild_name: None,
        start_level: 100,
        current_level: 100 + rank as i32,
        levels_gained: rank as i32,
        level_up_date: None,
    }
}

// ============================================================================
// Tests: is_cache_stale()
// ============================================================================

#[test]
fn test_is_cache_stale_false_within_ttl() {
    let ttl = Duration::from_secs(30);
    let cached_at = Instant::now();
    assert!(!is_cache_stale(cached_at, ttl));
}

#[test]
fn test_is_cache_stale_true_after_ttl() {
    let ttl = Duration::from_millis(100);
    let cached_at = Instant::now() - Duration::from_millis(150);
    assert!(is_cache_stale(cached_at, ttl));
}

#[test]
fn test_is_cache_stale_at_exact_ttl_boundary() {
    let ttl = Duration::from_secs(1);
    let cached_at = Instant::now() - Duration::from_secs(1) - Duration::from_millis(1);
    assert!(is_cache_stale(cached_at, ttl));
}

// ============================================================================
// Tests: apply_pagination()
// ============================================================================

#[test]
fn test_apply_pagination_page_1() {
    let scores: Vec<_> = (1..=50).map(|i| create_test_score(i, 1)).collect();
    let paginated = apply_pagination(scores, 1, 10);
    assert_eq!(paginated.len(), 10);
    assert_eq!(paginated[0].rank, 1);
    assert_eq!(paginated[9].rank, 10);
}

#[test]
fn test_apply_pagination_page_5() {
    let scores: Vec<_> = (1..=100).map(|i| create_test_score(i, 1)).collect();
    let paginated = apply_pagination(scores, 5, 10);
    assert_eq!(paginated.len(), 10);
    assert_eq!(paginated[0].rank, 41);
    assert_eq!(paginated[9].rank, 50);
}

#[test]
fn test_apply_pagination_partial_last_page() {
    let scores: Vec<_> = (1..=35).map(|i| create_test_score(i, 1)).collect();
    let paginated = apply_pagination(scores, 4, 10);
    assert_eq!(paginated.len(), 5);
    assert_eq!(paginated[0].rank, 31);
}

#[test]
fn test_apply_pagination_beyond_range() {
    let scores: Vec<_> = (1..=20).map(|i| create_test_score(i, 1)).collect();
    let paginated = apply_pagination(scores, 10, 10);
    assert!(paginated.is_empty());
}

#[test]
fn test_apply_pagination_page_zero_treated_as_page_one() {
    let scores: Vec<_> = (1..=20).map(|i| create_test_score(i, 1)).collect();
    let paginated = apply_pagination(scores, 0, 10);
    assert_eq!(paginated.len(), 10);
    assert_eq!(paginated[0].rank, 1);
}

#[test]
fn test_apply_pagination_page_size_zero() {
    let scores: Vec<_> = (1..=20).map(|i| create_test_score(i, 1)).collect();
    let paginated = apply_pagination(scores, 1, 0);
    assert!(paginated.is_empty());
}

#[test]
fn test_apply_pagination_empty_input() {
    let scores = vec![];
    let paginated = apply_pagination(scores, 1, 10);
    assert!(paginated.is_empty());
}

// ============================================================================
// Tests: apply_class_filter()
// ============================================================================

#[test]
fn test_apply_class_filter_single_class() {
    let scores: Vec<_> = (1..=30)
        .map(|i| create_test_score(i, ((i % 3) + 1) as i32))
        .collect();

    let filtered = apply_class_filter(scores, Some(1));
    for score in &filtered {
        assert_eq!(score.cha_class, 1);
    }
    assert!(!filtered.is_empty());
}

#[test]
fn test_apply_class_filter_none_returns_all() {
    let scores: Vec<_> = (1..=20)
        .map(|i| create_test_score(i, ((i % 3) + 1) as i32))
        .collect();

    let original_count = scores.len();
    let filtered = apply_class_filter(scores, None);
    assert_eq!(filtered.len(), original_count);
}

#[test]
fn test_apply_class_filter_no_matches() {
    let scores: Vec<_> = (1..=10).map(|i| create_test_score(i, 1)).collect();
    let filtered = apply_class_filter(scores, Some(99));
    assert!(filtered.is_empty());
}

#[test]
fn test_apply_class_filter_all_classes() {
    let scores: Vec<_> = (1..=30)
        .map(|i| create_test_score(i, ((i % 3) + 1) as i32))
        .collect();

    let class_1 = apply_class_filter(scores.clone(), Some(1));
    let class_2 = apply_class_filter(scores.clone(), Some(2));
    let class_3 = apply_class_filter(scores.clone(), Some(3));

    assert!(!class_1.is_empty());
    assert!(!class_2.is_empty());
    assert!(!class_3.is_empty());

    for score in &class_1 {
        assert_eq!(score.cha_class, 1);
    }
    for score in &class_2 {
        assert_eq!(score.cha_class, 2);
    }
    for score in &class_3 {
        assert_eq!(score.cha_class, 3);
    }
}

// ============================================================================
// Tests: Combined operations
// ============================================================================

#[test]
fn test_filter_then_paginate() {
    let scores: Vec<_> = (1..=100)
        .map(|i| create_test_score(i, ((i % 3) + 1) as i32))
        .collect();

    let filtered = apply_class_filter(scores, Some(1));
    let paginated = apply_pagination(filtered, 1, 10);

    for score in &paginated {
        assert_eq!(score.cha_class, 1);
    }
    assert!(paginated.len() <= 10);
}

#[test]
fn test_filter_with_no_results_then_paginate() {
    let scores: Vec<_> = (1..=20)
        .map(|i| create_test_score(i, ((i % 2) + 1) as i32))
        .collect();

    let filtered = apply_class_filter(scores, Some(99));
    let paginated = apply_pagination(filtered, 1, 10);

    assert!(paginated.is_empty());
}

// ============================================================================
// Tests: Level-up date field preservation
// ============================================================================

#[test]
fn test_pagination_preserves_level_up_date() {
    let event_id = Uuid::new_v4();
    let mut scores = vec![];
    for i in 1..=10 {
        let level_up_date = if i % 2 == 0 {
            Some(NaiveDateTime::from_timestamp_opt(1609459200 + (i as i64 * 3600), 0).unwrap())
        } else {
            None
        };

        scores.push(LevelCapRaceScore {
            rank: i,
            event_id,
            cha_num: 1000 + i as i32,
            cha_name: format!("Char{}", i),
            cha_class: 1,
            cha_school: None,
            guild_num: None,
            guild_name: None,
            start_level: 100,
            current_level: 100 + i as i32,
            levels_gained: i as i32,
            level_up_date,
        });
    }

    let paginated = apply_pagination(scores.clone(), 1, 10);
    assert_eq!(paginated.len(), 10);

    for (original, paginated) in scores.iter().zip(paginated.iter()) {
        assert_eq!(original.level_up_date, paginated.level_up_date);
    }
}

#[test]
fn test_class_filter_preserves_level_up_date() {
    let event_id = Uuid::new_v4();
    let mut scores = vec![];
    for i in 1..=20 {
        let level_up_date = if i % 2 == 0 {
            Some(NaiveDateTime::from_timestamp_opt(1609459200 + (i as i64 * 3600), 0).unwrap())
        } else {
            None
        };

        scores.push(LevelCapRaceScore {
            rank: i,
            event_id,
            cha_num: 1000 + i as i32,
            cha_name: format!("Char{}", i),
            cha_class: ((i % 2) + 1) as i32,
            cha_school: None,
            guild_num: None,
            guild_name: None,
            start_level: 100,
            current_level: 100 + i as i32,
            levels_gained: i as i32,
            level_up_date,
        });
    }

    let filtered = apply_class_filter(scores.clone(), Some(1));

    // Verify level_up_date is preserved
    for filtered_score in &filtered {
        let original = scores
            .iter()
            .find(|s| s.rank == filtered_score.rank)
            .unwrap();
        assert_eq!(original.level_up_date, filtered_score.level_up_date);
    }
}
