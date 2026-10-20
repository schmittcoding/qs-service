use actix_cors::Cors;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use actix_session::{config::{CookieContentSecurity, PersistentSession}, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{self, Key, SameSite}, http, middleware::*, web, App, HttpServer, HttpResponse
};
use anyhow::{Context, Result};
use database::DBState;
use services::initialize_cache;
use states::AppState;
use std::{env, sync::Arc};
use tokio::signal;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod constants;
pub mod database;
pub mod middleware;
pub mod openapi;
pub mod routes;
pub mod services;
pub mod states;

/// Initialise the global tracing subscriber.
///
/// - **Release builds** → JSON lines (structured, machine-readable, suited for log aggregators).
/// - **Debug builds** → pretty-printed text (human-readable in the terminal).
///
/// The log filter is driven by the `RUST_LOG` environment variable via `EnvFilter`.
fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().json().flatten_event(true))
            .init();
    }

    #[cfg(debug_assertions)]
    {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().pretty())
            .init();
    }
}

fn default_headers(version: &str) -> DefaultHeaders {
    DefaultHeaders::new()
        .add(("Strict-Transport-Security", "max-age=31536000; includeSubDomains; preload"))
        .add(("X-Content-Type-Options", "nosniff"))
        .add(("X-Powered-By", "YXGames"))
        .add(("Server", "YXNetCore"))
        .add(("X-Application-Version", format!("YXNetCore : v{}", version)))
}

fn cors_config() -> Cors {
    let mut cors = Cors::default();

    #[cfg(debug_assertions)]
    {
        cors = cors
            .allowed_origin("http://ranonlinegs.local:3000")
            .allowed_origin("http://ranonlinegs.local")
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:5173")
            .allowed_origin("https://dev.ranonlinegs.com")
            .allowed_origin("https://api.sandbox.paypal.com");
    }

    #[cfg(not(debug_assertions))]
    {
        cors = cors
            .allowed_origin("https://ranonlinegs.com")
            .allowed_origin("https://api.paypal.com");
    }

    cors.allowed_origin("http://tauri.localhost")
        .allowed_origin("tauri://localhost")
        .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
        .allow_any_header()
        .allowed_headers(vec![
            http::header::ACCEPT,
            http::header::CONTENT_TYPE,
            http::header::COOKIE,
            http::header::AUTHORIZATION,
        ])
        .supports_credentials()
        .max_age(3600)
}

fn session_middleware(cookie_name: String, secret_key: Key) -> SessionMiddleware<CookieSessionStore> {
    let cookie_domain = env::var("WEB_DOMAIN").unwrap_or_else(|_| "ranonlinegs.com".to_string());
    let secure: bool = !cookie_domain.ends_with(".local");

    SessionMiddleware::builder(CookieSessionStore::default(), secret_key)
        .cookie_name(cookie_name)
        .cookie_http_only(true)
        .cookie_same_site(SameSite::Lax)
        .cookie_secure(secure)
        .cookie_content_security(CookieContentSecurity::Private)
        .cookie_domain(Some(cookie_domain))
        .session_lifecycle(PersistentSession::default().session_ttl(cookie::time::Duration::days(1)))
        .build()
}

/// Endpoint to serve the OpenAPI JSON specification
#[actix_web::get("/api-docs/openapi.json")]
async fn openapi_json() -> impl actix_web::Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string_pretty(&openapi::ApiDoc::openapi()).unwrap_or_default())
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();
    std::env::var("RUST_LOG").context("RUST_LOG must be specified")?;

    let database_state = run().await.context("[ERROR] Initializing Database")?;
    let state = match AppState::new().await {
        Ok(state) => state,
        Err(err) => {
            tracing::error!("{}", format!("[ERROR] Initializing App State : {}", err));
            std::process::exit(1);
        }
    };

    let app_state = Arc::new(state);

    let app_env = get_app_env().context("Failed to load application environment variables")?;
    let tls_config = services::ssl::load_certificates()
        .await
        .context("Failed to load TLS certificates")?;

    let state_for_actix = database_state.clone();

    initialize_cache(
        web::Data::new(app_state.clone()),
        web::Data::new(state_for_actix.clone()),
    )
    .await;

    // Start the tally scheduler background job
    services::jobs::tally_scheduler::spawn_tally_scheduler(state_for_actix.web_db.clone(), app_state.clone());

    // Start the email campaign worker
    services::jobs::campaign_worker::spawn_campaign_worker(state_for_actix.web_db.clone(), app_state.clone());

    // Start the suppression cache refresh job
    services::jobs::suppression_refresh::spawn_suppression_refresh(state_for_actix.web_db.clone(), app_state.clone());

    // DISABLED: Match readiness scheduler is obsolete.
    // Tower results are now set manually by admin via PUT /admin/events/matches/{match_id}/tower-results,
    // which transitions the match from 'Pending' → 'Waiting' for tally.
    // services::jobs::match_readiness_scheduler::spawn_match_readiness_scheduler(state_for_actix.web_db.clone());

    let server = HttpServer::new(move || {
        let mut app = App::new()
            .wrap(Compress::default())
            .wrap(default_headers(&app_env.4))
            .wrap(cors_config())
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::Data::new(state_for_actix.clone()))
            .service(openapi_json);

        let swagger = SwaggerUi::new("/swagger-ui/{_:.*}")
            .url("/api-docs/openapi.json", openapi::ApiDoc::openapi());
        app = app.service(swagger);

        app.service(web::scope("/admin").configure(routes::admin_route))
            // Cookie-based session for website users
            .service(
                web::scope("")
                    .wrap(session_middleware(app_env.1.clone(), app_env.3.clone()))
                    .configure(routes::core_route)
            )
    })
    .workers(app_env.2)
    .keep_alive(std::time::Duration::from_secs(15))
    .client_request_timeout(std::time::Duration::from_secs(10))
    // Working for HTTP
    // .bind(&format!("{}:{}", &app_env.0, 80))
    // Used in local development
    .bind(&format!("{}:{}", &app_env.0, &app_env.5))
    .context("Failed to bind HTTP server")?
    // If may cert -> 443
    // .bind_rustls(&format!("{}:{}", &app_env.0, 443), tls_config)
    // Used in local development
    .bind_rustls(&format!("{}:{}", &app_env.0, &app_env.6), tls_config)
    .context("Failed to bind HTTPS server")?
    .run();

    info!("✅ SYSCore Running at {:?}", &app_env.0);

    let graceful_shutdown = async {
        signal::ctrl_c()
            .await
            .context("Failed to install CTRL+C handler")?;
        info!("🛑 Received shutdown signal. Cleaning up...");
        Result::<()>::Ok(())
    };

    let server_handle = tokio::spawn(server);
    graceful_shutdown.await?;
    info!("🛑 Graceful shutdown initiated...");
    server_handle.await.context("Server shutdown error")??;
    info!("🛑 Cleaning up database connections...");
    info!("🛑 Server has shut down.");

    Ok(())
}

fn get_app_env() -> Result<(String, String, usize, Key, String, String, String)> {
    let cargo_version = std::env::var("APP_VERSION").context("APP_VERSION must be set")?;
    let host = std::env::var("APP_HOST").context("APP_HOST must be set")?;
    let port = std::env::var("APP_PORT").context("APP_PORT must be set")?;
    let port_ssl = std::env::var("APP_PORT_SSL").context("APP_PORT_SSL must be set")?;

    let cookie_name = std::env::var("COOKIE_NAME").context("COOKIE_NAME must be set")?;
    let max_worker_threads: usize = std::env::var("MAX_WORKER_THREADS")
        .context("MAX_WORKER_THREADS must be set")?
        .parse()
        .context("MAX_WORKER_THREADS must be a valid integer")?;
    let secret = std::env::var("SESSION_SECRET")
        .context("SESSION_SECRET must be set and be a hex-encoded string of at least 128 characters (64 bytes)")?;
    let secret_key = Key::from(secret.as_bytes());
    info!("✅ Starting SYSCore");
    info!("✅ Max Worker Threads : {}", max_worker_threads);

    Ok((
        host,
        cookie_name,
        max_worker_threads,
        secret_key,
        cargo_version,
        port,
        port_ssl,
    ))
}

async fn run() -> Result<Arc<DBState>> {
    dotenvy::dotenv().ok();

    let db_state = database::DBState::initialize()
        .await
        .context("Failed to initialize the database")?;

    tokio::spawn({
        let db_state = db_state.clone();
        async move {
            signal::ctrl_c()
                .await
                .expect("Failed to install CTRL+C handler");
            info!("🛑 Closing database connections...");
            drop(db_state);
        }
    });

    Ok(db_state)
}
