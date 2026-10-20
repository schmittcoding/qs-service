# Workflow: Admin Players Multi-Value Filtering

## Feature Summary

Upgrade the `GET /admin/v1/players` endpoint to accept multiple values for `status` and `category` filters using the established codebase pattern: comma-separated strings with a split function. Follow the exact implementation approach applied to `GET /admin/v1/item-shop`, which already handles multi-value filtering via `serde_qs` with non-strict mode and comma-separated parameters in stored procedures.

## Agents Involved

1. **Architect** — Analyze the current players endpoint and design the filtering upgrade
2. **SQLGen** — Modify `sp_GetPlayers` to accept comma-separated string parameters (or create if it doesn't exist)
3. **CodeGen** — Update `AdminPlayersFilterParams` DTO, update `WebPlayerTrait::get_players` to convert `Vec<i16>` to comma-separated strings, update handler to use `serde_qs` non-strict mode
4. **Reviewer** — Review all changes for correctness, security, and pattern consistency with item-shop implementation

## Decision: Pagination Base

Before starting Phase 2, decide on pagination base:
- **Option A (Recommended):** Keep 1-based (match current behavior) — handler passes `params.page.unwrap_or(1)`, SP uses `@Page * @PageSize` as offset
- **Option B:** Switch to 0-based (match item-shop) — handler passes `params.page.unwrap_or(0)`, frontend must be updated

**Recommendation:** Option A (keep 1-based) to minimize breaking changes. This is a non-blocking decision — proceed with implementation assuming Option A, and coordinate with Cedric if a future migration to 0-based is desired.

## Implementation Workflow

### Phase 1 — Architect
**Status:** ✅ Complete

**Analysis Summary:**

The Architect has completed a detailed analysis of the current implementation. Key findings:

**Current State:**
- **SP:** `sp_GetPlayersList` at `/db-scripts/webdb/Procedures/Administration/sp_GetPlayersList.sql`
  - Parameters: `@Search`, `@UserType SMALLINT`, `@UserStatus SMALLINT`, `@Page`, `@PageSize`
  - Returns 2 result sets: TotalRecords count, then paginated data
- **Trait:** `get_players` in `ran-db-core/src/webmanager/admin/players/mod.rs`
  - Current signature: `async fn get_players(&self, page: i16, page_size: i16, r#type: Option<i16>, status: Option<i16>, search: Option<String>) -> Result<BaseDataCountResult>`
- **Handler:** Uses `web::Query<PaginationParams>` (shared DTO)
- **Split Function:** WebDB does NOT have `fn_SplitInts` — must be created (copy from ShopDB)

**Proposed Changes:**
1. **New DTO** `AdminPlayersFilterParams` with `category: Option<Vec<i16>>`, `status: Option<Vec<i16>>`
2. **Updated SP** signature: `@UserType NVARCHAR(MAX)`, `@UserStatus NVARCHAR(MAX)` (comma-separated strings)
3. **Updated Trait** signature: accept `Option<Vec<i16>>` for category and status, convert to comma-separated strings
4. **Updated Handler:** Use `serde_qs::Config::new(5, false)` with `HttpRequest`, parse `AdminPlayersFilterParams`
5. **Create split function** in WebDB (copy from ShopDB)

**Files to Modify:**
- `db-scripts/webdb/Functions/fn_SplitInts.sql` — **CREATE**
- `db-scripts/webdb/Procedures/Administration/sp_GetPlayersList.sql` — Modify
- `ran-models/src/admin/mod.rs` — Add DTO
- `ran-db-core/src/webmanager/admin/players/mod.rs` — Update trait
- `ran-access-point/src/routes/admin/players/mod.rs` — Update handler

**Risk Areas:**
1. Pagination: Current SP is 1-based (page 1 skips first page). Item-shop is 0-based. Decision needed on which to use.
2. Two-result-set shape: SP returns count then data (vs item-shop's `COUNT(*) OVER()`). Keep as-is unless asked to unify.
3. Frontend coordination: Ensure both `?category[]=1` and `?category=1` syntax work for backward compat.
### Phase 2 — SQLGen
**Depends on:** Architect output

Create the split function in WebDB and modify `sp_GetPlayersList` to accept comma-separated string parameters for multi-value filtering.

**Key tasks:**
1. Create `/Users/schmitt/development/ran-online/gs/gs-api/db-scripts/webdb/Functions/fn_SplitInts.sql` by copying from `/db-scripts/shopdb/Functions/fn_SplitInts.sql` and changing `USE RanShop` to `USE RanWeb`
2. Modify `/db-scripts/webdb/Procedures/Administration/sp_GetPlayersList.sql`:
   - Change `@UserType SMALLINT` to `@UserType NVARCHAR(MAX)`
   - Change `@UserStatus SMALLINT` to `@UserStatus NVARCHAR(MAX)`
   - Add temp tables: `@UserTypes TABLE (Value INT)`, `@UserStatuses TABLE (Value INT)`
   - Add split parsing:
     ```sql
     IF @UserType IS NOT NULL
         INSERT INTO @UserTypes (Value) SELECT Value FROM dbo.fn_SplitInts(@UserType, ',');
     IF @UserStatus IS NOT NULL
         INSERT INTO @UserStatuses (Value) SELECT Value FROM dbo.fn_SplitInts(@UserStatus, ',');
     ```
   - Update WHERE clause to use `NOT EXISTS + IN` pattern for both filters
   - Keep pagination and both result sets (count then data) unchanged

---

### Phase 3 — CodeGen
**Depends on:** SQLGen output

Create/update Rust code for multi-value filtering following the item-shop pattern.

**Key tasks:**
1. Add `AdminPlayersFilterParams` DTO in `ran-models/src/admin/mod.rs` with `category: Option<Vec<i16>>` and `status: Option<Vec<i16>>`
2. Update `get_players` trait method in `ran-db-core/src/webmanager/admin/players/mod.rs`:
   - Accept `Option<Vec<i16>>` for category and status
   - Convert vectors to comma-separated strings using the item-shop pattern
   - Call SP with new parameter order: @Search, @UserType, @UserStatus, @Page, @PageSize
   - Preserve two-result-set parsing
3. Update handler in `ran-access-point/src/routes/admin/players/mod.rs`:
   - Replace `web::Query<PaginationParams>` with `HttpRequest` + `serde_qs::Config::new(5, false)`
   - Parse `AdminPlayersFilterParams`
   - Normalize empty vectors to `None`
   - Update OpenAPI annotations for multi-value parameters

---

### Phase 4 — Reviewer
**Depends on:** CodeGen output

Review implementation for correctness, security, pattern consistency, and edge case handling.

**Focus areas:**
- ✅ SQL syntax and filtering logic match item-shop pattern
- ✅ Rust code follows exact handler/trait/DTO pattern from item-shop
- ✅ `serde_qs` non-strict mode correctly parses array syntax
- ✅ Error handling is consistent with item-shop
- ✅ OpenAPI documentation is correct
- ✅ No unwrap/panic in feature code
- ✅ Edge cases: empty arrays, single values, backward compat all work
- ✅ Build passes with no new warnings

Run `cargo check`, `cargo clippy`, and smoke test with multi-value filters before approving.

---

## Files Expected to Change

| File | Change | Crate |
|------|--------|-------|
| `db-scripts/webdb/Functions/fn_SplitInts.sql` | **CREATE** | N/A |
| `db-scripts/webdb/Procedures/Administration/sp_GetPlayersList.sql` | Modified | N/A |
| `ran-models/src/admin/mod.rs` | Add DTO | ran-models |
| `ran-db-core/src/webmanager/admin/players/mod.rs` | Update trait | ran-db-core |
| `ran-access-point/src/routes/admin/players/mod.rs` | Update handler | ran-access-point |

## Risk Areas

- **Pagination semantics:** Current SP is 1-based; item-shop is 0-based. Keeping 1-based to maintain backward compat.
- **Two-result-set shape:** Players SP returns count separately; item-shop uses `COUNT(*) OVER()`. Keeping as-is unless unified later.
- **Frontend coordination:** Ensure both `?category[]=1` and `?category=1` syntax work for backward compat.
- **Type drift:** `UserBlock` column type (see Architect analysis Risk Area 2).

---

**Ready to start implementation!**

Execute agents in order: SQLGen → CodeGen → Reviewer
