# Mileage Contract

> Generated from `ran-models/src/shop/mileage.rs`, `ran-models/src/shop/orders.rs`, `ran-models/src/item_shop.rs`, `ran-models/src/shop/mod.rs`, and `ran-access-point/src/routes/mileage_shop.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// item_shop.rs → ItemShopBase
// Note: fields are snake_case in JSON (no serde rename_all attribute).
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
  shop_type: number;
};

// item_shop.rs → ItemGalleryImage
export type ItemGalleryImage = {
  image_id: string; // UUID
  image_url: string;
  sort_order: number;
};

// item_shop.rs → GetItemShopResponse
// Returned as an array by GET /mileage-shop.
// Uses #[serde(flatten)] for ItemShopBase fields — all ItemShopBase fields appear at the top level.
export type GetItemShopResponse = {
  product_num: number;
  item_image: string;
  item_gallery: ItemGalleryImage[];
  remaining_purchase_limit: number | null;
  final_price: number;
  discount_percent: number;
  purchase_limit_text: string;
  mileage_points_to_earn: number;
  current_mileage_rate: number;
  // --- flattened from ItemShopBase ---
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
  shop_type: number;
};

// shop/mod.rs → PurchaseItem
export type PurchaseItem = {
  product_num: number;
  quantity: number;
};

// shop/orders.rs → ShopOrderSummary
// Returned as paginated list by GET /mileage-shop/orders.
export type ShopOrderSummary = {
  order_id: number;
  order_date: string; // ISO 8601
  total_items: number;
  total_quantity: number;
  total_amount: number;
  total_discount: number;
  order_status: number;
  created_at: string; // ISO 8601
};

// shop/orders.rs → ShopOrder
// Embedded inside ShopOrderDetail.
export type ShopOrder = {
  order_id: number;
  user_num: number;
  user_uid: string;
  order_date: string; // ISO 8601
  total_items: number;
  total_quantity: number;
  total_amount: number;
  total_discount: number;
  points_before: number;
  points_after: number;
  order_status: number;
  created_at: string; // ISO 8601
};

// shop/orders.rs → ShopOrderItem
export type ShopOrderItem = {
  order_item_id: number;
  order_id: number;
  user_num: number;
  product_num: number;
  item_name: string;
  item_main: number;
  item_sub: number;
  quantity: number;
  unit_price: number;
  discount_type: number | null;
  discount_value: number | null;
  final_unit_price: number;
  line_total: number;
};

// shop/orders.rs → ShopOrderDetail
// Returned by GET /mileage-shop/orders/{order_id}.
export type ShopOrderDetail = {
  order: ShopOrder;
  items: ShopOrderItem[];
};
```

## API Contract

### GET /mileage-shop

- **Auth**: User session (cookie)
- **Request body**: —
- **Response**: `GetItemShopResponse[]`

---

### POST /mileage-shop/purchase

- **Auth**: User session (cookie)
- **Request body**: `PurchaseItem[]`
- **Response**:
  ```typescript
  // {
  //   status: string;
  //   order_id: number;
  //   user_name: string;
  //   total_cost: number;
  //   items_purchased: string; // JSON-serialised PurchaseItem[] string
  // }
  ```

---

### GET /mileage-shop/orders

- **Auth**: User session (cookie)
- **Query params**: `page?: number`, `limit?: number`
- **Request body**: —
- **Response**:
  ```typescript
  // {
  //   data: ShopOrderSummary[];
  //   total_items: number;
  //   page: number;
  //   limit: number;
  // }
  ```

---

### GET /mileage-shop/orders/{order_id}

- **Auth**: User session (cookie)
- **Path param**: `order_id: number`
- **Request body**: —
- **Response**: `ShopOrderDetail`

---

### GET /mileage-shop/config

- **Auth**: None
- **Request body**: —
- **Response**:
  ```typescript
  // When config exists:
  // {
  //   mileage_rate: number;
  //   description: string | null;
  //   effective_from: string; // ISO 8601
  //   effective_to: string | null; // ISO 8601
  // }
  //
  // When no config is set:
  // {
  //   mileage_rate: number; // 0
  //   description: null;
  // }
  ```

---
