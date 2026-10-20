# Admin Profile Contract

> Generated from `ran-models/src/admin/auth/mod.rs` and `ran-access-point/src/routes/admin/profile/mod.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// admin/auth/mod.rs → AuthAdminProfileData
export type AuthAdminProfileData = {
  /** The admin's username. */
  user_name: string;
  /** The admin's unique identifier (UUID v4, formatted as a hyphenated string). */
  user_id: string;
  /** Numeric type/role code for the admin account. */
  user_type: number;
};
```

> No `#[serde(rename_all)]` or `#[serde(rename)]` attributes are present on `AuthAdminProfileData`.
> Field names in JSON are identical to the Rust field names: `user_name`, `user_id`, `user_type`.

---

## API Contract

### GET /admin/v1/profile

- **Auth**: Admin JWT — `Authorization: Bearer {token}` header required. The `JwtMiddleware` extracts and validates the token before the handler runs; `Claims` are injected into request extensions.
- **Request body**: None
- **Query parameters**: None

#### Success Response — 200 OK

Returns the profile of the authenticated admin, identified by the `sub` claim (UUID) in the JWT.

```json
{
  "user_name": "string",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "user_type": 1
}
```

Body type: `AuthAdminProfileData`

#### Error Responses

**400 Bad Request** — The `sub` field in the JWT claims is present but cannot be parsed as a valid UUID.

```json
{ "message": "Invalid token subject" }
```

**401 Unauthorized** — The JWT is missing, expired, or invalid (rejected by `JwtMiddleware` before reaching the handler), or `Claims` could not be extracted from request extensions.

```json
{ "message": "Unauthorized" }
```

**404 Not Found** — The JWT is valid and the UUID parsed successfully, but no admin record matching `user_id` exists in WebDB. This can occur if the admin account was deleted after the token was issued.

```json
{ "message": "Admin not found" }
```

**500 Internal Server Error** — A database error occurred while fetching the profile.

```json
{ "message": "Internal server error" }
```

---

## Notes

- `user_id` is serialized as a hyphenated UUID string (e.g., `"550e8400-e29b-41d4-a716-446655440000"`), not a raw integer or binary value.
- `user_type` maps to `i32` in Rust. The valid values and their meaning are defined by the application's role system; the contract does not constrain the integer range beyond what `i32` allows.
- The handler does **not** issue a new token or set any cookie — it is a read-only profile fetch.
- If a 404 is returned for a token that was previously valid, the frontend should treat the session as invalid, discard the stored JWT, and redirect to the admin login screen.
- JWT storage: store the token in memory (e.g., a Pinia/Zustand store) or in `sessionStorage`. Avoid `localStorage` for admin tokens.
