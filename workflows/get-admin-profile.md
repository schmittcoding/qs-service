# Workflow: GET Admin Profile

## Feature Summary

Add a `GET /admin/v1/profile` endpoint that returns the authenticated admin's profile information (name, user ID, and role) based on their valid JWT bearer token. This allows the admin frontend to display the current admin's details after authentication. The endpoint validates JWT, retrieves admin profile from WebDB, and returns the data as a JSON response.

## Agents Involved

1. **Architect** — Explore existing admin auth patterns, identify the data flow from JWT extraction to admin profile retrieval, determine if new stored procedures are needed.
2. **SQLGen** — Generate a stored procedure (if needed) to fetch admin profile details by user ID from WebDB.
3. **CodeGen** — Generate the route handler in `ran-access-point/src/routes/admin/auth/mod.rs` with proper JWT middleware integration.
4. **Tester** — Create unit and integration tests for JWT validation, admin profile retrieval, and edge cases (invalid token, missing user, etc.).
5. **OpenAPI Validator** — Ensure the route handler has correct utoipa documentation, proper endpoint definition, and response schemas.
6. **Reviewer** — Review the implementation for security (JWT claims validation), pattern consistency, and error handling.
7. **ContractGen** — Generate TypeScript types for the admin profile response and document the API contract for frontend consumption.

## Implementation Workflow

### Phase 1 — Architect
**Prompt:**

Analyze the gs-api codebase to design the `GET Admin Profile` endpoint. This endpoint should return an authenticated admin's profile information based on their JWT bearer token.

**Context:**
- Workspace: 3 crates (`ran-access-point`, `ran-db-core`, `ran-models`)
- Databases: 5 MSSQL databases (WebDB, GameDB, ShopDB, UserDB, LogDB)
- Auth pattern: Admins use JWT Bearer tokens validated via `JwtMiddleware` in `ran-access-point/src/services/jwt.rs`
- Existing admin auth: `ran-access-point/src/routes/admin/auth/mod.rs` handles admin authentication and JWT generation
- Admin models: `ran-models/src/admin/auth/mod.rs` contains `AuthAdminProfileData` struct with user_name, user_id, and user_type

**Your task:**

1. **Explore existing patterns:**
   - Read `ran-access-point/src/routes/admin/auth/mod.rs` to understand the current admin authentication flow and JWT generation
   - Check `ran-db-core/src/webmanager/admin/auth/mod.rs` to see the `WebAdminTrait::authenticate_admin` method
   - Review `ran-models/src/admin/auth/mod.rs` to understand the `AuthAdminProfileData` struct and what fields are available
   - Check how JWT middleware extracts claims in `ran-access-point/src/services/jwt.rs` (look for the `Claims` struct and middleware implementation)

2. **Design the endpoint:**
   - Endpoint path: `GET /admin/v1/profile`
   - URL structure: `/profile` under the `/admin/v1/auth` scope
   - Authentication: JWT Bearer token (validated by `JwtMiddleware`)
   - Request: No request body; token comes from Authorization header
   - Response: JSON object with admin profile data (user_id, user_name, user_type)
   - Database: WebDB (use existing methods or design new stored procedure)

3. **Identify data flow:**
   - JWT middleware extracts Bearer token from Authorization header
   - Middleware decodes JWT and extracts `Claims` (sub, role, exp)
   - Route handler receives request with `HttpRequest` containing JWT claims
   - Handler extracts user_id from JWT claims
   - Handler calls new DB method to fetch admin full profile (or reuse existing data)
   - Handler returns admin profile as JSON response

4. **Determine DB operations needed:**
   - Can the endpoint reuse `authenticate_admin` result (which already returns `AuthAdminProfileData`)?
   - If JWT claims already contain sufficient data (user_id, role), might not need a new DB call
   - If additional admin profile details are needed (email, created_date, last_login, etc.), design a new stored procedure `sp_GetAdminProfile` or similar
   - List exact input parameters (user_id) and output columns (user_name, user_id, user_type, and any others)

5. **Identify edge cases and validation:**
   - Invalid or expired JWT token (should be handled by JwtMiddleware before reaching handler)
   - Admin user_id not found in WebDB (return 404)
   - Database connection failure (return 500)
   - Empty or missing Authorization header (middleware handles, but handler should assume valid claims)

6. **Output a summary with:**
   - File paths that need to be created/modified
   - New stored procedures (if any) with names, input params, output columns
   - Struct names for request/response (models in `ran-models`)
   - Trait method name(s) needed in `ran-db-core`
   - Route handler name and full path
   - Data flow diagram (high-level steps from JWT to response)

### Phase 2 — SQLGen
**Depends on:** Architect output

**Prompt:**

Based on the Architect's analysis, generate SQL stored procedures for the admin profile endpoint.

**Input from Architect:**
The Architect will provide:
- Whether a new stored procedure is needed (e.g., `sp_GetAdminProfile`)
- If needed: procedure name, target database (WebDB), input parameters with types, expected output columns

**Your task:**

1. **Review existing procedures:**
   - Read `db-scripts/webdb/Procedures/` to understand naming conventions and parameter patterns used in WebDB stored procedures
   - Check if there's already a procedure that fetches admin user details that can be reused

2. **Generate new procedure (if required):**
   - Procedure name: `sp_GetAdminProfile` (or as specified by Architect)
   - Target database: WebDB
   - Input parameter: `@UserID` (UNIQUEIDENTIFIER or STRING - match the user_id type stored in the database)
   - Output columns: at minimum `user_name`, `user_id`, `user_type`; any additional fields the admin profile should include
   - The procedure should query the admin user table/view with appropriate filtering and return a single row
   - Save to `db-scripts/webdb/Procedures/sp_GetAdminProfile.sql`

3. **Follow conventions:**
   - Match existing procedure naming (sp_* prefix)
   - Match parameter naming (@ParameterName)
   - Include proper error handling and null checks
   - Return appropriate result sets with exact column names

4. **Output summary:**
   - Exact procedure name, database, and full signature
   - Input parameter names and types
   - Output column names, types, and order
   - File path where procedure is saved

### Phase 3 — CodeGen
**Depends on:** Architect output, SQLGen output (if procedure was generated)

**Prompt:**

Generate the route handler and database trait method for the admin profile endpoint.

**Context:**
- Crate structure: `ran-access-point` for handlers, `ran-db-core` for database traits and implementations
- Auth pattern: JWT middleware injects `Claims` into `HttpRequest` extensions
- Handler registration: Feature modules export `pub fn configure(cfg: &mut web::ServiceConfig)` that registers route handlers

**Your task:**

1. **Explore existing patterns:**
   - Read `ran-access-point/src/routes/admin/auth/mod.rs` to see handler pattern with JWT and responses
   - Check `ran-access-point/src/services/jwt.rs` to understand the `Claims` struct and how to extract it from the request
   - Read `ran-db-core/src/webmanager/admin/auth/mod.rs` to see the async trait pattern and how methods are implemented
   - Examine a similar feature (e.g., user profile endpoint) for response structure patterns

2. **Create/update database trait in `ran-db-core/src/webmanager/admin/auth/mod.rs`:**
   - Add a new method to `WebAdminTrait`: `async fn get_admin_profile(&self, user_id: &str) -> Result<AuthAdminProfileData>`
   - If no new stored procedure was needed, this might simply return data already known (from JWT)
   - If a new procedure exists, implement it to call the stored procedure with the user_id parameter
   - Use `.try_get::<Type, _>("ColumnName")` for manual row mapping
   - Use `anyhow::Context` for error messages (e.g., "Failed to fetch admin profile")

3. **Create route handler in `ran-access-point/src/routes/admin/auth/mod.rs`:**
   - Handler function: `async fn get_profile(...) -> impl Responder`
   - Attributes: `#[get("/profile")]` and proper utoipa documentation (see below for Phase 5)
   - Extract JWT claims from `HttpRequest` (use `req.extensions()` or similar to get the Claims injected by JwtMiddleware)
   - Validate claims exist and extract user_id
   - Call `db.web_db.get_admin_profile(&user_id).await`
   - Match the result:
     - Ok(profile) → return `HttpResponse::Ok().json(profile)`
     - Err(e) → log error with `tracing::error!` and return appropriate HTTP status (500 for DB errors, 404 if profile not found)
   - Register the handler in the `configure()` function: `cfg.service(get_profile);`

4. **Response format:**
   - Return the `AuthAdminProfileData` struct (already exists in `ran-models/src/admin/auth/mod.rs`)
   - Ensure it serializes to JSON with fields: `user_name`, `user_id`, `user_type`
   - Confirm struct has `#[derive(Serialize, Deserialize, utoipa::ToSchema)]` (it should already have these)

5. **Output summary:**
   - File paths modified: `ran-db-core/src/webmanager/admin/auth/mod.rs`, `ran-access-point/src/routes/admin/auth/mod.rs`
   - New trait method name and signature
   - New handler function name, route path, and expected response format
   - Full URL path: `/admin/v1/auth/profile`

### Phase 4 — Tester
**Depends on:** CodeGen output

**Prompt:**

Create unit and integration tests for the admin profile endpoint.

**Context:**
- Test location: `ran-access-point/tests/` for integration tests, crate modules for unit tests
- Testing framework: Rust's built-in test framework with possible integration test helpers
- High-risk feature: Admin auth is a critical feature; comprehensive test coverage is required

**Your task:**

1. **Identify test scenarios:**
   - Valid JWT token → returns admin profile (200 OK)
   - Invalid/expired JWT token → returns 401 Unauthorized (handled by middleware)
   - Missing Authorization header → returns 401 Unauthorized (handled by middleware)
   - Valid JWT but admin user not found in DB → returns 404 Not Found or 500 (as per implementation)
   - Database connection failure → returns 500 Internal Server Error

2. **Create tests:**
   - Write integration tests that validate the full request-response cycle
   - Mock or use a test database if integration tests are feasible
   - Test JWT extraction from headers
   - Test database trait implementation with both success and error scenarios
   - Validate response JSON structure matches expected schema

3. **Output:**
   - List of test files and test function names
   - Test coverage summary (which scenarios are covered)

### Phase 5 — OpenAPI Validator
**Depends on:** CodeGen output

**Prompt:**

Validate OpenAPI/Swagger documentation for the admin profile endpoint.

**Context:**
- Documentation tool: `utoipa` crate with `#[utoipa::path(...)]` macro for handlers
- Endpoint: `GET /admin/v1/profile`
- Expected responses: 200 (success), 401 (unauthorized), 500 (error)

**Your task:**

1. **Review the handler documentation:**
   - Ensure the handler has a `#[utoipa::path(...)]` attribute with:
     - HTTP method: `get`
     - Path: `/admin/v1/profile`
     - Operation ID: `get_admin_profile` (or similar)
     - Tag: `Admin - Auth`
     - Responses: 
       - 200: Success response with body `AuthAdminProfileData`
       - 401: Unauthorized (invalid/missing JWT)
       - 500: Internal server error
   - Verify `AuthAdminProfileData` struct has `#[derive(..., utoipa::ToSchema)]`

2. **Check handler registration:**
   - Verify the handler is registered in `configure()` function
   - Ensure it's properly scoped under `/admin/v1/auth` scope
   - Validate that route handlers are registered in the main `routes/mod.rs`

3. **Run validation:**
   - Run `cargo check -p ran-access-point` to ensure no compile errors
   - Review OpenAPI spec output to confirm endpoint appears correctly

4. **Output:**
   - List any missing or incorrect utoipa annotations
   - Confirm OpenAPI spec generation passes
   - Flag any schema issues

### Phase 6 — Reviewer
**Depends on:** All prior phases complete

**Prompt:**

Review the admin profile endpoint implementation for correctness, security, and pattern consistency.

**Files to review:**
- `ran-db-core/src/webmanager/admin/auth/mod.rs` (new/updated trait method)
- `ran-access-point/src/routes/admin/auth/mod.rs` (new handler)
- `db-scripts/webdb/Procedures/sp_GetAdminProfile.sql` (if created)

**Focus areas:**

1. **Security:**
   - JWT claims are validated (middleware responsibility, but handler should trust claims)
   - User ID from JWT is used for all profile lookups (no user input directly accessing database)
   - No sensitive data leakage in error messages
   - Proper error responses without exposing internal details

2. **Pattern consistency:**
   - Handler follows the same pattern as `authenticate` in the same file
   - Database trait method follows async trait pattern with proper error handling
   - Response format matches existing admin endpoints (e.g., JSON with consistent structure)
   - Naming conventions: snake_case functions, PascalCase types
   - Proper use of `anyhow::Context` for error messages in DB methods

3. **Error handling:**
   - Database errors are caught and logged with `tracing::error!`
   - Appropriate HTTP status codes returned (401, 404, 500)
   - User-facing error messages are concise and non-technical

4. **Code quality:**
   - No `unwrap()` calls; all errors handled with `.context()` or pattern matching
   - No `println!` or `eprintln!` calls; use `tracing`
   - Comments only where logic is non-obvious (usually none needed)
   - Imports are clean and organized

5. **Tests:**
   - Test coverage includes success and failure paths
   - Tests validate auth, DB interactions, and response format

6. **Run validation:**
   - `cargo check -p ran-db-core` — no compile errors
   - `cargo check -p ran-access-point` — no compile errors
   - `cargo clippy` — no clippy warnings
   - `cargo test` — all tests pass

7. **Output:**
   - Summary of any issues found
   - Approval or list of required fixes before moving to ContractGen

### Phase 7 — ContractGen
**Depends on:** Reviewer sign-off

**Prompt:**

Generate TypeScript types and API contract documentation for the admin profile endpoint.

**Context:**
- Output file: `workflows/get-admin-profile-contract.md`
- Rust models: `ran-models/src/admin/auth/mod.rs` contains `AuthAdminProfileData`
- Route handler: `ran-access-point/src/routes/admin/auth/mod.rs` (GET /admin/v1/profile)
- Note: There is no separate admin route directory; admin routes are mixed with feature routes under `admin/` structure

**Your task:**

1. **Locate and read the Rust models:**
   - File: `ran-models/src/admin/auth/mod.rs`
   - Struct: `AuthAdminProfileData`
   - Note any `#[serde(...)]` attributes (renames, skips, defaults)
   - Identify all fields, their types, and descriptions

2. **Generate TypeScript types:**
   - Convert `AuthAdminProfileData` to a TypeScript `type` (NOT interface)
   - Use the exact Rust field names (accounting for any `#[serde(rename = "...")]` attributes)
   - For UUID fields, use `string`; for i32/i64 integers, use `number`; for String, use `string`
   - Example structure:
     ```typescript
     export type AuthAdminProfileData = {
       user_name: string;
       user_id: string;
       user_type: number;
     };
     ```

3. **Document the API contract:**
   - Endpoint: `GET /admin/v1/profile`
   - Authentication: Bearer token (JWT) required in `Authorization: Bearer {token}` header
   - Request: No request body
   - Response (200):
     ```json
     {
       "user_name": "string",
       "user_id": "string",
       "user_type": "number"
     }
     ```
   - Response (401): Unauthorized (invalid/missing JWT)
   - Response (500): Internal server error

4. **Output structure:**
   - Save to: `workflows/get-admin-profile-contract.md`
   - Include TypeScript type definitions in a code block
   - Include API endpoint documentation with request/response examples
   - Note any special considerations for frontend integration (e.g., JWT storage, token refresh)

## Files Expected to Change

### New Files
- `db-scripts/webdb/Procedures/sp_GetAdminProfile.sql` (if new procedure needed)
- Test files in `ran-access-point/tests/` (if integration tests created)
- `workflows/get-admin-profile-contract.md` (ContractGen output)

### Modified Files
- `ran-db-core/src/webmanager/admin/auth/mod.rs` — Add `get_admin_profile` trait method and implementation
- `ran-access-point/src/routes/admin/auth/mod.rs` — Add `get_profile` handler and register it in `configure()`

## Risk Areas

- **JWT validation:** Admin tokens must be properly validated by middleware; endpoint should trust claims but validate token presence
- **Database connection:** Handle pool exhaustion and connection timeouts gracefully
- **Auth flows:** Admin profile endpoint must reject unauthenticated requests (middleware ensures this)
- **User lookup:** Ensure admin user exists in WebDB; return 404 if not found rather than 500
- **Data exposure:** Admin profile response should not leak sensitive internal data

## SQL Stored Procedures Needed

If new procedure is required (Architect will determine):

| Procedure Name       | Database | Input Parameters          | Output Columns                        |
|----------------------|----------|---------------------------|---------------------------------------|
| `sp_GetAdminProfile` | WebDB    | `@UserID` (UNIQUEIDENTIFIER or STRING) | `user_name` (VARCHAR), `user_id` (UNIQUEIDENTIFIER/STRING), `user_type` (INT) |
