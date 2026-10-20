# Workflow: OpenAPI Integration — Phase 2 (ToSchema) & Phase 3A–3D (Route Annotations)

## Feature Summary

Phase 1 of OpenAPI integration is complete: `utoipa` v4 and `utoipa-swagger-ui` v7 are wired into the workspace, `ran-access-point/src/openapi.rs` exposes an empty `ApiDoc`, and `main.rs` mounts Swagger UI at `/docs` (debug-only) and the JSON spec at `/api-docs/openapi.json`. This workflow drives Phase 2 — adding `#[derive(utoipa::ToSchema)]` to all public request/response models in `ran-models` — and Phase 3A–3D — annotating all 127 route handlers across 47 route files with `#[utoipa::path(...)]` macros and registering them in the central `ApiDoc`. The result is a fully populated OpenAPI document covering the public (cookie-session) and admin (JWT) HTTP surface.

## Agents Involved

1. **Architect** — Inventories every public DTO/model in `ran-models` and every handler across `ran-access-point/src/routes/`, classifies them into the four route groups (A–D), and produces a per-file checklist with type names, handler names, HTTP methods/paths, tags, and security requirements.
2. **CodeGen — Phase 2A (Priority Models)** — Adds `#[derive(utoipa::ToSchema)]` to the priority model files (users, admin/auth, news, payments, events).
3. **CodeGen — Phase 2B (Remaining Models)** — Adds `ToSchema` to all remaining model files (admin sub-modules, shop, marketplace, download, reports, item_shop, characters, etc.).
4. **CodeGen — Phase 3A (Group A Routes)** — Annotates `checks.rs`, `auth/mod.rs`, `admin/auth/mod.rs`, `user/mod.rs`, `news/mod.rs`. Updates `openapi.rs` `ApiDoc` after.
5. **CodeGen — Phase 3B (Group B Routes)** — Annotates `payments/mod.rs`, `events/*` (all 6 files), `rankings/mod.rs`, `transactions/mod.rs`, `characters.rs`. Updates `ApiDoc`.
6. **CodeGen — Phase 3C (Group C Admin Routes)** — Annotates all admin sub-routes: `admin/events/*`, `admin/players/*`, `admin/characters/*`, `admin/items/*`, `admin/recharge/*`, `admin/shop/*`, `admin/news/*`, `admin/purchases.rs`, `admin/item_shop.rs`, `admin/download/*`. Updates `ApiDoc`.
7. **CodeGen — Phase 3D (Group D Stragglers)** — Annotates `analytics/*`, `metrics.rs`, `item_shop.rs`, `download/*`, and any remaining handlers not covered by 3A–3C. Updates `ApiDoc`.
8. **Reviewer** — Audits all model and route changes for correctness, security tagging, schema completeness, and that `cargo check`, `cargo clippy`, and the Swagger UI render cleanly.

## Implementation Workflow

### Phase 1 — Architect

**Prompt:**

You are the Architect for the gs-api project (Rust workspace with three crates: `ran-access-point` [actix-web 4 binary], `ran-models` [DTOs], `ran-db-core` [Tiberius MSSQL trait layer over five databases: WebDB, GameDB, ShopDB, UserDB, LogDB]). Phase 1 of OpenAPI integration is complete — `utoipa` v4 and `utoipa-swagger-ui` v7 are workspace dependencies, `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/openapi.rs` defines an empty `ApiDoc` with `cookie_session` and `bearer_jwt` security schemes, and `main.rs` mounts Swagger UI at `/docs` (debug-only) and JSON at `/api-docs/openapi.json`.

Your job is to produce a complete inventory and plan, NOT to write code. Output a single markdown plan in your final assistant message (do not write a file).

**Tasks:**

1. **Inventory all public request/response types in `ran-models`.** Walk every file under `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-models/src/` (sub-dirs include `users/`, `admin/`, `events/`, `news/`, `payments/`, `shop/`, `marketplace/`, `download/`, `reports/`, `item_shop.rs`, `errors/`). For each file list every public struct or enum that is used as an HTTP request body, query, path param, or response. Note any `#[serde(rename = "...")]`, `#[serde(rename_all = "...")]`, or flatten attributes — these may need `#[schema(rename = ...)]` mirrors. Note types that already derive `Serialize/Deserialize/Validate`.

2. **Inventory all 127 route handlers across `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/`.** For each handler record: file path, handler `async fn` name, HTTP method (`#[get|post|put|patch|delete("...")]`), path string, request/response types referenced, whether it uses `actix_session::Session` (cookie auth) or sits behind `JwtMiddleware` (admin), and a proposed `tag` and `operation_id`.

3. **Classify routes into groups (A–D):**
   - **Group A (3A):** `checks.rs`, `auth/mod.rs`, `admin/auth/mod.rs`, `user/mod.rs`, `news/mod.rs`
   - **Group B (3B):** `payments/mod.rs`, all 6 files in `events/` (`mod.rs`, `leaderboard.rs`, `match_status.rs`, `participants.rs`, `registration.rs`, `snapshots.rs`), `rankings/mod.rs`, `transactions/mod.rs`, `characters.rs`
   - **Group C (3C):** all admin sub-routes — `admin/events/*` (`crud.rs`, `deductions.rs`, `export.rs`, `gimmicks.rs`, `match_management.rs`, `participants.rs`, `registration.rs`, `tally.rs`, `mod.rs`), `admin/players/*`, `admin/characters/*`, `admin/items/*`, `admin/recharge/*`, `admin/shop/*`, `admin/news/*`, `admin/purchases.rs`, `admin/item_shop.rs`, `admin/download/*`
   - **Group D (3D):** `analytics/*`, `metrics.rs`, `item_shop.rs`, `download/*`, plus anything not matched above. Confirm no handlers fall through.

4. **Tag convention.** Public/core routes get short tags (`"Auth"`, `"User"`, `"News"`, `"Payments"`, `"Events"`, `"Rankings"`, `"Transactions"`, `"Characters"`, `"Health"`, `"Analytics"`, `"Downloads"`). Admin routes get `"Admin - Auth"`, `"Admin - Events"`, `"Admin - Players"`, `"Admin - Characters"`, `"Admin - Items"`, `"Admin - Recharge"`, `"Admin - Shop"`, `"Admin - News"`, `"Admin - Purchases"`, `"Admin - ItemShop"`, `"Admin - Downloads"`.

5. **Operation_id collisions.** Specifically flag any handler name that exists in both core and admin scopes (e.g., `authenticate` exists in `auth/mod.rs` and `admin/auth/mod.rs`). For all such pairs require explicit `operation_id = "core_<name>"` and `operation_id = "admin_<name>"`.

6. **Path strings.** Document with hardcoded `/api/v1/...` for core routes and `/admin/v1/...` for admin routes. Path versions are runtime-mounted but the docs must be fixed.

7. **Visibility change.** Note in the plan that handler `async fn` declarations must be made `pub async fn`. This is safe because actix-web registers them via the route-attribute macros, not by direct call. List how many handlers will need this change.

8. **ToSchema priority order for Phase 2A:** `users/mod.rs`, `admin/auth/mod.rs`, `news/mod.rs`, `payments/mod.rs`, all `events/*`. Phase 2B covers everything else.

9. **Edge cases to flag:**
   - Generic wrapper types (e.g., `Vec<T>`, `HashMap`) — `ToSchema` must be on the inner `T`.
   - Types from external crates (`chrono::DateTime<Utc>`, `uuid::Uuid`, `rust_decimal::Decimal`) need `#[schema(value_type = ...)]` overrides.
   - Newtype wrappers around `String` or numeric types may need `value_type`.
   - Models with `#[serde(flatten)]` need `#[schema(inline)]` or careful handling.
   - `secrecy::SecretString` — must be marked `#[schema(value_type = String, format = "password")]` and only on request types, never response types.

**Deliverables in your final message (no files written):**

- Group A/B/C/D handler counts and file lists.
- For each route group: a table of `(file, handler, method, path, tag, operation_id, request_type, response_type, security)`.
- For Phase 2A and 2B: ordered list of model files and the public types in each.
- Explicit list of operation_id collisions and the disambiguated names.
- Edge-case list with proposed `#[schema(...)]` annotations.

### Phase 2A — CodeGen (Priority Models)

**Depends on:** Architect output (model inventory, edge-case list).

**Prompt:**

You are CodeGen for the gs-api Rust workspace. Phase 1 OpenAPI infrastructure is in place. Read the Architect's plan first; it lists every public DTO/model and any required `#[schema(...)]` overrides.

**Scope (priority models only):**

Add `#[derive(utoipa::ToSchema)]` to every public request/response struct or enum in these files:

- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-models/src/users/mod.rs` (and any sub-files under `ran-models/src/users/`)
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-models/src/admin/auth/` (all files)
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-models/src/news/` (all files)
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-models/src/payments/` (all files)
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-models/src/events/` (all files)

**Rules:**

1. Append `utoipa::ToSchema` to the existing `#[derive(...)]` line. Do not split derives across two attributes unless that is the existing style.
2. Preserve all existing derives (`Serialize`, `Deserialize`, `Validate`, `Debug`, `Clone`, etc.) and field-level attributes.
3. For fields whose types do not implement `ToSchema` natively (`chrono::DateTime<Utc>`, `chrono::NaiveDateTime`, `uuid::Uuid`, `rust_decimal::Decimal`, `secrecy::SecretString`), add `#[schema(value_type = ...)]` per the Architect's edge-case list. Examples:
   - `DateTime<Utc>` -> `#[schema(value_type = String, format = "date-time")]`
   - `Decimal` -> `#[schema(value_type = String)]`
   - `SecretString` -> `#[schema(value_type = String, format = "password")]`
   - `Uuid` -> `#[schema(value_type = String, format = "uuid")]`
4. Do NOT change struct visibility, field names, serde attributes, or business logic.
5. If a type uses `#[serde(rename_all = "camelCase")]`, mirror with `#[schema(rename_all = "camelCase")]` so the generated schema matches the wire format.
6. Update `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/openapi.rs`: add every newly-decorated type to the `components(schemas(...))` list (use fully-qualified path `ran_models::users::SomeType` or whatever matches the `pub use` re-exports in `ran-models/src/lib.rs`).
7. After editing, run `cargo check -p ran-models` then `cargo check -p ran-access-point` from the workspace root `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/`. Both must compile clean. Report any unresolved errors back rather than guessing.

**Deliverable:** All Phase 2A model files compile cleanly with `ToSchema` derives, and `openapi.rs` lists every new schema. Do NOT touch route handler files in this phase.

### Phase 2B — CodeGen (Remaining Models)

**Depends on:** Phase 2A complete and compiling.

**Prompt:**

You are CodeGen for the gs-api Rust workspace. Phase 2A added `ToSchema` to priority models (users, admin/auth, news, payments, events). Now finish the rest.

**Scope (remaining model files under `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-models/src/`):**

- `admin/` (all sub-modules except `auth/` which is done) — `characters/`, `players/`, `items/`, `game.rs`, `purchases.rs`, `users.rs`, plus `admin/mod.rs`
- `shop/`
- `marketplace/`
- `download/`
- `reports/`
- `item_shop.rs`
- `errors/` (only types that are returned over HTTP — skip internal `DatabaseError` if it never serializes)
- `util.rs` (only if it defines public DTOs)

**Rules:** Identical to Phase 2A. Add `#[derive(utoipa::ToSchema)]`, add `#[schema(value_type = ...)]` overrides where required, mirror `serde` rename attributes, register every new schema in `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/openapi.rs` under `components(schemas(...))`. Do not change semantics. Run `cargo check -p ran-models` and `cargo check -p ran-access-point` from `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/` and report any failures.

**Deliverable:** Every public request/response model in `ran-models` derives `ToSchema` and is registered in `ApiDoc`. Workspace compiles.

### Phase 3A — CodeGen (Group A Route Annotations)

**Depends on:** Phases 2A + 2B complete; `ApiDoc` already lists all schemas.

**Prompt:**

You are CodeGen for the gs-api Rust workspace. All `ran-models` types now derive `ToSchema`. You will annotate Group A route handlers and register them in `ApiDoc`. Read the Architect's per-handler table first.

**Scope (Group A route files):**

- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/checks.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/auth/mod.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/auth/mod.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/user/mod.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/news/mod.rs`

**Per-handler edits:**

1. Change the handler signature from `async fn handler_name(...)` to `pub async fn handler_name(...)`. This is safe — actix-web registers handlers via route-attribute macros, never by direct call. (If a handler is already `pub`, leave it.)

2. Add `#[utoipa::path(...)]` immediately above the existing `#[get|post|put|patch|delete("...")]` macro. Example shape for a public route:

   ```rust
   #[utoipa::path(
       get,
       path = "/api/v1/news",
       tag = "News",
       operation_id = "list_news",
       responses(
           (status = 200, description = "News list", body = Vec<NewsItem>),
           (status = 500, description = "Server error"),
       ),
       security(("cookie_session" = []))
   )]
   #[get("/news")]
   pub async fn list_news(...) -> impl Responder { ... }
   ```

   For admin routes use `path = "/admin/v1/..."`, `tag = "Admin - Auth"` (etc), and `security(("bearer_jwt" = []))`.

3. **Tag convention:**
   - `checks.rs` -> `"Health"`
   - `auth/mod.rs` -> `"Auth"` with `security(("cookie_session" = []))` (or no security on `login`/`register`)
   - `admin/auth/mod.rs` -> `"Admin - Auth"` with `security(("bearer_jwt" = []))` on protected handlers
   - `user/mod.rs` -> `"User"` with `security(("cookie_session" = []))`
   - `news/mod.rs` -> `"News"`

4. **Operation_id disambiguation:** `auth/mod.rs::authenticate` -> `operation_id = "core_authenticate"`; `admin/auth/mod.rs::authenticate` -> `operation_id = "admin_authenticate"`. Apply the same pattern to any other name collisions the Architect flagged.

5. **Path strings** are hardcoded as `/api/v1/<rest>` for core or `/admin/v1/<rest>` for admin. Pull the `<rest>` from the existing actix `#[get("...")]` plus the parent `web::scope("...")` chain in `routes/mod.rs`.

6. **Request/response bodies:** put request DTO type in `request_body = TypeName` and the success body in `responses(... body = TypeName ...)`. For empty success, use `responses((status = 204, description = "..."))`.

7. **Path & query params:** add `params(...)` entries for `web::Path<...>` and `web::Query<...>` extractors.

8. **Update `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/openapi.rs`** — append every Group A handler to the `paths(...)` list using its fully-qualified module path (e.g., `crate::routes::auth::login`).

9. After all edits, run `cargo check -p ran-access-point` from `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/`. The build must pass. Then start the server in debug and confirm `/docs` renders Group A endpoints — if you cannot run the server, at minimum confirm `cargo check` passes and report.

**Do NOT** modify routes in Groups B/C/D in this phase. Do NOT modify model files. Do NOT change handler bodies, route paths, or middleware.

### Phase 3B — CodeGen (Group B Route Annotations)

**Depends on:** Phase 3A complete and compiling.

**Prompt:**

You are CodeGen. Repeat the Phase 3A pattern for Group B routes. Read the Architect's table for tag, operation_id, request/response types, and security per handler.

**Scope (Group B):**

- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/payments/mod.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/events/mod.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/events/leaderboard.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/events/match_status.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/events/participants.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/events/registration.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/events/snapshots.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/rankings/mod.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/transactions/mod.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/characters.rs`

**Tags:** `"Payments"`, `"Events"`, `"Rankings"`, `"Transactions"`, `"Characters"`. All routes here are core/public — use `security(("cookie_session" = []))` unless the Architect's plan indicates otherwise. Path prefix `/api/v1/...`.

**Rules:** Same as 3A. Make handlers `pub`. Add `#[utoipa::path(...)]` above the existing actix attribute. Register every new handler in `paths(...)` in `openapi.rs`. Build with `cargo check -p ran-access-point`. Do not touch Groups A/C/D, models, or middleware. Watch for `events/registration.rs` having both core registration handlers — keep operation_ids unique (e.g., `events_register`, `events_unregister`).

### Phase 3C — CodeGen (Group C Admin Route Annotations)

**Depends on:** Phase 3B complete and compiling.

**Prompt:**

You are CodeGen. Repeat the pattern for Group C — these are all admin routes behind `JwtMiddleware`.

**Scope (Group C):**

- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/events/mod.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/events/crud.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/events/deductions.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/events/export.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/events/gimmicks.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/events/match_management.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/events/participants.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/events/registration.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/events/tally.rs`
- All files under `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/players/`
- All files under `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/characters/`
- All files under `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/items/`
- All files under `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/recharge/`
- All files under `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/shop/`
- All files under `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/news/`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/purchases.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/item_shop.rs`
- All files under `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/admin/download/`

**Tags:** `"Admin - Events"`, `"Admin - Players"`, `"Admin - Characters"`, `"Admin - Items"`, `"Admin - Recharge"`, `"Admin - Shop"`, `"Admin - News"`, `"Admin - Purchases"`, `"Admin - ItemShop"`, `"Admin - Downloads"`. Path prefix `/admin/v1/...`. Security `security(("bearer_jwt" = []))` on every handler.

**Rules:** Same as 3A/3B. Make handlers `pub`. Add `#[utoipa::path(...)]`. Register each handler in `openapi.rs` `paths(...)`. Run `cargo check -p ran-access-point`. Do not modify models or other groups.

**Special care:**
- `admin/events/registration.rs` has admin counterparts to core event registration — operation_ids must be `admin_events_register`, `admin_events_unregister`, etc.
- `admin/events/crud.rs` likely contains create/update/delete for events — each needs distinct operation_id (`admin_events_create`, `admin_events_update`, `admin_events_delete`).
- `admin/events/tally.rs` may expose force-tally endpoints; mark them clearly in description because they are destructive.

### Phase 3D — CodeGen (Group D Stragglers)

**Depends on:** Phase 3C complete and compiling.

**Prompt:**

You are CodeGen. Final batch — annotate any remaining handlers and confirm coverage.

**Scope (Group D):**

- All files under `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/analytics/`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/metrics.rs`
- `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/item_shop.rs`
- All files under `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/download/`
- Any handler in any other route file not yet covered (cross-reference Architect's full handler list).

**Tags:** `"Analytics"`, `"Metrics"`, `"ItemShop"`, `"Downloads"`. Most are core (`/api/v1/...`, `cookie_session`); `metrics.rs` may be unauthenticated or admin — follow the Architect's classification.

**Rules:** Same as prior phases. Make handlers `pub`. Add `#[utoipa::path(...)]`. Register in `openapi.rs`. Build clean.

**Final check:** Grep the workspace for any `#[get|post|put|patch|delete(...)]` attribute that lacks an immediately-preceding `#[utoipa::path(...)]`. The grep should return zero results across `ran-access-point/src/routes/`. Report the grep command used and its output.

### Phase 4 — Reviewer

**Depends on:** Phases 2A, 2B, 3A, 3B, 3C, 3D all complete.

**Prompt:**

You are the Reviewer for the gs-api project. All `ran-models` public DTOs now derive `utoipa::ToSchema`, and all 127 route handlers across the 47 files under `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/routes/` have `#[utoipa::path(...)]` annotations registered in `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/ran-access-point/src/openapi.rs`.

**Audit checklist:**

1. **Compile / lint.** From `/Users/cedricestrada/Desktop/development/workspace-schmitt/gs-main/gs-api/` run `cargo check --workspace` and `cargo clippy --workspace -- -D warnings`. Both must pass.

2. **Schema completeness.** Confirm every public DTO referenced by a `#[utoipa::path(...)]` `request_body` or `responses(... body = ...)` is registered in `openapi.rs` `components(schemas(...))`. Missing entries cause runtime spec errors but compile fine — grep for body types in route files and cross-reference.

3. **Path coverage.** Confirm every annotated handler is registered in `paths(...)` in `openapi.rs`. Run `rg "#\\[utoipa::path" ran-access-point/src/routes/ -c` and reconcile against the `paths(...)` list count.

4. **Security tagging.**
   - Every core/public handler that calls `get_user_session(session)` or otherwise needs auth must have `security(("cookie_session" = []))`.
   - Every admin handler under `admin_route()` must have `security(("bearer_jwt" = []))`.
   - Public unauthenticated handlers (login, register, public news read, health checks) must have NO `security` clause or an empty one.

5. **Tag consistency.** Public tags are short (`"Auth"`, `"User"`, etc.); admin tags use `"Admin - <Domain>"`. No mixed casing or stray spaces.

6. **Path strings.** Core handlers use `/api/v1/...`, admin handlers use `/admin/v1/...`. The path string in `#[utoipa::path]` matches the actix-web route plus its parent `web::scope` chain.

7. **operation_id collisions.** Search for duplicate `operation_id = "..."` strings across all annotated routes. Especially confirm `core_authenticate` vs `admin_authenticate` and any other dual-scope handler.

8. **Visibility.** Spot-check that handler `async fn` declarations were changed to `pub async fn`. Confirm `cargo check` would have caught any private-handler issue from the macro expansion.

9. **Sensitive fields.** Confirm any model field of type `secrecy::SecretString` (or otherwise sensitive) appears only in request bodies, not response bodies, and carries `#[schema(value_type = String, format = "password")]`.

10. **Generated spec sanity.** If feasible, build in debug, run the server, fetch `http://localhost:<port>/api-docs/openapi.json`, and confirm the JSON parses, lists ~127 paths, and has a `components.schemas` with ~45+ entries. If running locally is not possible, at minimum confirm `cargo check --workspace` passes.

11. **Pattern adherence.** Confirm no business logic, route paths, middleware, DB calls, or model field semantics were changed. Phase 2/3 should be additive only.

**Deliverable:** A short pass/fail summary in your final assistant message listing any issues found by file and line. If there are blockers, identify which CodeGen phase needs follow-up. If clean, sign off and note the final route + schema counts.

## Files Expected to Change

### Phase 2A (Priority Models)
- `ran-models/src/users/` (all files)
- `ran-models/src/admin/auth/` (all files)
- `ran-models/src/news/` (all files)
- `ran-models/src/payments/` (all files)
- `ran-models/src/events/` (all files)
- `ran-access-point/src/openapi.rs` (schemas list)

### Phase 2B (Remaining Models)
- `ran-models/src/admin/characters/`, `admin/players/`, `admin/items/`, `admin/game.rs`, `admin/purchases.rs`, `admin/users.rs`, `admin/mod.rs`
- `ran-models/src/shop/`
- `ran-models/src/marketplace/`
- `ran-models/src/download/`
- `ran-models/src/reports/`
- `ran-models/src/item_shop.rs`
- `ran-models/src/errors/` (only HTTP-serialized variants)
- `ran-access-point/src/openapi.rs` (schemas list)

### Phase 3A (Group A Routes)
- `ran-access-point/src/routes/checks.rs`
- `ran-access-point/src/routes/auth/mod.rs`
- `ran-access-point/src/routes/admin/auth/mod.rs`
- `ran-access-point/src/routes/user/mod.rs`
- `ran-access-point/src/routes/news/mod.rs`
- `ran-access-point/src/openapi.rs` (paths list)

### Phase 3B (Group B Routes)
- `ran-access-point/src/routes/payments/mod.rs`
- `ran-access-point/src/routes/events/mod.rs`, `leaderboard.rs`, `match_status.rs`, `participants.rs`, `registration.rs`, `snapshots.rs`
- `ran-access-point/src/routes/rankings/mod.rs`
- `ran-access-point/src/routes/transactions/mod.rs`
- `ran-access-point/src/routes/characters.rs`
- `ran-access-point/src/openapi.rs` (paths list)

### Phase 3C (Group C Admin Routes)
- `ran-access-point/src/routes/admin/events/` (all 9 files)
- `ran-access-point/src/routes/admin/players/` (all files)
- `ran-access-point/src/routes/admin/characters/` (all files)
- `ran-access-point/src/routes/admin/items/` (all files)
- `ran-access-point/src/routes/admin/recharge/` (all files)
- `ran-access-point/src/routes/admin/shop/` (all files)
- `ran-access-point/src/routes/admin/news/` (all files)
- `ran-access-point/src/routes/admin/purchases.rs`
- `ran-access-point/src/routes/admin/item_shop.rs`
- `ran-access-point/src/routes/admin/download/` (all files)
- `ran-access-point/src/openapi.rs` (paths list)

### Phase 3D (Group D Stragglers)
- `ran-access-point/src/routes/analytics/` (all files)
- `ran-access-point/src/routes/metrics.rs`
- `ran-access-point/src/routes/item_shop.rs`
- `ran-access-point/src/routes/download/` (all files)
- `ran-access-point/src/openapi.rs` (paths list)

## Risk Areas

- **Visibility flip on 127 handlers.** Every handler `async fn` must become `pub async fn`. Doing this in bulk with `sed` is risky if a handler signature spans multiple lines — CodeGen must edit per file, not workspace-wide. A miss does not break compile (actix macros call them through generated wrappers), but it does break direct `crate::routes::...` references in `openapi.rs` `paths(...)`.

- **operation_id collisions.** `auth/mod.rs::authenticate` and `admin/auth/mod.rs::authenticate` both exist; there are likely more (e.g., `events/registration.rs::register` vs `admin/events/registration.rs::register`). Without explicit `operation_id`, the generated client SDKs will collide silently or `utoipa` will refuse to build.

- **External-type ToSchema.** `chrono`, `uuid`, `rust_decimal`, and `secrecy` types do NOT implement `ToSchema`. Every field of these types needs `#[schema(value_type = ...)]` or compile fails with cryptic trait-bound errors deep inside the derive macro.

- **`#[serde(flatten)]` and untagged enums.** These break naive `ToSchema` derivation. Architect must flag them; CodeGen may need `#[schema(inline)]` or to refactor with care (do NOT change wire format).

- **`secrecy::SecretString` in responses.** Must never appear in a response DTO. If one does, that is a pre-existing bug — escalate to Reviewer rather than silently exposing it via the schema.

- **Schema list drift.** The `components(schemas(...))` and `paths(...)` lists in `openapi.rs` are append-only across six phases. Risk of duplicate entries, missing entries, or wrong fully-qualified paths. Reviewer must reconcile counts.

- **Build time.** `utoipa` derive expansion adds noticeable compile cost. After Phase 3D the cold build may grow by 30–60s. Acceptable but worth noting if CI has tight timeouts.

- **`metrics.rs` auth posture.** This may be a Prometheus scrape endpoint and must NOT require cookie_session — Architect to confirm before CodeGen tags it.

- **No frontend contract change.** This work is purely additive to the OpenAPI document. No `ContractGen` phase is needed because no new endpoints are introduced — existing handler shapes are unchanged. The frontend continues to consume the same JSON it does today; the OpenAPI spec is a parallel artifact.

## SQL Stored Procedures Needed

None. This workflow is documentation-only; no database changes are required for any phase.
