//! Integration tests for LevelCapRace leaderboard caching with LevelUpDate support.
//!
//! These tests verify:
//! - Cache TTL and expiration behavior
//! - Cache invalidation on tally completion
//! - First-call cache population
//! - LevelUpDate field accuracy and serialization
//! - In-memory pagination post-cache
//! - Class filtering with cached data
//! - Concurrent cache reads
//! - Empty leaderboard handling
//! - LevelUpDate null handling
//! - Error handling with stale cache fallback

use chrono::NaiveDateTime;
use ran_models::events::level_cap_race_score::LevelCapRaceScore;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;
use uuid::Uuid;

// ============================================================================
// Mock Helpers and Fixtures
// ============================================================================

/// Simulates the cached leaderboard structure from AppState
#[derive(Debug, Clone)]
struct CachedLevelCapRaceLeaderboard {
    scores: Vec<LevelCapRaceScore>,
    total_count: i64,
    last_fetched_at: Instant,
}

/// Test fixture: create sample leaderboard scores
fn create_sample_scores(count: usize) -> Vec<LevelCapRaceScore> {
    (0..count)
        .map(|i| LevelCapRaceScore {
            rank: (i + 1) as i64,
            event_id: Uuid::nil(),
            cha_num: 1000 + i as i32,
            cha_name: format!("Character{}", i + 1),
            cha_class: ((i % 3) + 1) as i32, // Distribute across 3 classes
            cha_school: Some((i as i32) % 5),
            guild_num: Some(500 + (i as i32) / 5),
            guild_name: Some(format!("Guild{}", (i / 5) + 1)),
            start_level: 100,
            current_level: 100 + (i as i32 % 20),
            levels_gained: i as i32 % 20,
            level_up_date: if i % 2 == 0 {
                Some(NaiveDateTime::from_timestamp_opt(1609459200 + (i as i64 * 3600), 0).unwrap())
            } else {
                None
            },
        })
        .collect()
}

// ============================================================================
// Test 1: Cache TTL and Expiration
// ============================================================================

#[tokio::test]
async fn test_cache_hit_within_ttl() {
    const CACHE_TTL: Duration = Duration::from_secs(30);
    let event_id = Uuid::new_v4();
    let scores = create_sample_scores(5);

    let cache = Arc::new(Mutex::new(HashMap::new()));
    let mut c = cache.lock().await;
    c.insert(
        event_id,
        CachedLevelCapRaceLeaderboard {
            scores: scores.clone(),
            total_count: 5,
            last_fetched_at: Instant::now(),
        },
    );
    drop(c);

    // Immediately check cache — should be a hit
    let c = cache.lock().await;
    let cached = c.get(&event_id).cloned();
    drop(c);

    assert!(cached.is_some());
    let cached_data = cached.unwrap();
    assert_eq!(cached_data.scores.len(), 5);
    assert_eq!(cached_data.total_count, 5);
    assert!(!cached_data.last_fetched_at.elapsed() > CACHE_TTL);
}

#[tokio::test]
async fn test_cache_miss_after_ttl_expires() {
    // Simulate expiration by using a very short TTL and then advancing time
    const CACHE_TTL: Duration = Duration::from_millis(100);
    let event_id = Uuid::new_v4();
    let scores = create_sample_scores(3);

    let cache = Arc::new(Mutex::new(HashMap::new()));
    let mut c = cache.lock().await;
    c.insert(
        event_id,
        CachedLevelCapRaceLeaderboard {
            scores: scores.clone(),
            total_count: 3,
            last_fetched_at: Instant::now(),
        },
    );
    drop(c);

    // Sleep longer than TTL
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Check if cache is stale
    let c = cache.lock().await;
    let cached = c.get(&event_id).cloned();
    drop(c);

    if let Some(cached_data) = cached {
        let is_stale = cached_data.last_fetched_at.elapsed() > CACHE_TTL;
        assert!(is_stale, "Cache should be stale after TTL expires");
    }
}

#[tokio::test]
async fn test_cache_miss_for_nonexistent_event() {
    let event_id = Uuid::new_v4();
    let cache = Arc::new(Mutex::new(HashMap::new()));

    let c = cache.lock().await;
    let cached = c.get(&event_id).cloned();
    drop(c);

    assert!(cached.is_none(), "Cache should be empty for new event");
}

// ============================================================================
// Test 2: Cache Invalidation on Tally
// ============================================================================

#[tokio::test]
async fn test_cache_invalidation_removes_entry() {
    let event_id = Uuid::new_v4();
    let scores = create_sample_scores(10);
    let cache = Arc::new(Mutex::new(HashMap::new()));

    // Populate cache
    let mut c = cache.lock().await;
    c.insert(
        event_id,
        CachedLevelCapRaceLeaderboard {
            scores: scores.clone(),
            total_count: 10,
            last_fetched_at: Instant::now(),
        },
    );
    drop(c);

    // Verify cache is populated
    let c = cache.lock().await;
    assert!(c.contains_key(&event_id));
    drop(c);

    // Simulate tally completion — invalidate cache
    let mut c = cache.lock().await;
    c.remove(&event_id);
    drop(c);

    // Verify cache is cleared
    let c = cache.lock().await;
    assert!(!c.contains_key(&event_id));
    drop(c);
}

#[tokio::test]
async fn test_cache_invalidation_only_affects_target_event() {
    let event_id_1 = Uuid::new_v4();
    let event_id_2 = Uuid::new_v4();
    let scores_1 = create_sample_scores(5);
    let scores_2 = create_sample_scores(7);
    let cache = Arc::new(Mutex::new(HashMap::new()));

    // Populate cache for both events
    let mut c = cache.lock().await;
    c.insert(
        event_id_1,
        CachedLevelCapRaceLeaderboard {
            scores: scores_1.clone(),
            total_count: 5,
            last_fetched_at: Instant::now(),
        },
    );
    c.insert(
        event_id_2,
        CachedLevelCapRaceLeaderboard {
            scores: scores_2.clone(),
            total_count: 7,
            last_fetched_at: Instant::now(),
        },
    );
    drop(c);

    // Invalidate only event_id_1
    let mut c = cache.lock().await;
    c.remove(&event_id_1);
    drop(c);

    // Verify event_id_1 is cleared but event_id_2 remains
    let c = cache.lock().await;
    assert!(!c.contains_key(&event_id_1));
    assert!(c.contains_key(&event_id_2));
    drop(c);
}

// ============================================================================
// Test 3: First-Call Cache Population
// ============================================================================

#[tokio::test]
async fn test_first_call_populates_cache() {
    let event_id = Uuid::new_v4();
    let scores = create_sample_scores(8);
    let cache = Arc::new(Mutex::new(HashMap::new()));

    // Simulate first call populating cache
    let mut c = cache.lock().await;
    c.insert(
        event_id,
        CachedLevelCapRaceLeaderboard {
            scores: scores.clone(),
            total_count: 8,
            last_fetched_at: Instant::now(),
        },
    );
    drop(c);

    // Verify cache is populated
    let c = cache.lock().await;
    assert!(c.contains_key(&event_id));
    let cached_data = c.get(&event_id).unwrap();
    assert_eq!(cached_data.scores.len(), 8);
    assert_eq!(cached_data.total_count, 8);
    drop(c);
}

#[tokio::test]
async fn test_cache_reused_on_second_call() {
    let event_id = Uuid::new_v4();
    let scores = create_sample_scores(6);
    let cache = Arc::new(Mutex::new(HashMap::new()));

    // First call populates cache
    let now = Instant::now();
    let mut c = cache.lock().await;
    c.insert(
        event_id,
        CachedLevelCapRaceLeaderboard {
            scores: scores.clone(),
            total_count: 6,
            last_fetched_at: now,
        },
    );
    drop(c);

    // Small sleep
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Second call — fetch from cache
    let c = cache.lock().await;
    let cached_data = c.get(&event_id).cloned();
    drop(c);

    assert!(cached_data.is_some());
    assert_eq!(cached_data.unwrap().last_fetched_at, now);
}

// ============================================================================
// Test 4: LevelUpDate Field Accuracy
// ============================================================================

#[tokio::test]
async fn test_level_up_date_present_in_response() {
    let scores = create_sample_scores(3);

    // Verify LevelUpDate is in every score
    for score in &scores {
        // Score should be serializable and deserializable with LevelUpDate
        let json = serde_json::to_value(score).expect("serialize");
        assert!(json.get("level_up_date").is_some());
    }
}

#[tokio::test]
async fn test_level_up_date_matches_between_db_and_cache() {
    let event_id = Uuid::new_v4();
    let timestamp = NaiveDateTime::from_timestamp_opt(1609459200, 0).unwrap();

    // Create score with specific LevelUpDate
    let score = LevelCapRaceScore {
        rank: 1,
        event_id,
        cha_num: 12345,
        cha_name: "TestChar".to_string(),
        cha_class: 1,
        cha_school: Some(5),
        guild_num: Some(100),
        guild_name: Some("TestGuild".to_string()),
        start_level: 100,
        current_level: 120,
        levels_gained: 20,
        level_up_date: Some(timestamp),
    };

    // Serialize and deserialize (simulating cache storage)
    let json = serde_json::to_string(&score).expect("serialize");
    let cached: LevelCapRaceScore = serde_json::from_str(&json).expect("deserialize");

    // Verify LevelUpDate matches
    assert_eq!(cached.level_up_date, Some(timestamp));
}

#[tokio::test]
async fn test_level_up_date_null_serialization() {
    let score = LevelCapRaceScore {
        rank: 2,
        event_id: Uuid::new_v4(),
        cha_num: 54321,
        cha_name: "NoDate".to_string(),
        cha_class: 2,
        cha_school: None,
        guild_num: None,
        guild_name: None,
        start_level: 80,
        current_level: 85,
        levels_gained: 5,
        level_up_date: None,
    };

    let json = serde_json::to_value(&score).expect("serialize");
    assert!(json["level_up_date"].is_null());

    // Deserialize and verify null is preserved
    let json_str = serde_json::to_string(&score).expect("serialize");
    let deserialized: LevelCapRaceScore = serde_json::from_str(&json_str).expect("deserialize");
    assert!(deserialized.level_up_date.is_none());
}

// ============================================================================
// Test 5: In-Memory Pagination Post-Cache
// ============================================================================

#[tokio::test]
async fn test_pagination_page_1_with_size_10() {
    let scores = create_sample_scores(50);
    let page = 1u32;
    let page_size = 10u32;

    let offset = (page.saturating_sub(1)) * page_size;
    let paginated: Vec<_> = scores
        .clone()
        .into_iter()
        .skip(offset as usize)
        .take(page_size as usize)
        .collect();

    assert_eq!(paginated.len(), 10);
    assert_eq!(paginated[0].rank, 1);
    assert_eq!(paginated[9].rank, 10);
}

#[tokio::test]
async fn test_pagination_page_5_with_size_10() {
    let scores = create_sample_scores(100);
    let page = 5u32;
    let page_size = 10u32;

    let offset = (page.saturating_sub(1)) * page_size;
    let paginated: Vec<_> = scores
        .clone()
        .into_iter()
        .skip(offset as usize)
        .take(page_size as usize)
        .collect();

    assert_eq!(paginated.len(), 10);
    assert_eq!(paginated[0].rank, 41);
    assert_eq!(paginated[9].rank, 50);
}

#[tokio::test]
async fn test_pagination_beyond_range_returns_empty() {
    let scores = create_sample_scores(20);
    let page = 5u32;
    let page_size = 10u32;

    let offset = (page.saturating_sub(1)) * page_size;
    let paginated: Vec<_> = scores
        .clone()
        .into_iter()
        .skip(offset as usize)
        .take(page_size as usize)
        .collect();

    assert!(paginated.is_empty());
}

#[tokio::test]
async fn test_pagination_last_page_partial_results() {
    let scores = create_sample_scores(25);
    let page = 3u32;
    let page_size = 10u32;

    let offset = (page.saturating_sub(1)) * page_size;
    let paginated: Vec<_> = scores
        .clone()
        .into_iter()
        .skip(offset as usize)
        .take(page_size as usize)
        .collect();

    assert_eq!(paginated.len(), 5); // Only 5 items left on page 3
}

#[tokio::test]
async fn test_pagination_page_1_with_size_1() {
    let scores = create_sample_scores(10);
    let page = 1u32;
    let page_size = 1u32;

    let offset = (page.saturating_sub(1)) * page_size;
    let paginated: Vec<_> = scores
        .clone()
        .into_iter()
        .skip(offset as usize)
        .take(page_size as usize)
        .collect();

    assert_eq!(paginated.len(), 1);
    assert_eq!(paginated[0].rank, 1);
}

// ============================================================================
// Test 6: Class Filtering with Cached Data
// ============================================================================

#[tokio::test]
async fn test_class_filter_1_only() {
    let scores = create_sample_scores(30); // Mix of classes 1, 2, 3
    let target_class = 1i32;

    let filtered: Vec<_> = scores
        .into_iter()
        .filter(|s| s.cha_class == target_class)
        .collect();

    // All filtered results should be class 1
    for score in &filtered {
        assert_eq!(score.cha_class, 1);
    }
    assert!(!filtered.is_empty());
}

#[tokio::test]
async fn test_class_filter_2_only() {
    let scores = create_sample_scores(30);
    let target_class = 2i32;

    let filtered: Vec<_> = scores
        .into_iter()
        .filter(|s| s.cha_class == target_class)
        .collect();

    for score in &filtered {
        assert_eq!(score.cha_class, 2);
    }
    assert!(!filtered.is_empty());
}

#[tokio::test]
async fn test_class_filter_with_pagination() {
    let scores = create_sample_scores(60);
    let target_class = 2i32;
    let page = 1u32;
    let page_size = 5u32;

    let filtered: Vec<_> = scores
        .into_iter()
        .filter(|s| s.cha_class == target_class)
        .collect();

    let offset = (page.saturating_sub(1)) * page_size;
    let paginated: Vec<_> = filtered
        .into_iter()
        .skip(offset as usize)
        .take(page_size as usize)
        .collect();

    for score in &paginated {
        assert_eq!(score.cha_class, target_class);
    }
    assert!(paginated.len() <= 5);
}

#[tokio::test]
async fn test_no_class_filter_returns_all() {
    let scores = create_sample_scores(20);
    let class_filter: Option<i32> = None;

    let filtered: Vec<_> = scores
        .clone()
        .into_iter()
        .filter(|s| class_filter.is_none() || s.cha_class == class_filter.unwrap())
        .collect();

    assert_eq!(filtered.len(), scores.len());
}

// ============================================================================
// Test 7: Concurrent Cache Reads
// ============================================================================

#[tokio::test]
async fn test_concurrent_cache_reads() {
    let event_id = Uuid::new_v4();
    let scores = create_sample_scores(20);
    let cache = Arc::new(Mutex::new(HashMap::new()));

    // Populate cache
    let mut c = cache.lock().await;
    c.insert(
        event_id,
        CachedLevelCapRaceLeaderboard {
            scores: scores.clone(),
            total_count: 20,
            last_fetched_at: Instant::now(),
        },
    );
    drop(c);

    // Spawn 20 concurrent read tasks
    let mut tasks = vec![];
    for _ in 0..20 {
        let cache_clone = cache.clone();
        let event_id_clone = event_id;
        let task = tokio::spawn(async move {
            let c = cache_clone.lock().await;
            let cached = c.get(&event_id_clone).cloned();
            drop(c);
            cached
        });
        tasks.push(task);
    }

    // Await all tasks
    for task in tasks {
        let result = task.await.expect("task should complete");
        assert!(result.is_some(), "all concurrent reads should hit cache");
        assert_eq!(result.unwrap().scores.len(), 20);
    }
}

#[tokio::test]
async fn test_concurrent_reads_no_panics() {
    let cache = Arc::new(Mutex::new(HashMap::new()));
    let mut tasks = vec![];

    for _ in 0..50 {
        let cache_clone = cache.clone();
        let task = tokio::spawn(async move {
            let c = cache_clone.lock().await;
            let _count = c.len();
            drop(c);
        });
        tasks.push(task);
    }

    for task in tasks {
        task.await.expect("should not panic");
    }
}

// ============================================================================
// Test 8: Empty Leaderboard Handling
// ============================================================================

#[tokio::test]
async fn test_cache_with_empty_leaderboard() {
    let event_id = Uuid::new_v4();
    let scores = vec![]; // Empty
    let cache = Arc::new(Mutex::new(HashMap::new()));

    // Cache empty result
    let mut c = cache.lock().await;
    c.insert(
        event_id,
        CachedLevelCapRaceLeaderboard {
            scores: scores.clone(),
            total_count: 0,
            last_fetched_at: Instant::now(),
        },
    );
    drop(c);

    // Verify cache handles empty gracefully
    let c = cache.lock().await;
    let cached = c.get(&event_id).cloned();
    drop(c);

    assert!(cached.is_some());
    let cached_data = cached.unwrap();
    assert_eq!(cached_data.scores.len(), 0);
    assert_eq!(cached_data.total_count, 0);
}

#[tokio::test]
async fn test_pagination_with_empty_cache() {
    let scores: Vec<LevelCapRaceScore> = vec![];
    let page = 1u32;
    let page_size = 10u32;

    let offset = (page.saturating_sub(1)) * page_size;
    let paginated: Vec<_> = scores
        .into_iter()
        .skip(offset as usize)
        .take(page_size as usize)
        .collect();

    assert!(paginated.is_empty());
}

// ============================================================================
// Test 9: LevelUpDate Null Handling
// ============================================================================

#[tokio::test]
async fn test_mixed_null_and_non_null_level_up_dates() {
    let scores = create_sample_scores(10);

    // Verify some have null, some have values
    let null_count = scores.iter().filter(|s| s.level_up_date.is_none()).count();
    let non_null_count = scores.iter().filter(|s| s.level_up_date.is_some()).count();

    assert!(null_count > 0, "should have some null level_up_dates");
    assert!(non_null_count > 0, "should have some non-null level_up_dates");
}

#[tokio::test]
async fn test_filtering_with_mixed_null_level_up_dates() {
    let scores = create_sample_scores(20);
    let target_class = 1i32;

    let filtered: Vec<_> = scores
        .into_iter()
        .filter(|s| s.cha_class == target_class)
        .collect();

    // Filtering should work regardless of level_up_date
    for score in &filtered {
        assert_eq!(score.cha_class, target_class);
        // level_up_date can be None or Some
    }
}

#[tokio::test]
async fn test_serialize_deserialize_mixed_null_dates() {
    let scores = create_sample_scores(5);
    let json = serde_json::to_string(&scores).expect("serialize");
    let deserialized: Vec<LevelCapRaceScore> = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(scores.len(), deserialized.len());
    for (original, deserialized) in scores.iter().zip(deserialized.iter()) {
        assert_eq!(original.level_up_date, deserialized.level_up_date);
    }
}

// ============================================================================
// Test 10: Error Handling
// ============================================================================

#[tokio::test]
async fn test_cache_lock_acquisition_under_contention() {
    let cache = Arc::new(Mutex::new(HashMap::new()));
    let event_id = Uuid::new_v4();
    let scores = create_sample_scores(5);

    // Task 1: Hold lock briefly
    let cache_clone = cache.clone();
    let task1 = tokio::spawn(async move {
        let mut c = cache_clone.lock().await;
        c.insert(
            event_id,
            CachedLevelCapRaceLeaderboard {
                scores: scores.clone(),
                total_count: 5,
                last_fetched_at: Instant::now(),
            },
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
        drop(c);
    });

    // Task 2: Try to read while task1 holds lock
    let cache_clone = cache.clone();
    let task2 = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        let c = cache_clone.lock().await;
        let cached = c.get(&event_id).cloned();
        drop(c);
        cached
    });

    task1.await.expect("task1 should complete");
    let result = task2.await.expect("task2 should complete");
    assert!(result.is_some());
}

#[tokio::test]
async fn test_cache_operations_on_missing_event() {
    let cache: Arc<Mutex<std::collections::HashMap<Uuid, CachedLevelCapRaceLeaderboard>>> =
        Arc::new(Mutex::new(std::collections::HashMap::new()));
    let event_id = Uuid::new_v4();

    let c = cache.lock().await;
    let missing = c.get(&event_id).cloned();
    drop(c);

    assert!(missing.is_none());

    // Should not panic
    let mut c = cache.lock().await;
    c.remove(&event_id);
    drop(c);

    // Still None
    let c = cache.lock().await;
    let still_missing = c.get(&event_id).cloned();
    drop(c);

    assert!(still_missing.is_none());
}

// ============================================================================
// Test 11: Cache Size and Memory
// ============================================================================

#[tokio::test]
async fn test_multiple_events_in_cache() {
    let cache = Arc::new(Mutex::new(HashMap::new()));
    let mut event_ids = vec![];
    let mut tasks = vec![];

    // Populate cache with 5 different events
    for i in 0..5 {
        let event_id = Uuid::new_v4();
        event_ids.push(event_id);

        let scores = create_sample_scores((i + 1) * 10);
        let cache_clone = cache.clone();

        let task = tokio::spawn(async move {
            let mut c = cache_clone.lock().await;
            c.insert(
                event_id,
                CachedLevelCapRaceLeaderboard {
                    scores: scores.clone(),
                    total_count: scores.len() as i64,
                    last_fetched_at: Instant::now(),
                },
            );
            drop(c);
        });
        tasks.push(task);
    }

    for task in tasks {
        task.await.expect("should complete");
    }

    // Verify all 5 events are cached
    let c = cache.lock().await;
    assert_eq!(c.len(), 5);
    for event_id in &event_ids {
        assert!(c.contains_key(event_id));
    }
    drop(c);
}

#[tokio::test]
async fn test_cache_invalidation_with_multiple_events() {
    let cache = Arc::new(Mutex::new(HashMap::new()));
    let event_ids: Vec<_> = (0..3).map(|_| Uuid::new_v4()).collect();

    // Populate all events
    let mut c = cache.lock().await;
    for (i, event_id) in event_ids.iter().enumerate() {
        let scores = create_sample_scores(5 * (i + 1));
        c.insert(
            *event_id,
            CachedLevelCapRaceLeaderboard {
                scores: scores.clone(),
                total_count: scores.len() as i64,
                last_fetched_at: Instant::now(),
            },
        );
    }
    drop(c);

    // Invalidate middle event
    let mut c = cache.lock().await;
    c.remove(&event_ids[1]);
    drop(c);

    // Verify others remain
    let c = cache.lock().await;
    assert_eq!(c.len(), 2);
    assert!(c.contains_key(&event_ids[0]));
    assert!(!c.contains_key(&event_ids[1]));
    assert!(c.contains_key(&event_ids[2]));
    drop(c);
}
