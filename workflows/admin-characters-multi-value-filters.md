# Workflow: Admin Characters Multi-Value Filters

## Feature Summary

Enhance the admin `GET /characters` endpoint filters to accept multiple values for `class` and `cha_online`, matching the existing `school` parameter pattern. Users can now filter by multiple character classes and/or multiple online statuses simultaneously using bracket notation (e.g., `class[]=1&class[]=2&cha_online[]=0&cha_online[]=1`).

## Agents Involved

1. **Architect** — Analyze current filtering pattern for `school` and design equivalent approach for `class` and `cha_online`.
2. **SQLGen** — Update `sp_GetCharacters` to accept comma-separated class and online status lists, apply filtering via `fn_SplitInts`.
3. **CodeGen** — Change model types from `Option<T>` to `Option<Vec<T>>`, update DB trait signature, update route handler.
4. **OpenAPI Validator** — Verify utoipa docs reflect the new array parameters.
5. **Reviewer** — Review for consistency with `school` pattern, SQL safety, and edge cases.

## Implementation Workflow

### Phase 1 — Architect

**Prompt:**

You are analyzing a filter enhancement to the admin characters endpoint in gs-api. The feature should make `class` and `cha_online` filters accept multiple values, identical to how the existing `school` filter works.

**Current state:**
- `school` filter (already working): `Option<Vec<i32>>` in model, comma-separated string in SQL, uses `fn_SplitInts` to parse
- `class` filter (to be enhanced): `Option<i32>` in model, single value only
- `cha_online` filter (to be enhanced): `Option<i16>` in model, single value only (0 or 1)

**Current school implementation reference:**
- Model: `school: Option<Vec<i32>>` (serde_qs parses `school[]=1&school[]=2` into a Vec)
- Handler: concatenates into comma-separated string: `v.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(",")`
- Stored procedure: `fn_SplitInts(@School, ',')` returns a table, WHERE clause uses `NOT EXISTS (SELECT 1 FROM @SchoolTable) OR CI.ChaSchool IN (SELECT SchoolID FROM @SchoolTable)`

**Your task:**
- Examine the current `school` implementation in code
- Design equivalent filtering for `class` and `cha_online` 
- Identify all changes needed across: model, stored procedure, DB trait, route handler, utoipa docs
- Consider validation: `class` can be any i32 (no range check), `cha_online` should still validate to {0, 1} per element
- Note edge cases: empty Vec should mean "no filter" (return all), duplicate values, mixed with other filters
- Output: summary of files to change, SQL stored procedure signature additions, validation strategy

Do NOT write code. Analyze only.

### Phase 2 — SQLGen

**Depends on:** Architect output

**Prompt:**

You are updating the `sp_GetCharacters` stored procedure to accept multiple class and online status values using the same pattern as the existing `school` filter.

**Current school filter in procedure:**
```sql
DECLARE @SchoolTable TABLE (SchoolID INT)
IF @School IS NOT NULL
BEGIN
    INSERT INTO @SchoolTable (SchoolID)
    SELECT Value FROM dbo.fn_SplitInts(@School, ',');
END
-- WHERE clause:
(NOT EXISTS (SELECT 1 FROM @SchoolTable) OR CI.ChaSchool IN (SELECT SchoolID FROM @SchoolTable))
```

**New filters to add:**
- `@Class NVARCHAR(MAX) = NULL` — comma-separated class IDs (e.g., "1,2,5")
- `@ChaOnline NVARCHAR(MAX) = NULL` — comma-separated online statuses (e.g., "0,1")

**Changes required:**
1. Add two new parameters to the procedure signature (replace the existing single-value `@Class INT` and `@ChaOnline INT`)
2. Create two table variables: `@ClassTable TABLE (ClassID INT)` and `@OnlineTable TABLE (OnlineStatus INT)`
3. Parse both using `fn_SplitInts` when not NULL
4. Update WHERE clause conditions:
   - Class: `(NOT EXISTS (SELECT 1 FROM @ClassTable) OR CI.ChaClass IN (SELECT ClassID FROM @ClassTable))`
   - Online: `(NOT EXISTS (SELECT 1 FROM @OnlineTable) OR CI.ChaOnline IN (SELECT OnlineStatus FROM @OnlineTable))`
5. Remove the old single-value `@Status INT` and `@ChaOnline INT` parameters (or keep them for backward compat as unused)
6. Keep all other filtering logic unchanged

**Output:**
Write the updated procedure to `db-scripts/gamedb/StoredProcedure/sp_GetCharacters.sql`. Output a summary showing:
- Old parameters replaced/removed
- New parameters added
- WHERE clause conditions for both class and online filtering

### Phase 3 — CodeGen

**Depends on:** Architect output, SQLGen output

**Prompt:**

You are updating the Rust code to support multi-value `class` and `cha_online` filters.

**Files to modify:**

1. **`ran-models/src/admin/mod.rs`** — Update `CharacterPaginationParams`:
   - Change `class: Option<i32>` to `class: Option<Vec<i32>>`
   - Change `cha_online: Option<i16>` to `cha_online: Option<Vec<i16>>`
   - Update validation on `cha_online`: validate each element is 0 or 1 (use custom validator or iterate in handler)
   - Keep `sort_by`, `sort_order`, and other fields unchanged

2. **`ran-db-core/src/gamemanager/admin/characters/mod.rs`** — Update trait:
   - Change parameter `class: Option<i32>` to `class: Option<String>` (comma-separated, handled by handler)
   - Change parameter `cha_online: Option<i16>` to `cha_online: Option<String>` (comma-separated)
   - Update implementation to pass these as-is to the stored procedure (no concatenation needed, handler pre-formats them)

3. **`ran-access-point/src/routes/admin/characters/mod.rs`** — Update handler:
   - After validation, convert `params.class` Vec to comma-separated string (like `school`):
     ```rust
     let class_list = params.class.as_ref().map(|v| v.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(","));
     ```
   - Convert `params.cha_online` Vec to comma-separated string, validating each element is 0 or 1:
     ```rust
     let online_list = params.cha_online.as_ref().map(|v| {
         if v.iter().all(|o| *o == 0 || *o == 1) {
             v.iter().map(|o| o.to_string()).collect::<Vec<_>>().join(",")
         } else {
             // validation error
         }
     });
     ```
   - Update DB trait call to pass the formatted strings
   - Update utoipa docs to reflect `class[]` and `cha_online[]` array parameters

**Code style:**
- Match the existing `school` parameter handling exactly
- Use same `.map()` and `.join()` patterns
- Keep error handling consistent

**Output:**
Write all three files with complete content. Verify compilation with `cargo check`.

### Phase 4 — OpenAPI Validator

**Depends on:** CodeGen output

**Prompt:**

Validate OpenAPI documentation for the updated admin characters endpoint.

**File to validate:** `ran-access-point/src/routes/admin/characters/mod.rs`

**New params expected in utoipa docs:**
- `class[]` (changed from `class`) — `Option<Vec<i32>>` — "Filter by one or more character classes"
- `cha_online[]` (changed from `cha_online`) — `Option<Vec<i16>>` — "Filter by one or more online statuses: 0 (offline) and/or 1 (online)"

**Task:**
- Read the file to verify both params are documented correctly
- Run `cargo check -p ran-access-point` to ensure compilation
- Verify no utoipa warnings

**Output:**
Report: Are the new array parameters documented? Are there compilation errors? If missing or incorrect, list what needs fixing.

### Phase 5 — Reviewer

**Depends on:** CodeGen output, OpenAPI Validator sign-off

**Prompt:**

Review the multi-value filter implementation for consistency with the existing `school` pattern and correctness.

**Files to review:**
- `ran-models/src/admin/mod.rs` — model changes
- `ran-db-core/src/gamemanager/admin/characters/mod.rs` — trait changes
- `ran-access-point/src/routes/admin/characters/mod.rs` — handler changes
- `db-scripts/gamedb/StoredProcedure/sp_GetCharacters.sql` — SQL changes

**Review focus:**

1. **Consistency with `school` pattern:**
   - Does `class` Vec handling match `school` Vec handling exactly?
   - Does `cha_online` Vec handling match `school` Vec handling exactly?
   - Are the WHERE clause conditions identical in structure?

2. **Validation correctness:**
   - Does `cha_online` validation ensure each element is 0 or 1?
   - What happens if user sends `cha_online[]=2`? (Should return 400)
   - What happens with empty arrays? (Should behave like no filter)

3. **SQL safety:**
   - Are the new parameters properly parameterized in the EXEC statement?
   - Does the new WHERE clause handle NULL correctly?
   - Are `fn_SplitInts` calls correct?

4. **Edge cases:**
   - Empty Vec: should behave as "no filter" (return all records)
   - Duplicate values in Vec: SQL IN clause handles duplicates fine
   - Mixed with other filters: should combine correctly with AND logic

5. **Backward compatibility:**
   - Old code passing single values will break — confirm this is acceptable
   - No migration needed (API version can remain the same since this is query string only)

**Output:**
- ✅/❌ for each focus area
- Any issues with line numbers
- Run `cargo check` and `cargo clippy` for both crates
- Final approval or required fixes

## Files Expected to Change

| File | Changes |
|---|---|
| `ran-models/src/admin/mod.rs` | `class: Option<Vec<i32>>`, `cha_online: Option<Vec<i16>>` |
| `ran-db-core/src/gamemanager/admin/characters/mod.rs` | Trait signature updates, parameter type changes to `Option<String>` |
| `ran-access-point/src/routes/admin/characters/mod.rs` | Vec → comma-separated string conversion, validation per element, utoipa docs |
| `db-scripts/gamedb/StoredProcedure/sp_GetCharacters.sql` | New parameters, table variables, splitting logic, WHERE conditions |

## Risk Areas

1. **Breaking change for API consumers:** Old code passing single `class=1` will fail. Must be coordinated with frontend.
2. **Empty array validation:** User sends `class[]=` (empty array) — should mean no filter, not an error.
3. **`cha_online` range validation:** Must validate each element (0 or 1) before passing to SQL.
4. **SQL type handling:** `fn_SplitInts` returns INT; ensure class IDs are cast properly if needed.

## SQL Changes Summary

Old signature:
```sql
@Class INT = NULL,
@ChaOnline INT = NULL,
```

New signature:
```sql
@Class NVARCHAR(MAX) = NULL,
@ChaOnline NVARCHAR(MAX) = NULL,
```

Old WHERE:
```sql
(@Class IS NULL OR CI.ChaClass = @Class) AND
(@ChaOnline IS NULL OR CI.ChaOnline = @ChaOnline)
```

New WHERE:
```sql
(NOT EXISTS (SELECT 1 FROM @ClassTable) OR CI.ChaClass IN (SELECT ClassID FROM @ClassTable)) AND
(NOT EXISTS (SELECT 1 FROM @OnlineTable) OR CI.ChaOnline IN (SELECT OnlineStatus FROM @OnlineTable))
```
