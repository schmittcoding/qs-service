# Workflow: Add Last Login Date to Players Endpoints

## Feature Summary

Add the `last_login_date` field to the admin players API responses for both list and detail endpoints (`GET /admin/v1/players` and `GET /admin/v1/players/{id}`). The field will be sourced from `RanUser.dbo.GSUserInfo.LastLoginDate`, allowing admins to see when each player last logged into the game. This requires updating two stored procedures and the `UserAccountInfo` model.

## Agents Involved

1. **Architect** — Plan the changes across stored procedures and models
2. **SQLGen** — Update stored procedures to include `LastLoginDate` in SELECT statements
3. **CodeGen** — Add `last_login_date` field to the `UserAccountInfo` struct and update `from_row()` mapping
4. **OpenAPI Validator** — Verify route documentation includes the new field in schemas
5. **Reviewer** — Review all changes for consistency and correctness
6. **ContractGen** — Document the API contract change for frontend integration

## Implementation Workflow

### Phase 1 — Architect

**Prompt:**

You are architecting an enhancement to the admin players API in the gs-api Rust backend (3-crate workspace: `ran-models`, `ran-db-core`, `ran-access-point`; 5 databases: WebDB, GameDB, ShopDB, UserDB, LogDB).

**Feature:** Add `LastLoginDate` field to player responses in both `GET /admin/v1/players` (list) and `GET /admin/v1/players/{id}` (detail) endpoints.

**Requirements:**
- The `LastLoginDate` value comes from the `RanUser.dbo.GSUserInfo` table, column `LastLoginDate` (NaiveDateTime)
- It should be returned as `last_login_date` in JSON responses, formatted as `DateTime<Local>` (ISO 8601 with timezone)
- Both list and detail endpoints must return the field
- Must follow existing `UserAccountInfo` struct pattern

**Your analysis should include:**
1. Which stored procedures need modification:
   - `sp_GetPlayersList` (called by the list endpoint) — currently located at `db-scripts/webdb/Procedures/Administration/sp_GetPlayersList.sql`
   - `sp_GetPlayerById` (called by the detail endpoint) — currently located at `db-scripts/webdb/Procedures/Administration/sp_GetPlayerById.sql`
2. Which Rust model needs modification:
   - `UserAccountInfo` struct in `ran-models/src/admin/players/mod.rs` — currently has 10 fields, `from_row()` impl must be updated
3. Data flow: 
   - User makes request → Route handler (`ran-access-point/src/routes/admin/players/mod.rs`) → DB trait `get_players()` or `get_player_by_id()` → Stored procedure → Map result via `UserAccountInfo::from_row()` → Serialize to JSON
4. Edge cases:
   - `LastLoginDate` could be NULL in the database — handle gracefully with a default DateTime
   - No schema changes needed (table already has the column)
5. OpenAPI documentation impact:
   - Route handlers already have `#[utoipa::path(...)]` annotations with response schemas
   - The schema will need to include the new field automatically once the struct changes

**Deliverables:**
- A list of exactly which files need modification
- For each file: what specific changes (line ranges or sections) need to be made
- The column name (`LastLoginDate`) and its SQL type (DATETIME2 or similar)
- Confirmation that no new traits, routes, or stored procedures are needed

---

### Phase 2 — SQLGen

**Depends on:** Architect output

**Prompt:**

You are the SQL code generator for gs-api. You generate T-SQL stored procedures for 5 MSSQL databases (WebDB, GameDB, ShopDB, UserDB, LogDB).

**Task:** Update two existing stored procedures to include `LastLoginDate` in their SELECT statements.

**Stored Procedures to Modify:**

1. **sp_GetPlayersList**
   - Location: `db-scripts/webdb/Procedures/Administration/sp_GetPlayersList.sql`
   - Target database: WebDB
   - Current behavior: Returns list of players with pagination, filtering by username/email
   - Modification: Add `UI.LastLoginDate` to the SELECT statement in both STEP 1 (count query comment) and STEP 2 (paginated result set)
   - Source table: `RanUser.dbo.GSUserInfo UI` — column `LastLoginDate` (nullable DATETIME2)
   - Column should appear in the final SELECT output for both the count and data queries

2. **sp_GetPlayerById**
   - Location: `db-scripts/webdb/Procedures/Administration/sp_GetPlayerById.sql`
   - Target database: WebDB
   - Current behavior: Returns a single player by UserNum
   - Modification: Add `UI.LastLoginDate` to the SELECT statement
   - Source table: `RanUser.dbo.GSUserInfo UI` — column `LastLoginDate`

**Instructions:**
1. Read both existing stored procedure files to understand their current structure
2. Add `UI.LastLoginDate` to the SELECT columns in each procedure
3. Maintain the exact same structure and formatting as the original files
4. Both files already INNER JOIN with `RanUser.dbo.GSUserInfo UI`, so the column is already available
5. Save the updated files to their original locations

**Output a summary table with:**
- File path
- Procedure name
- New SELECT columns added (just list the new ones)

---

### Phase 3 — CodeGen

**Depends on:** Architect output, SQLGen output

**Prompt:**

You are the Rust code generator for gs-api. You generate models, DB traits, and route handlers matching existing patterns in the codebase.

**Task:** Update the `UserAccountInfo` struct and its `from_row()` implementation to include the `last_login_date` field.

**Context:**
- 3 crates: `ran-models`, `ran-db-core`, `ran-access-point`
- Target file: `ran-models/src/admin/players/mod.rs`
- The struct `UserAccountInfo` currently has 10 fields (see lines 6-40)
- It already derives `Debug, Serialize, Deserialize, Clone, utoipa::ToSchema`
- The `from_row()` implementation (lines 197-248) manually maps database columns via `row.get::<Type, _>("ColumnName")`

**Modification Details:**

1. **Add field to struct** (after line 34, before closing brace):
   - Field name: `pub last_login_date: DateTime<Local>`
   - Type: `DateTime<Local>` (consistent with `user_create_date` and `user_block_date`)
   - It will be serialized as ISO 8601 by serde

2. **Update from_row() implementation** (in the Self { ... } block around lines 206-247):
   - Add mapping for `last_login_date`:
     ```rust
     last_login_date: row
         .get::<NaiveDateTime, _>("LastLoginDate")
         .map(|naive| to_local(naive))
         .unwrap_or_default(),
     ```
   - The `to_local()` helper function (lines 199-204) already handles NaiveDateTime → DateTime<Local> conversion
   - Placed after `user_create_date` and before `user_login_state` (alphabetical-ish grouping with other datetime fields)

**Instructions:**
1. Read `ran-models/src/admin/players/mod.rs` to understand the exact structure
2. Add the `last_login_date: DateTime<Local>` field to `UserAccountInfo` struct
3. Add the mapping in the `from_row()` impl — use the same pattern as `user_create_date` and `user_block_date`
4. No changes needed to route handlers or DB traits — they will work as-is
5. Run `cargo check -p ran-models` to verify

**Output:**
- Confirm the field was added to the struct
- Confirm the from_row() mapping was added
- Run check and report any errors

---

### Phase 4 — OpenAPI Validator

**Depends on:** CodeGen output

**Prompt:**

You are validating OpenAPI documentation for gs-api route handlers. The app uses `utoipa` for automatic OpenAPI schema generation from Rust types.

**Task:** Verify that the new `last_login_date` field appears in the OpenAPI schemas for the two player endpoints.

**Endpoints to validate:**
1. `GET /admin/v1/players` — handler in `ran-access-point/src/routes/admin/players/mod.rs`, function `get_players()`
   - Response schema should automatically pick up the new field from `UserAccountInfo`
   - Verify the response body shows `last_login_date` in the schema
2. `GET /admin/v1/players/{id}` — handler in `ran-access-point/src/routes/admin/players/mod.rs`, function `get_player_by_id()`
   - Response schema should automatically pick up the new field from `UserAccountInfo`
   - Verify the response body shows `last_login_date` in the schema

**Instructions:**
1. Read both route handlers to understand their `#[utoipa::path(...)]` annotations and response definitions
2. Build the project: `cargo build -p ran-access-point` (or cargo check)
3. If possible, generate and review the OpenAPI spec — the field should appear automatically since it's in the `UserAccountInfo` struct with `#[derive(utoipa::ToSchema)]`
4. Verify no manual schema updates are needed (utoipa derives schemas automatically from struct fields)

**Success criteria:**
- Both endpoints return HTTP 200 with body containing `last_login_date` field in JSON
- Field type is `string` (ISO 8601 datetime)
- No compilation errors

---

### Phase 5 — Reviewer

**Depends on:** All prior phases complete

**Prompt:**

You are reviewing Rust code for the gs-api backend for correctness, security, consistency, and patterns.

**Task:** Review all changes made for this feature (adding `last_login_date` to players endpoints).

**Files to review:**
1. `db-scripts/webdb/Procedures/Administration/sp_GetPlayersList.sql`
   - Verify `UI.LastLoginDate` was added to both SELECT statements (count + paginated results)
   - Check column alias or name consistency
   - Verify no SQL syntax errors
   
2. `db-scripts/webdb/Procedures/Administration/sp_GetPlayerById.sql`
   - Verify `UI.LastLoginDate` was added to the SELECT statement
   - Check column name consistency with sp_GetPlayersList
   - Verify no SQL syntax errors

3. `ran-models/src/admin/players/mod.rs`
   - Verify `last_login_date: DateTime<Local>` field was added to `UserAccountInfo` struct
   - Check field ordering and formatting consistency with other datetime fields
   - Verify the `from_row()` implementation correctly maps `LastLoginDate`:
     - Uses `row.get::<NaiveDateTime, _>("LastLoginDate")`
     - Applies `to_local()` conversion
     - Uses `.unwrap_or_default()` for NULL safety
   - Ensure struct still derives `Serialize, Deserialize, utoipa::ToSchema`

**Review focus areas:**
- **Consistency:** Field naming (snake_case in Rust, PascalCase in SQL) matches existing patterns
- **Error handling:** NULL handling via `.unwrap_or_default()` is appropriate
- **Pattern matching:** Follows exact same pattern as `user_create_date` and `user_block_date`
- **Type correctness:** `DateTime<Local>` is the right type for admin-facing timestamps
- **No unintended side effects:** Only SELECT changed, no WHERE/JOIN/parameter changes

**Validation:**
1. Run `cargo check -p ran-models` — should pass
2. Run `cargo check -p ran-db-core` — should pass
3. Run `cargo check -p ran-access-point` — should pass
4. Run `cargo clippy` — should report no new warnings
5. Verify the modified files are correctly saved

**Deliverable:**
- Approval checklist for each file
- Any issues found and how to resolve them
- Final sign-off that all changes are correct and ready for ContractGen

---

### Phase 6 — ContractGen

**Depends on:** Reviewer sign-off

**Prompt:**

You are the API contract generator. You translate Rust structs from `ran-models` into TypeScript types and document API contracts for the frontend team.

**Task:** Document the API contract for the two updated players endpoints now that they include `last_login_date`.

**Endpoints:**
1. `GET /admin/v1/players` — list endpoint
   - Route handler: `ran-access-point/src/routes/admin/players/mod.rs`, function `get_players()`
   - Response shape: JSON object with `data` (array of `UserAccountInfo`), `page`, `limit`, `total_items`
   
2. `GET /admin/v1/players/{id}` — detail endpoint
   - Route handler: `ran-access-point/src/routes/admin/players/mod.rs`, function `get_player_by_id()`
   - Response shape: JSON object directly serialized from `UserAccountInfo`

**Model to translate:**
- `UserAccountInfo` from `ran-models/src/admin/players/mod.rs`
- Struct derives `Serialize, Deserialize, Clone`
- All fields serialize to lowercase snake_case JSON (no `#[serde(rename)]` overrides in the actual struct)
- DateTime<Local> fields serialize as ISO 8601 strings

**Instructions:**
1. Read the `UserAccountInfo` struct in `ran-models/src/admin/players/mod.rs` to get all 11 fields (including the new `last_login_date`)
2. Read both route handler signatures to understand request/response shapes
3. Generate TypeScript type definitions that match the Rust types:
   - All fields are non-optional (no `?`)
   - DateTime fields map to `string` (ISO 8601 format)
   - Numeric fields map to `number`
   - UUIDs map to `string`
4. Document the two endpoint schemas with examples
5. Note any non-obvious field mappings or behaviors

**Output file:** `workflows/add-last-login-date-to-players-contract.md`

**Format:**
- TypeScript interface for `UserAccountInfo`
- Request/response schemas for both endpoints
- Field-by-field documentation
- Example JSON responses

---

## Files Expected to Change

### SQL (db-scripts/)
- `db-scripts/webdb/Procedures/Administration/sp_GetPlayersList.sql`
  - Add `UI.LastLoginDate` to SELECT (appears in both STEP 1 and STEP 2)
  
- `db-scripts/webdb/Procedures/Administration/sp_GetPlayerById.sql`
  - Add `UI.LastLoginDate` to SELECT

### Rust Models (ran-models/)
- `ran-models/src/admin/players/mod.rs`
  - Add `last_login_date: DateTime<Local>` field to `UserAccountInfo` struct
  - Update `from_row()` impl to map `LastLoginDate` column

### No changes needed to:
- Route handlers (`ran-access-point/src/routes/admin/players/mod.rs`) — work as-is
- DB traits (`ran-db-core/src/webmanager/admin/players/mod.rs`) — work as-is
- OpenAPI annotations — automatically updated via `utoipa::ToSchema` derive

## Risk Areas

**Low risk overall — this is a read-only data addition:**
- No schema changes (column already exists in GSUserInfo)
- No new stored procedures or traits
- No change to query logic or filtering
- Only adding a field to existing response DTOs

**Minimal concerns:**
- NULL handling: `LastLoginDate` may be NULL for users who've never logged in — handled via `.unwrap_or_default()` which creates a default DateTime
- Serialization: DateTime<Local> serializes to ISO 8601 strings automatically via serde

## SQL Stored Procedures Needed

**No new procedures — modifying existing ones:**

1. **sp_GetPlayersList** (existing, WebDB)
   - Modification: Add `UI.LastLoginDate` to SELECT
   - Input: `@SearchNum (BIGINT)`, `@Search (NVARCHAR)`, `@UserType (NVARCHAR)`, `@UserStatus (NVARCHAR)`, `@Page (INT)`, `@PageSize (INT)`
   - Output columns (existing): UserID, UserNum, UserName, UserType, UserLoginState, UserAvailable, UserBlock, UserBlockDate, UserEmail, UserVerified, WebPoints, CreateDate
   - **New output column:** LastLoginDate (DATETIME2, may be NULL)

2. **sp_GetPlayerById** (existing, WebDB)
   - Modification: Add `UI.LastLoginDate` to SELECT
   - Input: `@UserNum (INT)`
   - Output columns (existing): UserID, UserNum, UserName, UserType, UserLoginState, UserAvailable, UserBlock, UserBlockDate, CreateDate, UserEmail, WebPoints, UserVerified
   - **New output column:** LastLoginDate (DATETIME2, may be NULL)
