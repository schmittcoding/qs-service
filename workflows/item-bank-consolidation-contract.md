# Item Bank Admin Contract

> Generated from `ran-models/src/admin/item_bank/` and `ran-access-point/src/routes/admin/item_bank/`.
> Do not edit manually — re-run ContractGen to regenerate.

## Notes

- There is no user-facing route directory for item-bank. All endpoints are admin-only and live under the `/admin/item-bank` scope, protected by `JwtMiddleware` (JWT Bearer token).
- `DateTime<Local>` fields serialize as ISO 8601 strings with timezone offset (e.g., `"2026-05-29T10:00:00+08:00"`).
- `Uuid` fields serialize as UUID strings (e.g., `"550e8400-e29b-41d4-a716-446655440000"`).
- No `#[serde(rename_all)]` or `#[serde(rename)]` attributes are present on any model in this module. All field names are serialized as-is in `snake_case`.
- `PaginationParams` query parameters: `page` (number, optional), `limit` (number, optional), `status` (number, optional), `search` (string, optional).

---

## TypeScript Types

```typescript
// entry.rs → ItemBankEntry
export type ItemBankEntry = {
  purchase_id: number;
  user_num: number;
  user_name: string;
  product_num: number;
  purchase_date: string; // ISO 8601
  last_change_date: string; // ISO 8601
  purchase_status: number;
  item_name: string;
  item_main: number;
  item_sub: number;
};

// catalog.rs → ItemCatalogOption
export type ItemCatalogOption = {
  product_num: number;
  name: string;
  item_main: number;
  item_sub: number;
};

// dispatch.rs → ItemDispatchItem
export type ItemDispatchItem = {
  id: string; // UUID
  dispatch_id: string; // UUID
  product_num: number;
  item_main: number;
  item_sub: number;
  item_name: string;
  created_at: string; // ISO 8601
};

// dispatch.rs → ItemDispatchCharacter
export type ItemDispatchCharacter = {
  id: string; // UUID
  dispatch_id: string; // UUID
  user_num: number;
  user_name: string;
  cha_num: number;
  cha_name: string;
  cha_level: number;
  created_at: string; // ISO 8601
};

// dispatch.rs → ItemDispatch
export type ItemDispatch = {
  id: string; // UUID
  notes: string;
  created_at: string; // ISO 8601
  admin_id: string; // UUID
  admin_name: string;
  items: ItemDispatchItem[];
  users: ItemDispatchCharacter[];
};

// dispatch.rs → ItemDispatchList
export type ItemDispatchList = {
  id: string; // UUID
  item_count: number;
  user_count: number;
  admin_id: string; // UUID
  admin_name: string;
  notes: string;
  created_at: string; // ISO 8601
};

// dispatch.rs → ItemDispatchDetailItem (request payload member)
export type ItemDispatchDetailItem = {
  product_num: number;
  quantity: number;
};

// dispatch.rs → ItemDispatchDetailCharacter (request payload member)
export type ItemDispatchDetailCharacter = {
  cha_num: number;
};

// dispatch.rs → ItemDispatchDetailRequestPayload (POST /dispatches request body)
export type ItemDispatchDetailRequestPayload = {
  characters: ItemDispatchDetailCharacter[];
  items: ItemDispatchDetailItem[];
  notes: string | null;
};
```

---

## API Contract

### GET /admin/item-bank/entries

- **Auth**: Admin JWT
- **Query params**: `page?: number`, `limit?: number`, `status?: number`, `search?: string`
- **Request body**: —
- **Response**:
```typescript
{
  data: ItemBankEntry[];
  page: number;
  limit: number;
}
```

---

### GET /admin/item-bank/entries/{id}

- **Auth**: Admin JWT
- **Path param**: `id: number` — the `purchase_id`
- **Request body**: —
- **Response**: `ItemBankEntry`

---

### GET /admin/item-bank/entries/user/{user_num}

- **Auth**: Admin JWT
- **Path param**: `user_num: number`
- **Query params**: `page?: number`, `limit?: number`, `status?: number`
- **Request body**: —
- **Response**:
```typescript
{
  data: ItemBankEntry[];
  total_items: number;
  page: number;
  limit: number;
}
```

---

### GET /admin/item-bank/catalog

- **Auth**: Admin JWT
- **Request body**: —
- **Response**: `ItemCatalogOption[]`

---

### GET /admin/item-bank/dispatches

- **Auth**: Admin JWT
- **Query params**: `page?: number`, `limit?: number`, `search?: string`
- **Request body**: —
- **Response**:
```typescript
{
  data: ItemDispatchList[];
  page: number;
  limit: number;
  total_items: number;
}
```

---

### GET /admin/item-bank/dispatches/{id}

- **Auth**: Admin JWT
- **Path param**: `id: string` — UUID of the dispatch
- **Request body**: —
- **Response**: `ItemDispatch`

---

### POST /admin/item-bank/dispatches

- **Auth**: Admin JWT (caller identity extracted from JWT `sub` claim server-side — no need to send admin ID in body)
- **Request body**: `ItemDispatchDetailRequestPayload`
- **Response (success)**:
```typescript
// HTTP 200
{ success: true; dispatch_id: string } // dispatch_id is a UUID string
```
- **Response (forbidden)**:
```typescript
// HTTP 403
{ error: string }
```
- **Response (error)**:
```typescript
// HTTP 500
{ error: string }
```

---
