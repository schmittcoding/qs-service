# Orders Contract

> Generated from `ran-models/src/shop/orders.rs` and `ran-access-point/src/routes/item_shop.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// orders.rs → ShopOrderSummary
// Slim order list entry returned for the authenticated user.
// No identity fields (user_num, user_uid) or points fields are included.
export type ShopOrderSummary = {
  order_id: number;
  order_date: string; // ISO 8601 (local timezone offset)
  total_items: number;
  total_quantity: number;
  total_amount: number;
  total_discount: number;
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

### GET /api/v1/item-shop/orders

- **Auth**: User session (cookie)
- **Query params**: `PaginationParams` — `page` (default `1`), `limit` (default `10`)
- **Request body**: —
- **Response**:
  ```
  // {
  //   data: ShopOrderSummary[];
  //   total_items: number;
  //   page: number;
  //   limit: number;
  // }
  ```

---

### GET /api/v1/item-shop/orders/{order_id}

- **Auth**: User session (cookie)
- **Path params**: `order_id: number` (i64)
- **Request body**: —
- **Response**: `ShopOrderDetail`
  > The server passes the authenticated user's `user_num` as a filter, so users can only retrieve their own orders. A `404` is returned if the order does not exist or belongs to another user.

---
