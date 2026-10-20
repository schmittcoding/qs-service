# Workflow: Admin Players Login History

## Feature Summary

Add a new admin endpoint `GET /admin/v1/players/{id}/login-history` that returns paginated login/logout history for a specific player. This endpoint retrieves data from the `RanUser.dbo.GSLogLogin` table, sorted by LogDate descending, and includes login timestamp, IP address, and HWID for each event.

## Agents Involved

1. **Architect** — Analyze the codebase, identify the UserDB connection, examine similar pagination patterns (ban_account_history, account_transaction_history), design the data flow and stored procedure signature.
2. **SQLGen** — Generate stored procedure `sp_GetUserLoginHistory` in UserDB to fetch login records with pagination.
3. **CodeGen** — Create the route handler in `ran-access-point/src/routes/admin/players/mod.rs` and add the login history response DTO to `ran-models/src/admin/mod.rs`.
4. **OpenAPI Validator** — Validate utoipa documentation on the new route handler.
5. **Reviewer** — Review code for correctness, security, pattern consistency, and error handling.
6. **ContractGen** — Generate TypeScript types for frontend integration.

## Implementation Workflow

### Phase 1 — Architect

**Prompt:**

Analyze the gs-api codebase for the new endpoint `GET /admin/v1/players/{id}/login-history`. Your output should be a design document covering:

1. **Data Flow:**
   - Which database: UserDB (contains `RanUser.dbo.GSLogLogin`)
   - Which crate handles UserDB queries: `ran-db-core`
   - Auth mechanism: JWT Bearer token (admin-only), pattern from `/admin/v1/players` endpoints

2. **Codebase Exploration:**
   - Read `ran-access-point/src/routes/admin/players/mod.rs` to understand the existing admin/players endpoints (get_players, get_player_by_id, ban_account_history, account_transaction_history)
   - Note how pagination is implemented in `account_transaction_history` (line 257–328) and `ban_account_history` (line 211–236) — extract page/limit from query, call DB trait, map rows to DTO, return JSON with pagination metadata
   - Look at `ran-models/src/admin/mod.rs` to see existing admin DTOs and pagination param structs
   - Check `ran-db-core/src/usermanager/mod.rs` to understand the UserDB trait structure and how to add a new async method

3. **Stored Procedure Design:**
   - Procedure name: `sp_GetUserLoginHistory`
   - Database: UserDB (`RanUser.dbo`)
   - Source table: `GSLogLogin` with columns: LoginNum (BigInt), UserNum (Int), UserID (Varchar(50)), LogInOut (Int: 0=Logout, 1=Login), LogDate (DateTime), LogIpAddress (Varchar(23)), HWID (Varchar(65))
   - Input parameters: @UserNum (Int), @Page (Int), @Limit (Int)
   - Output: Total record count + paginated rows sorted by LogDate DESC
   - No search or filter logic needed

4. **Response Model Design:**
   - Create a DTO for a single login history record: `UserLoginHistoryRecord` with fields: login_num, user_id, login_logout (0 or 1), log_date (DateTime), log_ip_address, hwid
   - Response shape (from pattern in account_transaction_history): `{ "page": i16, "limit": i16, "total_items": i64, "data": Vec<UserLoginHistoryRecord> }`

5. **Files to Create/Modify:**
   - New: `db-scripts/userdb/Procedures/sp_GetUserLoginHistory.sql`
   - Modify: `ran-db-core/src/usermanager/mod.rs` — add `get_user_login_history` method to UserDB trait + impl
   - Modify: `ran-models/src/admin/mod.rs` — add `UserLoginHistoryRecord` DTO and reuse `BasicPaginationParams`
   - Modify: `ran-access-point/src/routes/admin/players/mod.rs` — add `get_login_history` handler function + register in `configure()`

6. **Risk Areas:**
   - Standard read-only operation, low risk
   - Ensure pagination params are validated (positive page/limit)
   - Error handling: connection failures, missing user, no records

Output a structured summary with file paths, data flow diagram (text), and the exact stored procedure signature.

### Phase 2 — SQLGen

**Depends on:** Architect output

**Prompt:**

Generate a new MSSQL stored procedure based on the Architect's design. Create:

**Procedure:** `sp_GetUserLoginHistory`
- **Database:** UserDB (`RanUser.dbo`)
- **Source Table:** `GSLogLogin` (LoginNum BigInt, UserNum Int, UserID Varchar(50), LogInOut Int, LogDate DateTime, LogIpAddress Varchar(23), HWID Varchar(65))
- **Input Parameters:**
  - `@UserNum` (Int) — the player's user ID (required)
  - `@Page` (Int) — page number (required, 1-based)
  - `@Limit` (Int) — records per page (required)
- **Output Rows:** Select LoginNum, UserNum, UserID, LogInOut, LogDate, LogIpAddress, HWID
- **Sort:** By LogDate DESC
- **Pagination:** Use OFFSET/FETCH to implement (OFFSET (@Page - 1) * @Limit ROWS FETCH NEXT @Limit ROWS ONLY)
- **Also Return:** Total record count for the user (as a second result set or as a column in the first result set — match the existing pattern in userdb procedures)

**Steps:**
1. Read existing procedures in `db-scripts/userdb/Procedures/` to understand the naming, parameter conventions, and pagination patterns used in this database
2. Read `db-scripts/webdb/Procedures/Administration/sp_GetPlayersList.sql` and `sp_GetAccountTransactionHistory.sql` as templates for paginated result sets
3. Write the procedure to `db-scripts/userdb/Procedures/sp_GetUserLoginHistory.sql`
4. Output a summary table with: procedure name, database, exact parameter names, return columns, and where the file is saved

### Phase 3 — CodeGen

**Depends on:** Architect output, SQLGen output (procedure name: `sp_GetUserLoginHistory`, parameters: @UserNum, @Page, @Limit)

**Prompt:**

Generate Rust code to implement the login history endpoint. Create/modify:

1. **New DTO in `ran-models/src/admin/mod.rs`:**
   - Struct `UserLoginHistoryRecord` with fields:
     - `login_num: i64`
     - `user_id: String`
     - `login_logout: i16` (0=Logout, 1=Login)
     - `log_date: String` (DateTime as ISO string)
     - `log_ip_address: String`
     - `hwid: String`
   - Derive: `Serialize`, `Deserialize`, `Debug`, `Clone`
   - Add `impl UserLoginHistoryRecord { fn from_row(row: &Row) -> Self { ... } }` to manually map from tiberius Row

2. **DB Trait Method in `ran-db-core/src/usermanager/mod.rs`:**
   - Add async trait method: `async fn get_user_login_history(&self, user_num: i64, page: i16, limit: i16) -> Result<PaginatedResult<UserLoginHistoryRecord>>`
   - PaginatedResult should contain `{ total_records: i64, data: Vec<UserLoginHistoryRecord> }`
   - Implementation: Get DB connection, exec `EXEC sp_GetUserLoginHistory @P1 = @user_num, @P2 = @page, @P3 = @limit`, map rows, return paginated result
   - Use `.context("Failed to fetch user login history")` for error propagation

3. **Route Handler in `ran-access-point/src/routes/admin/players/mod.rs`:**
   - Function name: `get_login_history`
   - Path: `GET /{id}/login-history`
   - Parameters: path (user_id: i64), query (BasicPaginationParams with page/limit)
   - utoipa documentation: operation_id = "admin_get_user_login_history", tag = "Admin - Players", security = JWT
   - Handler logic:
     - Extract user_id from path
     - Extract page/limit from query, default to 1/10
     - Call `db.user_db.get_user_login_history(user_id, page, limit).await`
     - Map result to JSON response: `{ "page", "limit", "total_items", "data" }`
     - Handle errors with InternalServerError
   - Register in `configure()` function: `cfg.service(get_login_history)`

**Reference existing handlers:**
- `ban_account_history` (line 211–236) for pagination pattern
- `account_transaction_history` (line 257–328) for detailed response shaping
- Both are JWT-protected admin routes in the same module

Output the generated code files as structured Rust modules.

### Phase 4 — OpenAPI Validator

**Depends on:** CodeGen output

**Prompt:**

Validate the OpenAPI documentation on the new `get_login_history` route handler in `ran-access-point/src/routes/admin/players/mod.rs`. Check:

1. **utoipa annotations present:**
   - `#[utoipa::path(...)]` with complete metadata
   - `operation_id`, `tag`, `security` defined
   - `params` section includes path param (id: i64) and query params (page, limit)
   - `responses` section covers 200, 400, 401, 500 status codes with proper descriptions

2. **Response schema:**
   - Response body is documented as `serde_json::Value` or a concrete model type
   - All response fields are clear in the description or via the body type

3. **Run checks:**
   - Execute `cargo check -p ran-access-point` to ensure no compilation errors
   - Execute `cargo clippy -p ran-access-point` and report any warnings

Output a validation report with any missing or incorrect documentation, and confirm the route is properly registered in the module's `configure()` function.

### Phase 5 — Reviewer

**Depends on:** OpenAPI Validator sign-off

**Prompt:**

Review the implementation for correctness, security, and pattern consistency. Check the files:
- `db-scripts/userdb/Procedures/sp_GetUserLoginHistory.sql`
- `ran-db-core/src/usermanager/mod.rs` (new get_user_login_history method)
- `ran-models/src/admin/mod.rs` (new UserLoginHistoryRecord DTO)
- `ran-access-point/src/routes/admin/players/mod.rs` (new get_login_history handler)

**Focus areas:**
1. **Security:** JWT auth is enforced, no SQL injection, input validation
2. **Pattern consistency:** Handler follows the style of ban_account_history and account_transaction_history, trait method uses `.context()` for errors, DTO derives correct traits
3. **Error handling:** All `.await` calls are properly error-checked, connection failures are logged, user errors return appropriate HTTP status codes
4. **Database correctness:** Procedure pagination logic is correct, column mappings in from_row() match the procedure output
5. **Code quality:** No unwrap(), no println!, proper use of anyhow errors

**Run final checks:**
- `cargo check -p ran-db-core -p ran-models -p ran-access-point`
- `cargo clippy --all`

Output a detailed review with any corrections needed or sign-off for ContractGen.

### Phase 6 — ContractGen

**Depends on:** Reviewer sign-off

**Prompt:**

Generate TypeScript types for the new admin login history endpoint. Translate the Rust structs from `ran-models/src/admin/mod.rs` (specifically `UserLoginHistoryRecord` and the response shape) into TypeScript types.

**Output file:** `workflows/admin-players-login-history-contract.md`

**Tasks:**
1. Glob `ran-models/src/admin/mod.rs` and locate the `UserLoginHistoryRecord` struct and the response JSON shape
2. Create TypeScript types:
   - `UserLoginHistoryRecord` (type, not interface)
   - Response wrapper with pagination metadata
3. Document the endpoint contract:
   - Path: `GET /admin/v1/players/{id}/login-history`
   - Query params: `page`, `limit` (both optional, default 1 and 10)
   - Response status: 200 (success), 401 (unauthorized), 500 (error)
   - Example response JSON
4. Note any `#[serde(rename = "...")]` attributes on fields that affect JSON serialization

The output should be a complete, copy-paste-ready TypeScript contract for the frontend team.

## Files Expected to Change

| File | Crate | Type | Purpose |
|------|-------|------|---------|
| `db-scripts/userdb/Procedures/sp_GetUserLoginHistory.sql` | N/A | New | Stored procedure for paginated login history query |
| `ran-db-core/src/usermanager/mod.rs` | ran-db-core | Modified | Add `get_user_login_history` trait method + impl |
| `ran-models/src/admin/mod.rs` | ran-models | Modified | Add `UserLoginHistoryRecord` DTO |
| `ran-access-point/src/routes/admin/players/mod.rs` | ran-access-point | Modified | Add `get_login_history` route handler + registration |

## Risk Areas

- **Database:** Standard read-only query, low risk; ensure pagination parameters prevent negative page/limit values
- **Auth:** Verify JWT Bearer token is required and validated before handler executes
- **Performance:** Large login history tables could cause slow queries; stored procedure should use indexes on UserNum and LogDate

## SQL Stored Procedures Needed

| Name | Database | Input Parameters | Output |
|------|----------|------------------|--------|
| `sp_GetUserLoginHistory` | UserDB | `@UserNum` (Int), `@Page` (Int), `@Limit` (Int) | LoginNum, UserNum, UserID, LogInOut, LogDate, LogIpAddress, HWID (paginated, sorted by LogDate DESC) + total count |
