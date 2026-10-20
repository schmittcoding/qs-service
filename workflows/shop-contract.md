# Shop Contract

> Generated from `ran-models/src/shop/`, `ran-models/src/item_shop.rs`, and `ran-access-point/src/routes/item_shop.rs`, `ran-access-point/src/routes/mileage_shop.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## Notes

- There is no `ran-access-point/src/routes/shop/` directory. User-facing shop endpoints live in `item_shop.rs` (registered under `/api/v{N}/item-shop`) and `mileage_shop.rs` (registered under `/api/v{N}/mileage-shop`).
- `ShopOrderDetail` is returned from the DB layer and contains a `ShopOrder` + `Vec<ShopOrderItem>`. These are documented below as they are serialized directly.
- `DateTime<Local>` fields serialize as ISO 8601 strings.

## TypeScript Types

```typescript
// item_shop.rs → ItemShopBase
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

// item_shop.rs → GetItemShopResponse  (returned by GET /item-shop)
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
} & ItemShopBase;

// shop/mod.rs → PurchaseItem  (used in POST /item-shop/purchase and POST /mileage-shop/purchase)
export type PurchaseItem = {
  product_num: number;
  quantity: number;
};

// shop/orders.rs → ShopOrderSummary  (items in GET /item-shop/orders and GET /mileage-shop/orders)
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

// shop/orders.rs → ShopOrderItem  (items inside ShopOrderDetail)
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

// shop/orders.rs → ShopOrder  (the order header inside ShopOrderDetail)
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

// shop/orders.rs → ShopOrderDetail  (returned by GET /item-shop/orders/{order_id} and GET /mileage-shop/orders/{order_id})
export type ShopOrderDetail = {
  order: ShopOrder;
  items: ShopOrderItem[];
};

// shop/mileage.rs → MileageConfig  (partial shape returned by GET /mileage-shop/config)
// Note: the public endpoint returns only a subset of MileageConfig fields.
export type MileageConfigPublic = {
  mileage_rate: number;
  description: string | null;
  effective_from: string; // ISO 8601
  effective_to: string | null; // ISO 8601
};
```

## API Contract

### GET /api/v{N}/item-shop

- **Auth**: User session (cookie)
- **Request body**: —
- **Response**: `GetItemShopResponse[]`

---

### POST /api/v{N}/item-shop/purchase

- **Auth**: User session (cookie)
- **Request body**: `PurchaseItem[]`
- **Response**:
```typescript
// {
//   status: "success";
//   order_id: number;
//   user_name: string;
//   total_cost: number;
//   mileage_earned: number;
//   mileage_rate: number;
//   items_purchased: string; // JSON-serialized PurchaseItem[]
// }
```

---

### GET /api/v{N}/item-shop/orders

- **Auth**: User session (cookie)
- **Request body**: —
- **Query params**: `page?: number`, `limit?: number`
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

### GET /api/v{N}/item-shop/orders/{order_id}

- **Auth**: User session (cookie)
- **Request body**: —
- **Path param**: `order_id: number`
- **Response**: `ShopOrderDetail`

---

### GET /api/v{N}/mileage-shop

- **Auth**: User session (cookie)
- **Request body**: —
- **Response**: `GetItemShopResponse[]`

---

### POST /api/v{N}/mileage-shop/purchase

- **Auth**: User session (cookie)
- **Request body**: `PurchaseItem[]`
- **Response**:
```typescript
// {
//   status: "success";
//   order_id: number;
//   user_name: string;
//   total_cost: number;
//   items_purchased: string; // JSON-serialized PurchaseItem[]
// }
```

---

### GET /api/v{N}/mileage-shop/orders

- **Auth**: User session (cookie)
- **Request body**: —
- **Query params**: `page?: number`, `limit?: number`
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

### GET /api/v{N}/mileage-shop/orders/{order_id}

- **Auth**: User session (cookie)
- **Request body**: —
- **Path param**: `order_id: number`
- **Response**: `ShopOrderDetail`

---

### GET /api/v{N}/mileage-shop/config

- **Auth**: None
- **Request body**: —
- **Response**: `MileageConfigPublic` (or `{ mileage_rate: 0, description: null }` if no active config)
