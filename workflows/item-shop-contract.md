# Item Shop Contract

> Generated from `ran-models/src/item_shop.rs`, `ran-models/src/shop/mod.rs`, and `ran-access-point/src/routes/item_shop.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// item_shop.rs → ItemShopBase
// Note: no serde rename_all — all field names are snake_case in JSON
export type ItemShopBase = {
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

// item_shop.rs → ItemGalleryImage
export type ItemGalleryImage = {
  image_id: string;   // UUID
  image_url: string;
  sort_order: number;
};

// item_shop.rs → ItemShopData
// Fields from ItemShopBase are inlined via #[serde(flatten)]
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

// item_shop.rs → GetItemShopResponse
// Fields from ItemShopBase are inlined via #[serde(flatten)]
// Returned by GET /api/v{n}/item-shop
export type GetItemShopResponse = {
  product_num: number;
  item_image: string;
  item_gallery: ItemGalleryImage[];
  remaining_purchase_limit: number | null;
  final_price: number;
  discount_percent: number;
  purchase_limit_text: string;
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

// shop/mod.rs → PurchaseItem
export type PurchaseItem = {
  product_num: number;
  quantity: number;
};
```

## API Contract

### GET /api/v{n}/item-shop

- **Auth**: User session (cookie) — calls `get_user_session`
- **Request body**: —
- **Response**: `GetItemShopResponse[]`

Items are sorted: discounted items first (descending by `discount_percent`), then by ascending `final_price`, then ascending `product_num`.

---

### POST /api/v{n}/item-shop/purchase

- **Auth**: User session (cookie) — calls `get_user_session`
- **Request body**: `PurchaseItem[]`
- **Response**:

```typescript
// success
{
  status: "success";
  user_name: string;
  total_cost: number;
  items_purchased: string; // JSON-serialized array of PurchaseItem
}

// error
{
  status: "error";
  message: string;
}
```

---
