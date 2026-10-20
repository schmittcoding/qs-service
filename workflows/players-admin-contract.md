# Players Admin Contract

> Generated from `ran-models/src/admin/players/` and `ran-access-point/src/routes/admin/players/mod.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// admin/players/mod.rs → UserAccountInfo
export type UserAccountInfo = {
  user_id: string;           // UUID
  user_num: number;          // i64
  user_name: string;
  user_type: number;         // i16
  user_login_state: number;  // u8 cast — 0 = logged out, 1 = logged in
  user_available: number;    // u8 cast — 0 | 1
  user_block: number;        // i32
  user_block_date: string;   // ISO 8601 (DateTime<Local>)
  user_create_date: string;  // ISO 8601 (DateTime<Local>)
  last_login_date: string;   // ISO 8601 (DateTime<Local>)
  user_email: string;
  user_web_points: number;   // f64
  user_verified: number;     // u8 cast — 0 | 1
};

// admin/players/mod.rs → UserCharacterList
export type UserCharacterList = {
  cha_num: number;          // i32
  cha_name: string;
  cha_class: number;        // i32
  cha_school: number;       // i32
  cha_sex: number;          // i16
  cha_level: number;        // i32
  cha_money: number;        // f64
  cha_online: number;       // i32
  cha_deleted: number;      // i32
  cha_deleted_date: string; // ISO 8601 (DateTime<Local>)
};

// admin/players/mod.rs → BanAccountHistory
export type BanAccountHistory = {
  block_num: number;    // i32
  block_date: string;   // ISO 8601 (DateTime<Local>)
  block_state: number;  // i32
  block_user: number;   // i32
  block_memo: string;
  admin_id: string;     // UUID
  admin_name: string;
  created_at: string;   // ISO 8601 (DateTime<Local>)
};

// admin/players/mod.rs → BanAccountRequestPayload (request body)
export type BanAccountRequestPayload = {
  block_date: string | null; // Optional ban expiry date string
  status: number;            // i16 — ban state to apply
  notes: string | null;      // Optional admin notes
};

// admin/players/mod.rs → UserLoginHistoryRecord
export type UserLoginHistoryRecord = {
  login_num: number;      // i64
  user_id: string;        // username string (not UUID)
  login_logout: number;   // i16 — 0 = logout, 1 = login
  log_date: string;       // RFC 3339 / ISO 8601
  log_ip_address: string;
  hwid: string;
};

// reports/transactions/mod.rs → WebReportTransaction
export type WebReportTransaction = {
  transaction_id: string;      // UUID
  user_num: number;            // i64
  user_name: string;
  transaction_type: number;    // i16
  transaction_time: string;    // ISO 8601 (DateTime<Local>)
  transaction_price: number;   // f64
  transaction_status: number;  // i16
  transaction_amount: number;  // f64
  updated_at: string | null;   // ISO 8601 (DateTime<Local>) — nullable
  reference_code: string | null;
  admin_remarks: string | null;
};
```

## API Contract

All endpoints require **Admin JWT** (`Authorization: Bearer <token>`) via `JwtMiddleware`.
Base path: `/admin/v1/players`

---

### GET /admin/v1/players

Fetch a paginated, filterable list of player accounts.

- **Auth**: Admin JWT
- **Query params** (`AdminPlayersFilterParams` — parsed with `serde_qs`, use bracket notation for arrays):
  | Parameter | Type | Default | Description |
  |-----------|------|---------|-------------|
  | `page` | `number` | `1` | Page number |
  | `limit` | `number` | `10` | Items per page |
  | `search` | `string` | — | Search by username |
  | `category[]` | `number[]` | — | Filter by user type (`i16`), e.g. `?category[0]=1&category[1]=2` |
  | `status[]` | `number[]` | — | Filter by block status (`i16`), same bracket syntax |
- **Request body**: —
- **Response** `200 OK`:
  ```typescript
  {
    data: UserAccountInfo[];
    page: number;
    limit: number;
    total_items: number;
  }
  ```
- **Response** `400`: plain text — `"Invalid query: <reason>"`
- **Response** `500`: `{ error: "Failed to fetch players list." }`

---

### GET /admin/v1/players/{id}

Fetch full account details for a single player.

- **Auth**: Admin JWT
- **Path param**: `id: number` — `UserNum` (i64)
- **Request body**: —
- **Response** `200 OK`: `UserAccountInfo`
- **Response** `500`: `{ error: "Failed to fetch player data." }`

---

### POST /admin/v1/players/{id}/ban-account

Ban or unban a player account. The acting admin is identified from the JWT `Claims.sub`.

- **Auth**: Admin JWT
- **Path param**: `id: number` — `UserNum` (i64)
- **Request body**: `BanAccountRequestPayload`
  ```typescript
  {
    block_date: string | null; // Ban expiry datetime string; null for permanent
    status: number;            // i16 — e.g. 1 = ban, 0 = unban
    notes: string | null;      // Optional admin memo
  }
  ```
- **Response** `200 OK`: `{ success: true }`
- **Response** `401`:
  - `{ error: "You don't have permission to perform this transaction" }` — SP returned UNAUTHORIZED
  - Plain text `"No valid JWT found"` — JWT extension missing from request
- **Response** `404`: `{ error: "User not found" }` — SP returned USER_NOT_FOUND
- **Response** `500`: `{ error: "Failed to process account ban" }`

---

### GET /admin/v1/players/{id}/ban-history

Retrieve the paginated ban/unban history for a player account.

- **Auth**: Admin JWT
- **Path param**: `id: number` — `UserNum` (i64)
- **Query params** (`PaginationParams`):
  | Parameter | Type | Default | Description |
  |-----------|------|---------|-------------|
  | `page` | `number` | `1` | Page number |
  | `limit` | `number` | `10` | Items per page |
- **Request body**: —
- **Response** `200 OK`:
  ```typescript
  {
    page: number;
    limit: number;
    total_items: number;
    data: BanAccountHistory[];
  }
  ```
- **Response** `500`: `{ error: "Failed to fetch account ban history" }`

---

### GET /admin/v1/players/{id}/transaction-history

Retrieve paginated payment transactions for a player, with a grouped summary broken down by transaction type and status.

- **Auth**: Admin JWT
- **Path param**: `id: number` — `UserNum` (i64)
- **Query params** (`AdminPaginationParams`):
  | Parameter | Type | Default | Description |
  |-----------|------|---------|-------------|
  | `page` | `number` | `1` | Page number |
  | `limit` | `number` | `10` | Items per page |
  | `start_date` | `string` | — | Optional filter start date |
  | `end_date` | `string` | — | Optional filter end date |
- **Request body**: —
- **Response** `200 OK`:
  ```typescript
  {
    page: number;
    limit: number;
    total_items: number;
    data: WebReportTransaction[];
    summary: Record<string, {   // outer key = TransactionType cast to string (e.g. "1", "2")
      pending?: number;         // total amount for TransactionStatus 0
      approved?: number;        // total amount for TransactionStatus 1
      cancelled?: number;       // total amount for TransactionStatus 2
      rejected?: number;        // total amount for TransactionStatus 3
    }>;
  }
  ```
- **Response** `500`: `{ error: "Failed to fetch account ban history" }`

---

### GET /admin/v1/players/{id}/characters

Retrieve paginated characters belonging to a player account.

- **Auth**: Admin JWT
- **Path param**: `id: number` — `UserNum` (i32)
- **Query params** (`BasicPaginationParams`):
  | Parameter | Type | Default | Description |
  |-----------|------|---------|-------------|
  | `page` | `number` | `1` | Page number |
  | `limit` | `number` | `10` | Items per page |
- **Request body**: —
- **Response** `200 OK`:
  ```typescript
  {
    page: number;
    limit: number;
    total_items: number;
    data: UserCharacterList[];
  }
  ```
- **Response** `500`: `{ error: "Internal server error" }`

---

### GET /admin/v1/players/{id}/login-history

Retrieve paginated login/logout history for a player account.

- **Auth**: Admin JWT
- **Path param**: `id: number` — `UserNum` (i64)
- **Query params** (`BasicPaginationParams`):
  | Parameter | Type | Default | Description |
  |-----------|------|---------|-------------|
  | `page` | `number` | `1` | Page number (minimum enforced: 1) |
  | `limit` | `number` | `10` | Items per page (maximum enforced: 100) |
- **Request body**: —
- **Response** `200 OK`:
  ```typescript
  {
    page: number;
    limit: number;
    total_items: number;
    data: UserLoginHistoryRecord[];
  }
  ```
- **Response** `500`: `{ error: "Failed to fetch login history" }`

---

## Notes

- `category` and `status` on `GET /admin/v1/players` are parsed with `serde_qs` (depth 5). Use bracket array notation: `?category[0]=1&category[1]=2`. Standard comma-separated or repeated-key formats will not work.
- The `id` path parameter type differs between endpoints: `i64` for most endpoints, `i32` for `/{id}/characters`. Ensure the correct numeric range is used on the frontend.
- `user_available`, `user_login_state`, and `user_verified` on `UserAccountInfo` are serialized as integers (`0` or `1`) because the Rust source casts the DB `bool` to `u8` before serialization.
- The `summary` field in the transaction-history response has `TransactionType` values as string keys (e.g., `"1"`, `"2"`), not integers. This is a consequence of inserting a `i16`-to-string conversion into a `HashMap<String, ...>`.
- `UserLoginHistoryRecord.user_id` is a username string, not a UUID — it maps to the `UserID` varchar column in the login log table.
- `BanAccountHistory` does not derive `Deserialize`; it is a server-side response type only.
