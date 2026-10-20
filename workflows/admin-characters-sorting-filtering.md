# Workflow: Admin Characters Sorting & Filtering

## Feature Summary

Enhance the admin `GET /characters` endpoint with two new capabilities: (1) sorting by `cha_contribution_point`, `cha_level`, or `cha_money` with ascending/descending order, and (2) filtering by `cha_online` status (online=1 or offline=0). The stored procedure, Rust traits, models, and route handler will be updated to support these parameters.

## Agents Involved

1. **Architect** — Identify affected files, data flow, validation requirements, and stored procedure signature changes.
2. **SQLGen** — Update `sp_GetCharacters` to accept sort_by and online filter parameters and apply them in the WHERE and ORDER BY clauses.
3. **CodeGen** — Update `CharacterPaginationParams` model, `GameCharacterTrait::get_characters` signature, and the route handler.
4. **OpenAPI Validator** — Verify utoipa documentation includes the new `sort_by`, `sort_order`, and `cha_online` query parameters.
5. **Reviewer** — Review all changes for correctness, consistency, and edge cases.

## Implementation Workflow

### Phase 1 — Architect

**Prompt:**

You are analyzing an enhancement to the admin characters endpoint in the gs-api Rust backend (3 crates: ran-access-point, ran-models, ran-db-core; 5 MSSQL databases). The feature adds sorting and filtering to `GET /admin/v1/characters`.

**Current state:**
- Route handler: `ran-access-point/src/routes/admin/characters/mod.rs` — handler `get_characters`
- Model params: `ran-models/src/admin/mod.rs` — struct `CharacterPaginationParams`
- DB trait: `ran-db-core/src/gamemanager/admin/characters/mod.rs` — trait `GameCharacterTrait`, method `get_characters`
- Stored procedure: `db-scripts/gamedb/StoredProcedure/sp_GetCharacters.sql` — executes filtering, pagination, and sorting
- Current sorting: hardcoded `ORDER BY ChaCreateDate DESC` in the stored procedure
- Current filters: `class`, `school`, `status` (ChaDeleted), `search` (character/user name)

**New requirements:**
1. Add sort_by parameter: enum with values `contribution_point`, `level`, `money`
2. Add sort_order parameter: enum with values `asc` (ascending), `desc` (descending)
3. Add cha_online filter: integer 0 (offline) or 1 (online)
4. All parameters optional with sensible defaults (sort_by=contribution_point, sort_order=desc, cha_online=null means both)

**Your task:**
- Explore the codebase to understand the current pagination pattern and how filters are applied in the stored procedure
- Identify all files that must change: models, DB trait, stored procedure, route handler, utoipa docs
- Design the data flow: query params → model → Rust trait → SQL procedure → results
- Identify validation requirements (enum validation for sort_by/sort_order, online must be 0 or 1)
- Note any edge cases (NULL handling in SQL for optional online filter, ensure correct column names in ORDER BY)
- Output a summary including: (1) list of all files to create/modify, (2) new stored procedure signature, (3) new model struct fields, (4) validation strategy, (5) any risks

Do NOT write code. Only plan and analyze.

### Phase 2 — SQLGen

**Depends on:** Architect output

**Prompt:**

You are generating SQL for the gs-api MSSQL backend. The current stored procedure `sp_GetCharacters` in `db-scripts/gamedb/StoredProcedure/sp_GetCharacters.sql` must be updated to support sorting by contribution point, level, or money, and filtering by online status.

**Current procedure signature:**
```sql
CREATE OR ALTER PROCEDURE [dbo].[sp_GetCharacters]
    @SearchNum INT = NULL,
    @Search NVARCHAR(50) = NULL,
    @Class INT = NULL,
    @School NVARCHAR(MAX) = NULL,
    @Status INT = NULL,
    @Page INT = 1,
    @PageSize INT = 10
```

**New parameters to add:**
- `@SortBy NVARCHAR(20) = 'contribution_point'` — valid values: 'contribution_point', 'level', 'money'
- `@SortOrder NVARCHAR(4) = 'desc'` — valid values: 'asc', 'desc'
- `@ChaOnline INT = NULL` — valid values: 0 (offline), 1 (online), NULL (all)

**Required changes:**
1. Add the three new parameters to the procedure signature with defaults
2. Add WHERE clause condition to filter by ChaOnline when @ChaOnline IS NOT NULL
3. Replace the hardcoded `ORDER BY ChaCreateDate DESC` with dynamic ordering based on @SortBy and @SortOrder:
   - If @SortBy = 'contribution_point': ORDER BY ContributionPoint
   - If @SortBy = 'level': ORDER BY ChaLevel
   - If @SortBy = 'money': ORDER BY ChaMoney
   - If @SortOrder = 'asc': ASC, else DESC
4. Include all current filtering logic unchanged

**Output format:**
Write the updated stored procedure to `db-scripts/gamedb/StoredProcedure/sp_GetCharacters.sql`. After writing, output a summary table with:
- Procedure name: `sp_GetCharacters`
- New parameters: @SortBy (NVARCHAR(20), 'contribution_point'), @SortOrder (NVARCHAR(4), 'desc'), @ChaOnline (INT, NULL)
- Return columns (unchanged): ChaNum, ChaName, ChaClass, ChaSchool, ChaSex, ChaLevel, ChaMoney, ChaOnline, ChaCreateDate, ChaDeleted, ChaDeletedDate, ContributionPoint, UserNum, UserID
- Database: RanGameS1 (GameDB in Rust)

### Phase 3 — CodeGen

**Depends on:** Architect output, SQLGen output

**Prompt:**

You are generating Rust code for the gs-api backend. Three files must be updated to support the new sorting and online filtering on the admin characters endpoint.

**Files to modify:**

1. **`ran-models/src/admin/mod.rs`** — Update `CharacterPaginationParams`:
   - Add field: `pub sort_by: Option<String>` (user passes 'contribution_point', 'level', or 'money')
   - Add field: `pub sort_order: Option<String>` (user passes 'asc' or 'desc')
   - Add field: `pub cha_online: Option<i32>` (user passes 0 or 1)
   - All three optional, no validation here (SQL/trait will handle)
   - Keep existing fields unchanged

2. **`ran-db-core/src/gamemanager/admin/characters/mod.rs`** — Update trait `GameCharacterTrait`:
   - Update method signature `get_characters`:
     - Add parameter: `sort_by: Option<String>`
     - Add parameter: `sort_order: Option<String>`
     - Add parameter: `cha_online: Option<i32>`
   - Update the implementation `get_characters` in `impl GameCharacterTrait for GameDB`:
     - Pass the three new parameters to the stored procedure call
     - Convert None to sensible SQL defaults (sort_by='contribution_point', sort_order='desc', cha_online=NULL)
     - Example call: `"EXEC sp_GetCharacters @SearchNum = @P1, @Search = @P2, @Class = @P3, @School = @P4, @Status = @P5, @Page = @P6, @PageSize = @P7, @SortBy = @P8, @SortOrder = @P9, @ChaOnline = @P10"`

3. **`ran-access-point/src/routes/admin/characters/mod.rs`** — Update handler `get_characters`:
   - Extract `sort_by`, `sort_order`, `cha_online` from `CharacterPaginationParams`
   - Pass them to `db.game_db.get_characters(...)` call
   - Default values: sort_by='contribution_point', sort_order='desc', cha_online=None
   - Update utoipa docs (Phase 4 will validate)

**Code style:**
- Follow existing patterns: use `.map()` and `.unwrap_or()` for Option handling
- No new validation logic; defer to stored procedure
- Match indentation and naming conventions in existing code
- Keep changes minimal and focused

**Output:**
Write all three files with full content. Do NOT skip unchanged sections — provide complete files.

### Phase 4 — OpenAPI Validator

**Depends on:** CodeGen output

**Prompt:**

You are validating OpenAPI documentation for the admin characters endpoint in gs-api using utoipa.

**File to validate:** `ran-access-point/src/routes/admin/characters/mod.rs` — function `get_characters` with its `#[utoipa::path(...)]` macro.

**Current params in utoipa docs:**
- page (Option<usize>)
- limit (Option<usize>)
- class (Option<i32>)
- school (Option<Vec<i32>>)
- status (Option<i16>)
- search (Option<String>)

**New params that MUST be added to the utoipa docs:**
- sort_by (Option<String>): description = "Sort by field: 'contribution_point', 'level', or 'money'"
- sort_order (Option<String>): description = "Sort order: 'asc' or 'desc'"
- cha_online (Option<i32>): description = "Filter by online status: 1 (online) or 0 (offline)"

**Task:**
- Verify that all three new parameters are present in the `params(...)` section of the utoipa macro
- Verify that descriptions are clear and match the implementation
- Run `cargo check -p ran-access-point` to ensure the macro compiles without errors
- Verify no compilation warnings related to utoipa

**Output:**
Report: Are all three parameters documented? Are there any utoipa compilation errors or warnings? If missing or incorrect, list what needs to be fixed. Do NOT modify code yourself — only validate.

### Phase 5 — Reviewer

**Depends on:** CodeGen output, OpenAPI Validator sign-off

**Prompt:**

You are reviewing the admin characters endpoint enhancement in gs-api for correctness, security, pattern consistency, and edge cases.

**Files modified:**
- `ran-models/src/admin/mod.rs` — CharacterPaginationParams
- `ran-db-core/src/gamemanager/admin/characters/mod.rs` — GameCharacterTrait and impl
- `ran-access-point/src/routes/admin/characters/mod.rs` — handler and utoipa docs
- `db-scripts/gamedb/StoredProcedure/sp_GetCharacters.sql` — stored procedure

**Review focus areas:**

1. **Data flow correctness:**
   - Do query params flow correctly from handler → trait → stored procedure?
   - Are all parameters passed to the SQL EXEC statement?
   - Are defaults applied correctly (contribution_point, desc, NULL for online)?

2. **SQL safety & correctness:**
   - Does the stored procedure handle NULL values for @ChaOnline correctly (no filtering when NULL)?
   - Does the dynamic ORDER BY logic correctly map 'contribution_point' → ContributionPoint, 'level' → ChaLevel, 'money' → ChaMoney?
   - Does CASE/IF logic for @SortOrder work (asc vs desc)?
   - No SQL injection vectors (user-provided sort_by/sort_order are validated in SQL)?

3. **Pattern consistency:**
   - Does the code follow gs-api conventions (async traits, anyhow::Result, stored procedures)?
   - Are parameter names consistent with surrounding code (cha_online, sort_by, sort_order)?
   - Does error handling match existing code?

4. **Edge cases:**
   - What happens if sort_by is an invalid value? (Should SQL reject or default?)
   - What happens if sort_order is invalid? (Should SQL reject or default?)
   - What if cha_online is not 0 or 1? (Should SQL reject or accept as NULL?)
   - Pagination: are OFFSET/FETCH applied after sorting?

5. **Performance:**
   - Are there any N+1 queries or unnecessary round-trips?
   - Is the indexing on ChaLevel, ChaMoney, ContributionPoint adequate for sorting? (Note: review only, don't recommend index changes unless critical)

**Output:**
Provide a concise review:
1. ✅ or ❌ for each focus area
2. If ❌, list specific issues and line numbers
3. Any security or correctness concerns
4. Recommendations for fixes (if any)

Then run:
- `cargo check -p ran-db-core` — verify database trait compiles
- `cargo check -p ran-access-point` — verify handler compiles
- `cargo clippy -p ran-db-core` and `cargo clippy -p ran-access-point` — check for lint warnings

Report any errors or clippy warnings that need fixing.

## Files Expected to Change

### Models (ran-models)
- `ran-models/src/admin/mod.rs` — CharacterPaginationParams struct (add 3 optional fields)

### Database Layer (ran-db-core)
- `ran-db-core/src/gamemanager/admin/characters/mod.rs` — GameCharacterTrait trait definition and impl

### Routes (ran-access-point)
- `ran-access-point/src/routes/admin/characters/mod.rs` — get_characters handler and utoipa docs

### SQL (db-scripts)
- `db-scripts/gamedb/StoredProcedure/sp_GetCharacters.sql` — updated procedure with new parameters and dynamic ORDER BY

## Risk Areas

1. **SQL injection via sort_by/sort_order:** The stored procedure should only accept predefined values ('contribution_point', 'level', 'money' for sort_by; 'asc', 'desc' for sort_order). SQLGen must validate or use safe SQL logic (e.g., CASE statement, not string concatenation).
2. **NULL handling for cha_online filter:** When @ChaOnline is NULL, the filter should not apply (return all characters). Must verify the WHERE clause doesn't break with NULL.
3. **Column name correctness:** Map user-facing names ('contribution_point', 'level', 'money') to correct SQL column names (ContributionPoint, ChaLevel, ChaMoney).
4. **Pagination order:** Ensure ORDER BY is applied before OFFSET/FETCH, not after. MSSQL evaluates OFFSET/FETCH as part of ORDER BY clause, so this should be automatic.

## SQL Stored Procedures Needed

| Procedure Name | Database | Input Parameters | Output Columns |
|---|---|---|---|
| `sp_GetCharacters` | RanGameS1 (GameDB) | @SearchNum (INT, NULL), @Search (NVARCHAR(50), NULL), @Class (INT, NULL), @School (NVARCHAR(MAX), NULL), @Status (INT, NULL), @Page (INT, 1), @PageSize (INT, 10), **@SortBy** (NVARCHAR(20), 'contribution_point'), **@SortOrder** (NVARCHAR(4), 'desc'), **@ChaOnline** (INT, NULL) | ChaNum, ChaName, ChaClass, ChaSchool, ChaSex, ChaLevel, ChaMoney, ChaOnline, ChaCreateDate, ChaDeleted, ChaDeletedDate, ContributionPoint, UserNum, UserID |

**New parameters (bold):** @SortBy, @SortOrder, @ChaOnline
