# Orders Admin Contract

> Generated from `ran-models/src/shop/orders.rs` and `ran-access-point/src/routes/admin/orders.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// orders.rs → ShopOrder
// Full order row including identity and points balance snapshot.
export type ShopOrder = {
  order_id: number;
  user_num: number;
  user_uid: string;
  order_date: string; // ISO 8601 (local timezone offset)
  total_items: number;
  total_quantity: number;
  total_amount: number;
  total_discount: number;
  points_before: number;
  points_after: number;
  order_status: number;
  created_at: string; // ISO 8601 (local timezone offset)
};

// orders.rs → ShopOrderItem
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

// orders.rs → ShopOrderItemWithDate
// ShopOrderItem flattened with order-level date and status fields.
// The #[serde(flatten)] on the `item` field means all ShopOrderItem
// fields appear at the top level alongside order_date and order_status.
export type ShopOrderItemWithDate = ShopOrderItem & {
  order_date: string; // ISO 8601 (local timezone offset)
  order_status: number;
};

// orders.rs → ShopOrderDetail
// Composite type returned by the order detail endpoint.
export type ShopOrderDetail = {
  order: ShopOrder;
  items: ShopOrderItem[];
};

// lib.rs → PaginationParams (query string)
export type PaginationParams = {
  page?: number;
  limit?: number;
  status?: number;
  category?: number;
  search?: string;
  start_date?: string;
  end_date?: string;
};
```

## API Contract

### GET /admin/v1/orders

- **Auth**: Admin JWT
- **Query params**: `PaginationParams` — `page` (default `1`), `limit` (default `10`), `search` (optional string), `status` (optional number mapped to `u8`)
- **Request body**: —
- **Response**:
  ```
  // {
  //   data: ShopOrder[];
  //   total_items: number;
  //   page: number;
  //   limit: number;
  // }
  ```

---

### GET /admin/v1/orders/{order_id}

- **Auth**: Admin JWT
- **Path params**: `order_id: number` (i64)
- **Request body**: —
- **Response**: `ShopOrderDetail`
  > No user filter is applied — any order can be retrieved by ID.

---

### GET /admin/v1/orders/user/{user_num}

- **Auth**: Admin JWT
- **Path params**: `user_num: number` (i32)
- **Query params**: `PaginationParams` — `page` (default `1`), `limit` (default `10`)
- **Request body**: —
- **Response**:
  ```
  // {
  //   data: ShopOrder[];
  //   total_items: number;
  //   page: number;
  //   limit: number;
  // }
  ```

---

### GET /admin/v1/orders/user/{user_num}/items

- **Auth**: Admin JWT
- **Path params**: `user_num: number` (i32)
- **Query params**: `PaginationParams` — `page` (default `1`), `limit` (default `10`)
- **Request body**: —
- **Response**:
  ```
  // {
  //   data: ShopOrderItemWithDate[];
  //   total_items: number;
  //   page: number;
  //   limit: number;
  // }
  ```
  > Each entry is a flat object: all `ShopOrderItem` fields plus `order_date` and `order_status` at the top level (due to `#[serde(flatten)]` on the Rust side).

---
