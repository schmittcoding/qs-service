# Downloads Admin Contract

> Generated from `ran-models/src/download/` and `ran-access-point/src/routes/admin/download/`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// download/mod.rs → DownloadItem
export type DownloadItem = {
  id: string;        // UUID
  provider: string;
  link: string;
  status: number;    // u8 — 0 = inactive, 1 = active
  type: number;      // i16 — download category/type discriminator
  version: string;
};

// download/mod.rs → DownloadItemPayload
export type DownloadItemPayload = {
  provider: string;
  link: string;
  status: number;    // u8
  type: number;      // i16
  version: string;
};
```

## API Contract

### GET /admin/v{version}/downloads

- **Auth**: Admin JWT (`JwtMiddleware`)
- **Request body**: —
- **Query params**: `page?: number`, `limit?: number`, `status?: number`, `category?: number`
- **Response**: `// { data: DownloadItem[]; total_items: number }`

Note: no status filter is applied server-side — all items (active and inactive) are returned. Results are paginated client-side using `page` and `limit`.

---

### POST /admin/v{version}/downloads

- **Auth**: Admin JWT (`JwtMiddleware`)
- **Request body**: `DownloadItemPayload`
- **Response (200)**: `// { message: string }`
- **Response (403)**: `// { error: string }` — forbidden (caller lacks permission)
- **Response (409)**: `// { error: string }` — duplicate data
- **Response (500)**: `// { error: string }` — internal server error

---

### PUT /admin/v{version}/downloads/{id}

- **Auth**: Admin JWT (`JwtMiddleware`)
- **Path param**: `id` — UUID of the download item to update
- **Request body**: `DownloadItemPayload`
- **Response (200)**: `// { message: string }`
- **Response (403)**: `// { error: string }` — forbidden
- **Response (404)**: `// { error: string }` — item not found
- **Response (500)**: `// { error: string }` — internal server error

---
