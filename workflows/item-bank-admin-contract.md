# Item Bank Admin Contract

> Generated from `ran-models/src/admin/item_bank/` and `ran-access-point/src/routes/admin/item_bank/`.
> Do not edit manually — re-run ContractGen to regenerate.

All endpoints are mounted under `/admin/v{n}/item-bank/` and are protected by `JwtMiddleware` (Admin JWT Bearer token).

## TypeScript Types

```typescript
// entry.rs → ItemBankEntry
export type ItemBankEntry = {
  purchase_id: number;
  user_num: number;
  user_name: string;
  product_num: number;
  purchase_date: string; // ISO 8601 (DateTime<Local>)
  last_change_date: string; // ISO 8601 (DateTime<Local>)
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
  created_at: string; // ISO 8601 (DateTime<Local>)
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
  created_at: string; // ISO 8601 (DateTime<Local>)
};

// dispatch.rs → ItemDispatch
export type ItemDispatch = {
  id: string; // UUID
  notes: string;
  created_at: string; // ISO 8601 (DateTime<Local>)
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
  created_at: string; // ISO 8601 (DateTime<Local>)
};

// dispatch.rs → ItemDispatchDetailItem (request sub-type)
export type ItemDispatchDetailItem = {
  product_num: number;
  quantity: number;
};

// dispatch.rs → ItemDispatchDetailCharacter (request sub-type)
export type ItemDispatchDetailCharacter = {
  cha_num: number;
};

// dispatch.rs → ItemDispatchDetailRequestPayload
export type ItemDispatchDetailRequestPayload = {
  characters: ItemDispatchDetailCharacter[];
  items: ItemDispatchDetailItem[];
  notes: string | null;
};

// dispatch.rs → DispatchCharacterFilterRequest (query params)
export type DispatchCharacterFilterRequest = {
  search: string | null;
  min_level: number | null;
  max_level: number | null;
  login_from: string | null; // YYYY-MM-DD
  login_to: string | null;   // YYYY-MM-DD
  page: number | null;
  limit: number | null;
};

// dispatch.rs → DispatchCharacterFilterResult
export type DispatchCharacterFilterResult = {
  user_num: number;
  user_id: string;
  cha_name: string;
  cha_num: number;
  cha_level: number;
  cha_class: number;
  cha_school: number;
};
```

## API Contract

### GET /item-bank/entries

- **Auth**: Admin JWT
- **Request body**: —
- **Query params**: `page?: number`, `limit?: number`, `status?: number`, `search?: string`
- **Response**:

```typescript
// { data: ItemBankEntry[], page: number, limit: number }
```

---

### GET /item-bank/entries/{id}

- **Auth**: Admin JWT
- **Request body**: —
- **Path params**: `id: number` (purchase ID, i32)
- **Response**: `ItemBankEntry`

---

### GET /item-bank/entries/user/{user_num}

- **Auth**: Admin JWT
- **Request body**: —
- **Path params**: `user_num: number` (i32)
- **Query params**: `page?: number`, `limit?: number`, `status?: number`
- **Response**:

```typescript
// { data: ItemBankEntry[], total_items: number, page: number, limit: number }
```

---

### GET /item-bank/catalog

- **Auth**: Admin JWT
- **Request body**: —
- **Response**: `ItemCatalogOption[]`

---

### GET /item-bank/dispatches

- **Auth**: Admin JWT
- **Request body**: —
- **Query params**: `page?: number`, `limit?: number`, `search?: string`
- **Response**:

```typescript
// { data: ItemDispatchList[], page: number, limit: number, total_items: number }
```

---

### GET /item-bank/dispatches/{id}

- **Auth**: Admin JWT
- **Request body**: —
- **Path params**: `id: string` (UUID)
- **Response**: `ItemDispatch`

---

### POST /item-bank/dispatches

- **Auth**: Admin JWT (reads `Claims.sub` from JWT extensions to identify the dispatching admin)
- **Request body**: `ItemDispatchDetailRequestPayload`
- **Response**:

```typescript
// 200 OK:      { success: true, dispatch_id: string }
// 403 Forbidden: { error: string }
// 500 Error:   { error: string }
```

---

### GET /item-bank/dispatches/characters/filter

- **Auth**: Admin JWT
- **Request body**: —
- **Query params**: all fields from `DispatchCharacterFilterRequest`
  - `search?: string` — searched as character name (string) or user number (parsed as i32 if numeric)
  - `min_level?: number`
  - `max_level?: number`
  - `login_from?: string` — format `YYYY-MM-DD`; returns 400 if invalid
  - `login_to?: string` — format `YYYY-MM-DD`; returns 400 if invalid
  - `page?: number` — defaults to `0`
  - `limit?: number` — defaults to `50`
- **Response**:

```typescript
// { data: DispatchCharacterFilterResult[], page: number, limit: number, total_items: number }
```

---
