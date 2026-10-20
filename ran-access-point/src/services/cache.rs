use tokio::sync::Mutex;
use std::{collections::HashMap, fmt, time::{Duration, Instant}};
use ran_db_core::gamemanager::{CommonRankingData, LevelRankingData};

pub struct SimpleVecCache<T> {
    pub fetching_in_progress: Mutex<bool>,
    pub data: Mutex<Option<Vec<T>>>,
    pub last_fetched_at: Mutex<Option<Instant>>,
    pub refresh_interval: Duration,
}

impl<T> fmt::Debug for SimpleVecCache<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SimpleVecCache")
            .field("data", &self.data)
            .field("last_fetched_at", &self.last_fetched_at) 
            .field("refresh_interval", &self.refresh_interval) 
            .field("fetching_in_progress", &self.fetching_in_progress)
            .finish()
    }
}

impl<T> SimpleVecCache<T> {
    pub fn new(refresh_interval: Duration) -> Self {
        SimpleVecCache {
            fetching_in_progress: Mutex::new(false),
            data: Mutex::new(None),
            last_fetched_at: Mutex::new(None),
            refresh_interval,
        }
    }

    pub async fn is_stale(&self) -> bool {
        let last = self.last_fetched_at.lock().await;
        match *last {
            Some(instant) => instant.elapsed() >= self.refresh_interval,
            None => true,
        }
    }

    pub async fn update(&self, new_data: Vec<T>) {
        *self.data.lock().await = Some(new_data);
        *self.last_fetched_at.lock().await = Some(Instant::now());
    }
}

// --- Indexed Ranking Cache ---

/// Pre-indexed ranking data. Built once on cache refresh, O(1) bucket lookups on read.
pub struct IndexedRankingData {
    /// All records sorted by cha_money descending (with rank assigned)
    pub all: Vec<CommonRankingData>,
    /// Indices into `all`, filtered by cha_class only
    pub by_class: HashMap<i32, Vec<usize>>,
    /// Indices into `all`, filtered by cha_school only
    pub by_school: HashMap<i32, Vec<usize>>,
    /// Indices into `all`, filtered by (cha_class, cha_school)
    pub by_class_and_school: HashMap<(i32, i32), Vec<usize>>,
}

impl IndexedRankingData {
    pub fn build(data: Vec<CommonRankingData>) -> Self {
        let mut by_class: HashMap<i32, Vec<usize>> = HashMap::new();
        let mut by_school: HashMap<i32, Vec<usize>> = HashMap::new();
        let mut by_class_and_school: HashMap<(i32, i32), Vec<usize>> = HashMap::new();

        for (i, record) in data.iter().enumerate() {
            by_class.entry(record.cha_class).or_default().push(i);
            by_school.entry(record.cha_school).or_default().push(i);
            by_class_and_school
                .entry((record.cha_class, record.cha_school))
                .or_default()
                .push(i);
        }

        IndexedRankingData {
            all: data,
            by_class,
            by_school,
            by_class_and_school,
        }
    }

    /// Returns the indices to iterate based on the filter combination.
    /// `None` means iterate all records (no filter applied).
    pub fn get_bucket(&self, cha_class: Option<i32>, cha_school: Option<i32>) -> Option<&[usize]> {
        match (cha_class, cha_school) {
            (Some(c), Some(s)) => self.by_class_and_school.get(&(c, s)).map(|v| v.as_slice()),
            (Some(c), None) => self.by_class.get(&c).map(|v| v.as_slice()),
            (None, Some(s)) => self.by_school.get(&s).map(|v| v.as_slice()),
            (None, None) => None, // caller should use `all` directly
        }
    }
}

pub struct RankingCache {
    pub fetching_in_progress: Mutex<bool>,
    pub data: Mutex<Option<IndexedRankingData>>,
    pub last_fetched_at: Mutex<Option<Instant>>,
    pub refresh_interval: Duration,
}

impl fmt::Debug for RankingCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RankingCache")
            .field("last_fetched_at", &self.last_fetched_at)
            .field("refresh_interval", &self.refresh_interval)
            .field("fetching_in_progress", &self.fetching_in_progress)
            .finish()
    }
}

impl RankingCache {
    pub fn new(refresh_interval: Duration) -> Self {
        RankingCache {
            fetching_in_progress: Mutex::new(false),
            data: Mutex::new(None),
            last_fetched_at: Mutex::new(None),
            refresh_interval,
        }
    }

    pub async fn is_stale(&self) -> bool {
        let last = self.last_fetched_at.lock().await;
        match *last {
            Some(instant) => instant.elapsed() >= self.refresh_interval,
            None => true,
        }
    }

    pub async fn update(&self, raw_data: Vec<CommonRankingData>) {
        let indexed = IndexedRankingData::build(raw_data);
        *self.data.lock().await = Some(indexed);
        *self.last_fetched_at.lock().await = Some(Instant::now());
    }
}


// --- Level Ranking Cache ---

pub struct LevelIndexedRankingData {
    pub all: Vec<LevelRankingData>,
    pub by_class: HashMap<i32, Vec<usize>>,
    pub by_school: HashMap<i32, Vec<usize>>,
    pub by_class_and_school: HashMap<(i32, i32), Vec<usize>>,
}

impl LevelIndexedRankingData {
    pub fn build(data: Vec<LevelRankingData>) -> Self {
        let mut by_class: HashMap<i32, Vec<usize>> = HashMap::new();
        let mut by_school: HashMap<i32, Vec<usize>> = HashMap::new();
        let mut by_class_and_school: HashMap<(i32, i32), Vec<usize>> = HashMap::new();

        for (i, record) in data.iter().enumerate() {
            by_class.entry(record.cha_class).or_default().push(i);
            by_school.entry(record.cha_school).or_default().push(i);
            by_class_and_school
                .entry((record.cha_class, record.cha_school))
                .or_default()
                .push(i);
        }

        LevelIndexedRankingData {
            all: data,
            by_class,
            by_school,
            by_class_and_school,
        }
    }

    pub fn get_bucket(&self, cha_class: Option<i32>, cha_school: Option<i32>) -> Option<&[usize]> {
        match (cha_class, cha_school) {
            (Some(c), Some(s)) => self.by_class_and_school.get(&(c, s)).map(|v| v.as_slice()),
            (Some(c), None) => self.by_class.get(&c).map(|v| v.as_slice()),
            (None, Some(s)) => self.by_school.get(&s).map(|v| v.as_slice()),
            (None, None) => None,
        }
    }
}

pub struct LevelRankingCache {
    pub fetching_in_progress: Mutex<bool>,
    pub data: Mutex<Option<LevelIndexedRankingData>>,
    pub last_fetched_at: Mutex<Option<Instant>>,
    pub refresh_interval: Duration,
}

impl fmt::Debug for LevelRankingCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LevelRankingCache")
            .field("last_fetched_at", &self.last_fetched_at)
            .field("refresh_interval", &self.refresh_interval)
            .field("fetching_in_progress", &self.fetching_in_progress)
            .finish()
    }
}

impl LevelRankingCache {
    pub fn new(refresh_interval: Duration) -> Self {
        LevelRankingCache {
            fetching_in_progress: Mutex::new(false),
            data: Mutex::new(None),
            last_fetched_at: Mutex::new(None),
            refresh_interval,
        }
    }

    pub async fn is_stale(&self) -> bool {
        let last = self.last_fetched_at.lock().await;
        match *last {
            Some(instant) => instant.elapsed() >= self.refresh_interval,
            None => true,
        }
    }

    pub async fn update(&self, raw_data: Vec<LevelRankingData>) {
        let indexed = LevelIndexedRankingData::build(raw_data);
        *self.data.lock().await = Some(indexed);
        *self.last_fetched_at.lock().await = Some(Instant::now());
    }
}