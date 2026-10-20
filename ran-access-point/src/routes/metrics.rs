use actix_web::{get, HttpResponse, Responder};
use anyhow::Result;
use ran_db_core::metrics;
use tracing::error;
/// Initialize metrics functionality
/// This should be called during application startup
pub async fn init_metrics() -> Result<(), anyhow::Error> {
    // Initialize metrics recorder
    metrics::init_metrics();

    // Set path for metrics files
    metrics::set_metrics_path("logs/metrics");

    // Start an HTTP server for Prometheus metrics
    // This uses a different port than the main application
    tokio::spawn(async {
        if let Err(e) = metrics::start_metrics_server("127.0.0.1:9091") {
            error!("Failed to start metrics server: {:?}", e);
        }
    });

    Ok(())
}

/// Get metrics prometheus format
/// This endpoint exposes metrics in Prometheus format
#[get("/metrics")]
pub async fn get_metrics() -> impl Responder {
    let handle_lock = metrics::PROMETHEUS_HANDLE.lock().unwrap();
    if let Some(handle) = handle_lock.as_ref() {
        let metrics_text = handle.render();
        HttpResponse::Ok()
            .content_type("text/plain")
            .body(metrics_text)
    } else {
        HttpResponse::InternalServerError().body("Metrics recorder not initialized")
    }
}
