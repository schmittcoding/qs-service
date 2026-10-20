# Profile Admin Contract

> Generated from `ran-models/src/admin/auth/mod.rs` and `ran-access-point/src/routes/admin/profile/mod.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// admin/auth/mod.rs → AuthAdminProfileData
export type AuthAdminProfileData = {
  user_name: string;
  user_id: string;   // UUID string, e.g. "550e8400-e29b-41d4-a716-446655440000"
  user_type: number;
};
```

## API Contract

### GET /admin/v1/profile

- **Auth**: Admin JWT — `Authorization: Bearer <token>` header required. The route is wrapped by `JwtMiddleware`; the token's `sub` claim must be a valid UUID identifying the admin user.
- **Request body**: —
- **Response**: `AuthAdminProfileData`

#### Response status codes

| Status | Body | Meaning |
|--------|------|---------|
| 200 | `AuthAdminProfileData` | Profile retrieved successfully |
| 400 | `{ message: string }` | Token `sub` claim is not a valid UUID |
| 401 | `{ message: "Unauthorized" }` | JWT missing or invalid (middleware rejected the request, or `Claims` not present in request extensions) |
| 404 | `{ message: string }` | No admin record found for the given UUID |
| 500 | `{ message: string }` | Internal server error |

---
