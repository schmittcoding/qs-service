# Players Admin — Login History Contract

> Generated from `ran-models/src/admin/players/mod.rs` and `ran-access-point/src/routes/admin/players/mod.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// admin/players/mod.rs → UserLoginHistoryRecord
//
// Field notes:
//   login_logout — integer flag: 0 = Logout, 1 = Login (maps from DB column "LogInOut")
//   log_date     — RFC 3339 / ISO 8601 string, e.g. "2026-05-26T08:00:00+08:00"
//                  Falls back to "1970-01-01T00:00:00+00:00" when the DB value is NULL
//   hwid         — hardware identifier string; empty string when absent
//
// No #[serde(rename_all)] or #[serde(rename)] attributes are present on this struct.
// Rust snake_case field names are serialised as-is (snake_case JSON keys).
export type UserLoginHistoryRecord = {
  login_num: number;       // i64 — primary key of the login log row
  user_id: string;         // DB column "UserID" — stored as a string (not UUID)
  login_logout: number;    // i16 — 0 = Logout, 1 = Login
  log_date: string;        // ISO 8601 datetime string
  log_ip_address: string;  // IPv4 / IPv6 address string; empty string when absent
  hwid: string;            // hardware identifier; empty string when absent
};

// Response wrapper for GET /admin/v1/players/{id}/login-history
// page and limit are i16 on the server; they arrive as JSON numbers.
// total_items reflects the i32 TotalRecords value from the stored procedure.
export type UserLoginHistoryResponse = {
  page: number;
  limit: number;
  total_items: number;
  data: UserLoginHistoryRecord[];
};
```

## API Contract

### GET /admin/v1/players/{id}/login-history

- **Auth**: Admin JWT Bearer token — the route is registered under `admin_route()` and protected by `JwtMiddleware`. Include the token in the `Authorization: Bearer <token>` header.
- **Path parameter**:
  - `id` (`integer / i64`) — the numeric user ID (`UserNum`) of the player whose login history is requested.
- **Query parameters**:

  | Parameter | Type    | Required | Default | Constraints          | Description        |
  |-----------|---------|----------|---------|----------------------|--------------------|
  | `page`    | integer | No       | `1`     | minimum clamped to 1 | Page number (1-based) |
  | `limit`   | integer | No       | `10`    | maximum clamped to 100 | Records per page |

- **Request body**: none
- **Response — 200 OK**: `UserLoginHistoryResponse`
- **Error responses**:

  | Status | Body                                            | Reason                                          |
  |--------|-------------------------------------------------|-------------------------------------------------|
  | 401    | *(actix-web middleware rejection)*              | Missing or invalid JWT Bearer token             |
  | 500    | `{ "error": "Failed to fetch login history" }` | Database query failure or connection pool error |

---

### Example request

```bash
curl -s \
  -H "Authorization: Bearer <admin_jwt_token>" \
  "https://api.example.com/admin/v1/players/1001/login-history?page=1&limit=10"
```

### Example response (200 OK)

```json
{
  "page": 1,
  "limit": 10,
  "total_items": 42,
  "data": [
    {
      "login_num": 9001,
      "user_id": "a3f2c1d0-beef-dead-cafe-000000000001",
      "login_logout": 1,
      "log_date": "2026-05-26T08:15:30+08:00",
      "log_ip_address": "203.0.113.55",
      "hwid": "ABCD1234EFGH5678"
    },
    {
      "login_num": 9000,
      "user_id": "a3f2c1d0-beef-dead-cafe-000000000001",
      "login_logout": 0,
      "log_date": "2026-05-25T23:47:10+08:00",
      "log_ip_address": "203.0.113.55",
      "hwid": "ABCD1234EFGH5678"
    }
  ]
}
```

## Notes

- The handler clamps `page` to a minimum of `1` (`.max(1)`) and `limit` to a maximum of `100` (`.min(100)`). Values outside these bounds are silently adjusted rather than rejected.
- `log_date` is formatted with `chrono`'s `to_rfc3339()`, which produces a timezone-offset string (e.g. `+08:00`) rather than a `Z`-suffix UTC string. Consumers should parse it as a generic ISO 8601 datetime.
- `login_logout` is a raw integer, not a boolean or string enum. Frontend code should map `0 → "Logout"` and `1 → "Login"` at the display layer.
- `user_id` is the string representation of a UUID as stored in the `UserID` column — it is not re-parsed into a typed UUID on the server, so the format depends on whatever the database returns.
- Empty strings (`""`) are used as the null-safe default for `log_ip_address` and `hwid` when the database column is NULL.
- The stored procedure called is `sp_GetUserLoginHistory @UserNum, @Page, @PageSize` against `UserDB`.
