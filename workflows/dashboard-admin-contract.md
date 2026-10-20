# Dashboard Admin Contract

> Generated from `ran-access-point/src/routes/admin/dashboard/mod.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

This feature returns only scalar numbers. There are no complex TypeScript types to define.

```typescript
// No Rust structs exist for this feature.
// All responses are bare number (i32 or f64).
// Query parameters accepted as plain string — valid values are listed per endpoint.
```

## API Contract

### GET /admin/v1/dashboard/online

- **Auth**: Admin JWT
- **Request body**: —
- **Query parameters**: —
- **Response**: `number` — current online player count (i32)
- **Errors**:
  - `500` — `{ status: "error", message: "Failed to get online player count" }`

---

### GET /admin/v1/dashboard/registered

- **Auth**: Admin JWT
- **Request body**: —
- **Query parameters**: —
- **Response**: `number` — total registered user count (i32)
- **Errors**:
  - `500` — `{ status: "error", message: "Failed to get registered user count" }`

---

### GET /admin/v1/dashboard/transactions

- **Auth**: Admin JWT
- **Request body**: —
- **Query parameters**:
  - `type` (required): `"pending" | "approved" | "declined"` — filters transactions by status
- **Response**: `number` — transaction count matching the given status (i32)
- **Errors**:
  - `400` — `{ status: "error", message: "Invalid type parameter" }` — when `type` is not one of the allowed values
  - `500` — `{ status: "error", message: "Failed to get transaction count" }`

---

### GET /admin/v1/dashboard/transactions/total

- **Auth**: Admin JWT
- **Request body**: —
- **Query parameters**:
  - `type` (required): `"today" | "month" | "year"` — period over which to sum recharge amounts
- **Response**: `number` — total recharge amount for the given period (f64)
- **Errors**:
  - `400` — `{ status: "error", message: "Invalid type parameter" }` — when `type` is not one of the allowed values
  - `500` — `{ status: "error", message: "Failed to get total recharge amount" }`

---

## Notes

- This feature is admin-only. There are no user-facing (non-admin) endpoints, so no `dashboard-contract.md` is produced.
- No Rust structs are defined in `ran-models/src/dashboard/` — the feature uses only scalar return types (`i32`, `f64`). As a result, there are no TypeScript types to export.
- All four endpoints are protected by `JwtMiddleware` via `admin_route()` and require a valid Bearer token in the `Authorization` header.
- The `type` query parameter on both `/transactions` and `/transactions/total` is validated in the handler. An unrecognised value returns `400 Bad Request` before any database call is made.
- The internal mapping for `/transactions` is: `"pending"` → status `0`, `"approved"` → status `1`, `"declined"` → status `2`. The frontend only needs to supply the string label.
