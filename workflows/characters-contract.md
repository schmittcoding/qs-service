# Characters Contract

> Generated from `ran-models/src/admin/mod.rs`, `ran-models/src/admin/players/mod.rs`,
> and `ran-access-point/src/routes/characters.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// ran-models/src/admin/mod.rs → BasicPaginationParams
// Sent as query parameters — both fields are optional.
export type BasicPaginationParams = {
  page: number | null;
  limit: number | null;
};

// ran-models/src/admin/players/mod.rs → UserCharacterList
export type UserCharacterList = {
  cha_num: number;
  cha_name: string;
  cha_class: number;
  cha_school: number;
  cha_sex: number;
  cha_level: number;
  cha_money: number;
  cha_online: number;
  cha_deleted: number;
  cha_deleted_date: string; // ISO 8601 (DateTime<Local>)
};
```

## API Contract

### GET /characters

- **Auth**: User session (cookie) — validated via `get_user_session`. Returns `401` if no session is present.
- **Request body**: — (none)
- **Query params**: `BasicPaginationParams`

  | Parameter | Type     | Default | Description                             |
  |-----------|----------|---------|-----------------------------------------|
  | `page`    | `number` | `1`     | Page number (minimum 1)                 |
  | `limit`   | `number` | `10`    | Number of items per page (minimum 1)    |

- **Response**:

  ```typescript
  // 200 OK
  {
    page: number;
    limit: number;
    total_pages: number;
    total_items: number;
    data: UserCharacterList[];
  }
  ```

- **Error responses**:

  ```typescript
  // 401 Unauthorized — no active session
  { error: "No session found"; message: "User not logged in" }

  // 500 Internal Server Error — database failure
  { error: "Failed to fetch characters" }
  ```

#### Notes

- Results are cached server-side per authenticated user for 60 seconds. On a cache miss the full character list for that user is fetched from the database (`page` and `limit` slice the in-memory list).
- Only characters belonging to the authenticated user (derived from the session `user_num`) are ever returned.

---
