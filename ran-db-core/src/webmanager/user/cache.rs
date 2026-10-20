use cached::{Cached, TimedCache};
use once_cell::sync::Lazy;
use ran_models::users::ProfileData;
use std::sync::Mutex;

/// Cache for profile data, keyed by user_num
/// This reduces database load by caching frequently accessed profiles
static PROFILE_CACHE: Lazy<Mutex<TimedCache<i64, ProfileData>>> = Lazy::new(|| {
    // Cache profiles for 10 seconds, with a maximum of 1000 profiles
    Mutex::new(TimedCache::with_lifespan_and_capacity(10, 1000))
});

/// Get a profile from the cache
pub fn get_profile(user_num: i64) -> Option<ProfileData> {
    let mut cache = PROFILE_CACHE.lock().unwrap();
    cache.cache_get(&user_num).cloned()
}

/// Store a profile in the cache
pub fn store_profile(profile: ProfileData) {
    let user_num = profile.user_num;
    let mut cache = PROFILE_CACHE.lock().unwrap();
    cache.cache_set(user_num, profile);
}

/// Clear a user's profile from the cache
pub fn invalidate_profile(user_num: i64) {
    let mut cache = PROFILE_CACHE.lock().unwrap();
    cache.cache_remove(&user_num);
}

/// Get cache statistics
pub fn get_cache_stats() -> (usize, usize, f64) {
    let cache = PROFILE_CACHE.lock().unwrap();

    // The cache_hits() and cache_misses() methods return Option<u64>
    // We need to unwrap these safely and convert to usize for the return type
    let hits = match cache.cache_hits() {
        Some(h) => h as usize,
        None => 0,
    };

    let misses = match cache.cache_misses() {
        Some(m) => m as usize,
        None => 0,
    };

    // Calculate hit rate safely
    let hit_rate = if hits + misses > 0 {
        hits as f64 / (hits + misses) as f64
    } else {
        0.0
    };

    (hits, misses, hit_rate)
}
