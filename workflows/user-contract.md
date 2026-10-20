# User Contract

> Generated from `ran-models/src/users/` and `ran-access-point/src/routes/user/`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// users/mod.rs → ProfileData
// Note: shape reflects ProfileData::to_json(), which omits `user_type` and
// adds a derived `user_role` field. The raw Rust struct is never serialised
// directly for this endpoint.
export type ProfileData = {
  user_name: string;
  user_num: number;
  user_email: string;
  user_available: number;
  web_points: number;
  mileage_points: number;
  created_at: string; // ISO 8601
  last_updated: string; // ISO 8601
  user_role: string; // "admin" | "player" — derived from user_type at serialisation time
};
```

## API Contract

### GET /profile

- **Auth**: User session (cookie)
- **Request body**: —
- **Response**: `ProfileData`

---
