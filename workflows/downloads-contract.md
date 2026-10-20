# Downloads Contract

> Generated from `ran-models/src/download/` and `ran-access-point/src/routes/download/`.
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
```

## API Contract

### GET /api/v{version}/downloads

- **Auth**: None
- **Request body**: —
- **Query params**: `page?: number`, `limit?: number`, `category?: number`
- **Response**: `// { data: DownloadItem[]; page: number; limit: number; total_items: number; total_pages: number }`

Note: only items with `status === 1` (active) are returned. The `category` param filters by `type` field.

---
