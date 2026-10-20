# CLAUDE.md — gs-api

## Overview

Rust backend API for RAN Online GS — a private game server web platform. Handles user authentication, item shop, payments (PayPal), rankings, events, character data, analytics, and admin operations.

**Workspace crates:**

- `ran-access-point` — HTTP server (actix-web). Routes, middleware, services, background jobs. This is the binary.
- `ran-models` — Shared data models, request/response DTOs, error types. No business logic.
- `ran-db-core` — Database layer. Trait-based access to 5 MSSQL databases via Tiberius + deadpool connection pools.

**Tech stack:** Rust 2021, actix-web 4, Tiberius (MSSQL), deadpool-tiberius, tokio, serde, jsonwebtoken, actix-session (cookies), paypal-rs, aws-sdk-s3 (Cloudflare R2), lettre (SMTP), validator, tracing.

## Architecture Guidelines

### Request flow

```
HTTP → CORS/Security headers → Session or JWT middleware → Route handler → DB trait method → Stored procedure → MSSQL
```

### Two auth systems

- **Users**: Cookie-based sessions via `actix-session`. Validate with `get_user_session(session)` from `middleware/session/`.
- **Admins**: JWT Bearer tokens via `JwtMiddleware`. Admin routes live under `admin_route()`, separated from `core_route()`.

### Database layer (`ran-db-core`)

- 5 databases: `WebDB`, `GameDB`, `ShopDB`, `UserDB`, `LogDB` — each wraps a `deadpool_tiberius::Pool`.
- All DB access goes through **async traits** (e.g., `WebAuthTrait`, `WebUserTrait`, `ShopItemsTrait`).
- Trait is defined and implemented in the same file: trait definition first, then `impl Trait for XxxDB`.
- Queries use **stored procedures**: `EXEC ProcName @Param1 = @P1, @Param2 = @P2`.
- Row mapping is **manual** via `.try_get::<Type, _>("ColumnName")` or positional `.try_get(index)`.
- Use `anyhow::Context` for error propagation in DB methods. Return `Result<T>` (anyhow).

### State

- `DBState` (in `ran-access-point/src/database/`) — holds `Arc<XxxDB>` for each database + R2 client.
- `AppState` (in `ran-access-point/src/states/`) — holds in-memory caches (`Mutex<HashMap<...>>`), PayPal client.
- Both injected via `web::Data<Arc<...>>` into handlers.

### Caching

- In-memory caches in `AppState` with TTL-based invalidation.
- Profile cache in `ran-db-core/webmanager/user/cache.rs` — invalidate on login/logout via `cache::invalidate_profile(user_num)`.
- `RankingCache` uses indexed lookups and a `fetching_in_progress` flag to prevent concurrent refreshes.

## Coding Standards

### Handler pattern

```rust
#[get("/path")]
async fn handler_name(
    session: actix_session::Session,        // if user auth needed
    db: web::Data<Arc<DBState>>,            // database access
    _states: web::Data<Arc<AppState>>,      // if cache/state needed
    path: web::Path<i64>,                   // path params
    query: web::Query<SomeQuery>,           // query params
    body: web::Json<SomeRequest>,           // request body
) -> impl Responder {
    // 1. Validate session/auth
    // 2. Validate input (validator crate if DTO has #[derive(Validate)])
    // 3. Call db trait method
    // 4. Return HttpResponse::Ok().json(...) or error response
}
```

### Response format

- Success: `HttpResponse::Ok().json(data)` — shape varies per endpoint.
- Error: `HttpResponse::Status().json(json!({"status": "error", "message": "..."}))` or `json!({"message": "..."})`.
- Validation errors: `HttpResponse::BadRequest().json(flatten_validation_errors(&errors))` — returns `HashMap<field, message>`.

### DB trait pattern

```rust
#[async_trait::async_trait]
pub trait SomeTrait {
    async fn do_thing(&self, param: Type) -> Result<ReturnType>;
}

#[async_trait::async_trait]
impl SomeTrait for WebDB {
    async fn do_thing(&self, param: Type) -> Result<ReturnType> {
        let mut client = self.pool.get().await.context("Failed to get DB connection")?;
        let row = client
            .query("EXEC ProcName @P1 = @P1", &[&param])
            .await
            .context("Query execution failed")?
            .into_row()
            .await
            .context("Failed to fetch row")?;
        // manual row mapping...
    }
}
```

### Naming

- Modules/files: `snake_case`
- Types/structs: `PascalCase`
- Functions: `snake_case` — `get_*`, `create_*`, `update_*`, `delete_*`
- DB managers: `WebDB`, `GameDB`, `ShopDB`, `UserDB`, `LogDB`
- Traits: `PascalCase` + domain suffix (e.g., `WebAuthTrait`, `ShopItemsTrait`)
- Constants: `UPPER_SNAKE_CASE`

### Error handling

- DB layer: `anyhow::Result<T>` with `.context("descriptive message")`.
- Handlers: pattern match on `Result`, log errors with `tracing::error!`, return appropriate HTTP status.
- `DatabaseError` enum in `ran-models/src/errors/` for pool/query-level errors.
- Do NOT create new error types for one-off situations. Use `anyhow` or existing `DatabaseError` variants.

### Logging

- Use `tracing::error!`, `tracing::info!`, `tracing::debug!` — not `println!` or `eprintln!`.
- Structured JSON logging in release, pretty-print in debug.

## File & Folder Conventions

```
ran-access-point/src/
├── main.rs                    # Server bootstrap, middleware setup, SSL
├── routes/                    # One module per feature domain
│   ├── mod.rs                 # core_route() and admin_route() registration
│   ├── auth/mod.rs            # Auth endpoints
│   ├── user/mod.rs            # User endpoints
│   ├── admin/                 # Admin-only endpoints (sub-modules)
│   └── ...
├── middleware/session/         # Session helpers
├── services/                  # JWT, SMTP, R2, cache, SSL, background jobs
├── states/                    # AppState definition
├── database/                  # DBState initialization
└── constants/                 # Static strings, HTTP messages

ran-db-core/src/
├── lib.rs                     # DBManager, DBConfig, DatabaseTrait, pool init
├── webmanager/                # WebDB: auth, user, news, downloads, events, etc.
│   ├── mod.rs                 # WebDB struct + DatabaseTrait impl
│   ├── auth/mod.rs            # WebAuthTrait
│   ├── user/mod.rs            # WebUserTrait + cache.rs
│   └── ...
├── gamemanager/               # GameDB: characters, rankings
├── shopmanager/               # ShopDB: items, purchases
├── usermanager/               # UserDB
└── logmanager/                # LogDB

ran-models/src/
├── lib.rs                     # Re-exports all modules
├── users/                     # Auth, profile, registration DTOs
├── payments/                  # PayPal, GCash, Wise models
├── shop/                      # Shop item models
├── errors/                    # DatabaseError, validation helpers
├── events/                    # Event management models
├── admin/                     # Admin-specific DTOs
└── ...
```

### Adding a new feature

1. **Models** — Add request/response structs in `ran-models/src/{domain}/`. Derive `Serialize`, `Deserialize`, optionally `Validate`.
2. **DB trait** — Add trait + impl in `ran-db-core/src/{manager}/{feature}/mod.rs`. Use stored procedures.
3. **Route** — Add handler in `ran-access-point/src/routes/{feature}/mod.rs`. Export `pub fn configure(cfg: &mut web::ServiceConfig)`.
4. **Register** — Add `.service(web::scope("/feature").configure(feature::configure))` in `routes/mod.rs` under `core_route()` or `admin_route()`.

### Adding a new route to existing feature

1. Add handler function with `#[get|post|put|patch|delete("/path")]`.
2. Register it in the feature's `configure()` function via `cfg.service(handler_name)`.

## Common Tasks

### Database scripts

- SQL files in `db-scripts/` organized by database (`webdb/`, `gamedb/`, `shopdb/`, `userdb/`, `logdb/`).
- Sub-organized: `Tables/`, `Procedures/`, `Triggers/`, `Functions/`.

## Do's and Don'ts

### Do

- Match existing patterns exactly — study surrounding code before writing.
- Use stored procedures for all DB operations.
- Use `anyhow::Context` for meaningful error messages in DB calls.
- Use `tracing` macros for logging.
- Validate input with `validator` derive macros on request DTOs.
- Keep handler functions focused: auth check, validate, call DB, return response.
- Use `Arc<DBState>` and `Arc<AppState>` — never clone the inner data.
- Wrap sensitive data in `secrecy::SecretString` where applicable.
- Return early on auth/validation failures.

### Don't

- Don't introduce new patterns, frameworks, or abstractions without being asked.
- Don't use raw SQL inline — all queries go through stored procedures.
- Don't create new error enum variants for one-off cases; use `anyhow`.
- Don't add ORM-like abstractions or query builders — the manual `try_get` pattern is intentional.
- Don't refactor or "improve" code outside the scope of the current task.
- Don't add generic middleware, helper traits, or utility functions speculatively.
- Don't use `unwrap()` in handler or DB code — always handle errors.
- Don't use `println!` / `eprintln!` — use `tracing`.
- Don't commit `.env` or any file containing credentials.
- Don't add comments that restate what the code does. Only comment non-obvious logic.
