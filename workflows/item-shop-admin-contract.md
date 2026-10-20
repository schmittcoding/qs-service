# Item Shop Admin Contract

> Generated from `ran-models/src/item_shop.rs`, `ran-models/src/admin/mod.rs`, and `ran-access-point/src/routes/admin/item_shop.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## Notes

- All admin endpoints are mounted under `/admin/v{n}/item-shop` and protected by `JwtMiddleware` (Bearer JWT).
- The PUT and POST endpoints accept `multipart/form-data` (not JSON), with a `file` field for the primary image, an optional `gallery` field for gallery images, and a `data` field containing a JSON blob of the item data.
- `Uuid` fields serialize as lowercase hyphenated UUID strings (e.g., `"550e8400-e29b-41d4-a716-446655440000"`).

## TypeScript Types

```typescript
// item_shop.rs → ItemShopBase
// All field names are snake_case in JSON (no serde rename_all)
export type ItemShopBase = {
  item_main: number;
  item_sub: number;
  item_name: string;
  item_price: number;
  item_category: number;
  item_description: string;
  item_discount: number | null;
  item_discount_type: number | null;   // 1 = Percentage, 2 = Fixed
  item_status: number;
  item_stock: number;
  item_purchase_limit: number | null;
  item_purchase_limit_type: number | null;
  item_tag: string;
};

// item_shop.rs → ItemGalleryImage
export type ItemGalleryImage = {
  image_id: string;   // UUID
  image_url: string;
  sort_order: number;
};

// item_shop.rs → ItemShopData
// Fields from ItemShopBase are inlined via #[serde(flatten)]
// Returned by GET /admin/v{n}/item-shop/{id}, POST, and PUT
export type ItemShopData = {
  product_num: number;
  item_image: string;
  item_gallery: ItemGalleryImage[];
  // flattened from ItemShopBase:
  item_main: number;
  item_sub: number;
  item_name: string;
  item_price: number;
  item_category: number;
  item_description: string;
  item_discount: number | null;
  item_discount_type: number | null;
  item_status: number;
  item_stock: number;
  item_purchase_limit: number | null;
  item_purchase_limit_type: number | null;
  item_tag: string;
};

// item_shop.rs → GetItemShopData
// Fields from ItemShopData (and therefore ItemShopBase) are inlined via #[serde(flatten)]
// Returned inside the paginated list from GET /admin/v{n}/item-shop
export type GetItemShopData = {
  discounted_price: number;
  purchase_limit_text: string;
  // flattened from ItemShopData → ItemShopBase:
  product_num: number;
  item_image: string;
  item_gallery: ItemGalleryImage[];
  item_main: number;
  item_sub: number;
  item_name: string;
  item_price: number;
  item_category: number;
  item_description: string;
  item_discount: number | null;
  item_discount_type: number | null;
  item_status: number;
  item_stock: number;
  item_purchase_limit: number | null;
  item_purchase_limit_type: number | null;
  item_tag: string;
};

// item_shop.rs → UpdateItemShopRequest
// Used as the `data` JSON part in the PUT multipart form body
// Fields from ItemShopBase are inlined via #[serde(flatten)]
export type UpdateItemShopRequest = {
  item_image: string | null;           // existing image URL to retain; omit/null to replace via file upload
  removed_gallery_ids: string[] | null; // array of UUID strings to delete
  // flattened from ItemShopBase:
  item_main: number;
  item_sub: number;
  item_name: string;
  item_price: number;
  item_category: number;
  item_description: string;
  item_discount: number | null;
  item_discount_type: number | null;
  item_status: number;
  item_stock: number;
  item_purchase_limit: number | null;
  item_purchase_limit_type: number | null;
  item_tag: string;
};

// admin/mod.rs → AdminItemShopFilterParams
// Used as query parameters for GET /admin/v{n}/item-shop
export type AdminItemShopFilterParams = {
  page?: number;
  limit?: number;
  search?: string;
  category?: number[];    // filter by category IDs (multi-value, e.g. category[]=1&category[]=2)
  status?: number[];      // filter by status values
  purchase_limit?: number[]; // filter by purchase limit thresholds
};

// Paginated list response shape for GET /admin/v{n}/item-shop
export type AdminGetItemShopResponse = {
  data: GetItemShopData[];
  page: number;
  limit: number;
  total_items: number;
};

// Envelope returned by PUT /{id} and POST
export type ItemShopMutationResponse = {
  data: ItemShopData;
  message: string;
};
```

## API Contract

### GET /admin/v{n}/item-shop

- **Auth**: Admin JWT (Bearer token via `JwtMiddleware`)
- **Request body**: —
- **Query params**: `AdminItemShopFilterParams` (all optional; arrays passed via `serde_qs` bracket notation, e.g. `category[0]=1&category[1]=2`)
- **Response**: `AdminGetItemShopResponse`

```typescript
// { data: GetItemShopData[]; page: number; limit: number; total_items: number; }
```

Defaults: `page = 0`, `limit = 10`.

---

### GET /admin/v{n}/item-shop/{id}

- **Auth**: Admin JWT (Bearer token via `JwtMiddleware`)
- **Path param**: `id: number` — the `product_num` of the item
- **Request body**: —
- **Response**: `ItemShopData`

The `item_gallery` array is populated from a separate DB call (`sp_GetItemImages`).

---

### PUT /admin/v{n}/item-shop/{id}

- **Auth**: Admin JWT (Bearer token via `JwtMiddleware`)
- **Path param**: `id: number` — the `product_num` of the item to update
- **Request body**: `multipart/form-data`
  - `file` (optional, max 5 MB): new primary image file; if omitted, the existing `item_image` URL in `data` is retained
  - `gallery` (optional, repeated field, max 5 MB each, up to 5 images): new gallery images to append
  - `data` (required, JSON part): `UpdateItemShopRequest`
- **Response**: `ItemShopMutationResponse`

```typescript
// { data: ItemShopData; message: string; }
```

Gallery images listed in `removed_gallery_ids` are deleted before new gallery images are uploaded.

---

### POST /admin/v{n}/item-shop

- **Auth**: Admin JWT (Bearer token via `JwtMiddleware`)
- **Request body**: `multipart/form-data`
  - `file` (required, max 5 MB): primary image file
  - `gallery` (optional, repeated field, max 5 MB each, up to 5 images): initial gallery images
  - `data` (required, JSON part): `ItemShopBase`
- **Response**: `ItemShopMutationResponse`

```typescript
// { data: ItemShopData; message: string; }
```

---
