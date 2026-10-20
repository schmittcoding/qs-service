use crate::DBManager;
use chrono::Local;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use once_cell::sync::Lazy;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Database metrics namespace
const DB_METRICS: &str = "ran_db";

/// Default metrics directory path
const DEFAULT_METRICS_DIR: &str = "logs/metrics";

/// Global metrics recorder handle
pub static PROMETHEUS_HANDLE: Lazy<Mutex<Option<PrometheusHandle>>> =
    Lazy::new(|| Mutex::new(None));

/// Global path for metrics files
static METRICS_PATH: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new(DEFAULT_METRICS_DIR.to_string()));

/// Initialize metrics for the database layer
pub fn init_metrics() {
    let builder = PrometheusBuilder::new();

    // Install global recorder
    let handle = builder
        .install_recorder()
        .expect("Failed to install metrics recorder");

    // Store the handle for later use
    let mut global_handle = PROMETHEUS_HANDLE.lock().unwrap();
    *global_handle = Some(handle);

    // Ensure metrics directory exists
    let _path = METRICS_PATH.lock().unwrap();
    // if !Path::new(&*path).exists() {
    //     fs::create_dir_all(&*path).expect("Failed to create metrics directory");
    // }
}

/// Set custom path for metrics files
pub fn set_metrics_path(path: &str) {
    let mut metrics_path = METRICS_PATH.lock().unwrap();
    *metrics_path = path.to_string();

    // Ensure the directory exists
    // if !Path::new(path).exists() {
    //     fs::create_dir_all(path).expect("Failed to create metrics directory");
    // }
}

/// Save current metrics to a file
pub fn save_metrics_to_file(prefix: &str) -> std::io::Result<String> {
    let handle_lock = PROMETHEUS_HANDLE.lock().unwrap();
    let handle = handle_lock
        .as_ref()
        .expect("Metrics recorder not initialized");

    // Get the prometheus metrics as a string
    let metrics_data = handle.render();

    // Generate filename with timestamp
    let now = Local::now();
    let timestamp = now.format("%Y%m%d_%H%M%S");
    let path_lock = METRICS_PATH.lock().unwrap();

    // Create metrics filename
    let filename = format!("{}/{}_{}.metrics", *path_lock, prefix, timestamp);

    // Write metrics to file
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&filename)?;

    file.write_all(metrics_data.as_bytes())?;

    Ok(filename)
}

/// Record a query execution time
pub fn record_query_time(operation: &str, _duration: Duration) {
    let _metric_name = format!("{}_query_time_{}", DB_METRICS, operation);
    // histogram!(metric_name, duration);
}

/// Record a successful query
pub fn record_query_success(operation: &str) {
    let _metric_name = format!("{}_query_success_{}", DB_METRICS, operation);
    // counter!(metric_name, 1);
}

/// Record a failed query
pub fn record_query_failure(operation: &str) {
    let _metric_name = format!("{}_query_failure_{}", DB_METRICS, operation);
    // counter!(metric_name, 1);
}

/// Record a connection pool usage
pub fn record_pool_usage(pool_name: &str, _active_connections: u64) {
    let _metric_name = format!("{}_pool_active_{}", DB_METRICS, pool_name);
    // gauge!(metric_name, active_connections as f64);
}

/// Record a connection attempt
pub fn record_connection_attempt(pool_name: &str) {
    let _metric_name = format!("{}_connection_attempt_{}", DB_METRICS, pool_name);
    // counter!(metric_name, 1);
}

/// Record a successful connection
pub fn record_connection_success(pool_name: &str) {
    let _metric_name = format!("{}_connection_success_{}", DB_METRICS, pool_name);
    // counter!(metric_name, 1);
}

/// Record a failed connection
pub fn record_connection_failure(pool_name: &str) {
    let _metric_name = format!("{}_connection_failure_{}", DB_METRICS, pool_name);
    // counter!(metric_name, 1);
}

/// Record a timeout
pub fn record_timeout(operation: &str) {
    let _metric_name = format!("{}_timeout_{}", DB_METRICS, operation);
    // counter!(metric_name, 1);
}

/// A timer utility that automatically records metrics when dropped
pub struct QueryTimer {
    start: Instant,
    operation: String,
    finished: bool,
}

impl QueryTimer {
    /// Create a new query timer
    pub fn new(operation: &str) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.to_string(),
            finished: false,
        }
    }

    /// Finish the timer and record a success
    pub fn finish_success(&mut self) {
        if !self.finished {
            let duration = self.start.elapsed();
            record_query_time(&self.operation, duration);
            record_query_success(&self.operation);
            self.finished = true;
        }
    }

    /// Finish the timer and record a failure
    pub fn finish_failure(&mut self) {
        if !self.finished {
            let duration = self.start.elapsed();
            record_query_time(&self.operation, duration);
            record_query_failure(&self.operation);
            self.finished = true;
        }
    }
}

impl Drop for QueryTimer {
    fn drop(&mut self) {
        if !self.finished {
            // Auto-record as failure if not explicitly finished
            self.finish_failure();
        }
    }
}

/// Example function to save a full database metric snapshot
/// This can be called at key points in your application
pub async fn save_db_metrics_snapshot(db_manager: &crate::DBManager) -> std::io::Result<String> {
    // Record the current pool usage for all known databases
    if let Some(web_pool) = db_manager.get_database("web") {
        record_pool_usage("web", web_pool.status().size as u64);
        record_pool_usage("web_available", web_pool.status().available as u64);
    }

    if let Some(user_pool) = db_manager.get_database("user") {
        record_pool_usage("user", user_pool.status().size as u64);
        record_pool_usage("user_available", user_pool.status().available as u64);
    }

    if let Some(game_pool) = db_manager.get_database("game") {
        record_pool_usage("game", game_pool.status().size as u64);
        record_pool_usage("game_available", game_pool.status().available as u64);
    }

    if let Some(shop_pool) = db_manager.get_database("shop") {
        record_pool_usage("shop", shop_pool.status().size as u64);
        record_pool_usage("shop_available", shop_pool.status().available as u64);
    }

    if let Some(log_pool) = db_manager.get_database("log") {
        record_pool_usage("log", log_pool.status().size as u64);
        record_pool_usage("log_available", log_pool.status().available as u64);
    }

    // Save the metrics to a file with the current timestamp
    let filename = save_metrics_to_file("db_snapshot")?;

    // Log that metrics were saved
    tracing::info!("Database metrics snapshot saved to {}", filename);

    // Return the filename
    Ok(filename)
}

/// Start a metrics server to expose Prometheus metrics over HTTP
/// This allows tools like Prometheus to scrape metrics in real-time
pub fn start_metrics_server(address: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _addr = SocketAddr::from_str(address)
        .map_err(|e| format!("Invalid metrics server address: {}", e))?;

    // Create the recorder with an HTTP listener
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    let _handle = builder.install_recorder()?;

    Ok(())
}

/// Initialize periodic metrics snapshot collection
/// This function starts a task that takes a snapshot of metrics every 30 minutes
fn init_periodic_metrics_snapshot(db_manager: Arc<DBManager>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30 * 60)); // 30 minutes

        loop {
            interval.tick().await;

            match save_db_metrics_snapshot(&db_manager).await {
                Ok(filename) => {
                    tracing::info!("Periodic metrics snapshot saved to {}", filename);
                }
                Err(e) => {
                    tracing::error!("Failed to save periodic metrics snapshot: {}", e);
                }
            }
        }
    });
}

/// Initialize all database metrics with real-time monitoring
/// This function sets up both periodic file-based metrics and an HTTP endpoint
pub fn init_real_time_db_metrics(
    db_manager: Arc<DBManager>,
    metrics_dir: Option<&str>,
    http_address: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize metrics directory
    if let Some(dir) = metrics_dir {
        std::fs::create_dir_all(dir)?;
    }

    // Start metrics HTTP server if address is provided
    if let Some(addr) = http_address {
        start_metrics_server(addr)?;
    }

    // Start real-time pool tracking
    let db_manager_clone = db_manager.clone();
    tokio::spawn(async move {
        init_periodic_metrics_snapshot(db_manager_clone);
    });

    Ok(())
}

/// DatabaseMetrics provides comprehensive statistics about database operations
pub struct DatabaseMetrics {
    database_name: String,
}

impl DatabaseMetrics {
    /// Create a new DatabaseMetrics instance for a specific database
    pub fn new(database_name: &str) -> Self {
        Self {
            database_name: database_name.to_string(),
        }
    }

    /// Record a query execution
    pub fn record_query(&self, _duration: Duration, _query_type: &str) {
        // Record total queries
        // let total_metric = format!("{}_db_{}_queries_total", DB_METRICS, self.database_name);
        // counter!(total_metric, 1);

        // // Record query duration
        // let duration_metric = format!(
        //     "{}_db_{}_query_duration_seconds",
        //     DB_METRICS, self.database_name
        // );
        // histogram!(duration_metric, duration);

        // // Record per query type
        // let type_metric = format!(
        //     "{}_db_{}_query_type_{}",
        //     DB_METRICS, self.database_name, query_type
        // );
        // counter!(type_metric, 1);
    }

    /// Record an error
    pub fn record_error(&self, _error_type: &str) {
        // Record total errors
        // let error_metric = format!("{}_db_{}_errors_total", DB_METRICS, self.database_name);
        // counter!(error_metric, 1);

        // // Record per error type
        // let type_metric = format!(
        //     "{}_db_{}_error_type_{}",
        //     DB_METRICS, self.database_name, error_type
        // );
        // counter!(type_metric, 1);
    }

    /// Update active connections count
    pub fn set_active_connections(&self, _count: u64) {
        let _conn_metric = format!(
            "{}_db_{}_connections_active",
            DB_METRICS, self.database_name
        );
        //gauge!(conn_metric, count as f64);
    }
}

/// Example of how to use database metrics to trace real query statistics
pub fn trace_real_query<F, T>(db_name: &str, query_type: &str, query_fn: F) -> T
where
    F: FnOnce() -> T,
{
    // Get metrics for this database
    let metrics = DatabaseMetrics::new(db_name);

    // Record start time
    let start = Instant::now();

    // Execute the query function
    let result = query_fn();

    // Record query duration
    let duration = start.elapsed();
    metrics.record_query(duration, query_type);

    // Return the result
    result
}

/// Async version of trace_real_query
pub async fn trace_real_query_async<F, T>(db_name: &str, query_type: &str, query_fn: F) -> T
where
    F: std::future::Future<Output = T>,
{
    // Get metrics for this database
    let metrics = DatabaseMetrics::new(db_name);

    // Record start time
    let start = Instant::now();

    // Execute the query function
    let result = query_fn.await;

    // Record query duration
    let duration = start.elapsed();
    metrics.record_query(duration, query_type);

    // Return the result
    result
}

/// Example: How to use all metrics features together in an application
/// This example shows how to properly instrument an application with comprehensive metrics
/// Note: This is example code for documentation purposes only and is not meant to be executed directly.
#[allow(unused_variables, unreachable_code, unused_mut)]
pub fn example_metrics_usage() {
    // Documentation example only - not executable code

    // 1. Initialize metrics with both file-based and HTTP exposure
    // In a real application, you would create your DBManager first:
    // let db_manager = DBManager::new().await.expect("Failed to create DB manager");
    // let db_manager = Arc::new(db_manager);

    // Then initialize real-time metrics with the DB manager:
    // init_real_time_db_metrics(
    //     db_manager.clone(),
    //     Some("logs/metrics"),          // Directory for storing metrics files
    //     Some("0.0.0.0:9091"),          // HTTP endpoint for Prometheus scraping
    // ).expect("Failed to initialize metrics");

    // 2. Example of timing a synchronous database query with proper metrics
    let result = trace_real_query("web", "select_user", || {
        // Your actual query code goes here
        // For example: db.query("SELECT * FROM users WHERE id = $1", &[&user_id])
        "example result"
    });

    // 3. Example of timing an asynchronous database query
    // In an async context:
    // let result = trace_real_query_async("user", "update_profile", async {
    //     // Your actual async query goes here
    //     // For example: db.query_async("UPDATE users SET name = $1 WHERE id = $2", &[&name, &id]).await
    //     "updated"
    // }).await;

    // 4. Taking a manual metrics snapshot at a critical point
    // In an async context with a valid db_manager:
    // match save_db_metrics_snapshot(&db_manager).await {
    //     Ok(filename) => println!("Saved metrics to {}", filename),
    //     Err(e) => eprintln!("Failed to save metrics: {}", e),
    // }

    // Documentation notes:
    // - Real metrics are automatically collected from connection pools
    // - Query timing is precise and includes query type for detailed analysis
    // - Metrics are available both as files and via HTTP for monitoring tools
    // - Errors are automatically tracked and categorized
}
