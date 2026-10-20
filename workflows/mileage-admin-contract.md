# Mileage Admin Contract

> Generated from `ran-models/src/shop/mileage.rs`, `ran-models/src/shop/orders.rs`, and `ran-access-point/src/routes/admin/mileage.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// shop/mileage.rs → MileageConfig
// Returned in full by GET /admin/mileage/config and as `data` by PUT /admin/mileage/config.
export type MileageConfig = {
  config_id: number;
  mileage_rate: number;
  is_active: boolean;
  effective_from: string; // ISO 8601
  effective_to: string | null; // ISO 8601
  description: string | null;
  updated_by: string;
  created_at: string; // ISO 8601
  updated_at: string; // ISO 8601
};

// shop/mileage.rs → UpdateMileageConfigRequest
// Request body for PUT /admin/mileage/config.
export type UpdateMileageConfigRequest = {
  mileage_rate: number;
  effective_from: string | null;
  effective_to: string | null;
  description: string | null;
};

// shop/orders.rs → ShopOrder
// Returned as paginated list items by GET /admin/mileage/orders.
// See mileage-contract.md for the full type definition.
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
// See mileage-contract.md for the full type definition.
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
// Returned by GET /admin/mileage/orders/{order_id}.
export type ShopOrderDetail = {
  order: ShopOrder;
  items: ShopOrderItem[];
};
```

## API Contract

### GET /admin/mileage/config

- **Auth**: Admin JWT
- **Request body**: —
- **Response**: `MileageConfig`

---

### PUT /admin/mileage/config

- **Auth**: Admin JWT
- **Request body**: `UpdateMileageConfigRequest`
- **Response**:
  ```typescript
  // {
  //   status: string;
  //   message: string;
  //   data: MileageConfig;
  // }
  ```

---

### GET /admin/mileage/orders

- **Auth**: Admin JWT
- **Query params**: `page?: number`, `limit?: number`, `search?: string`, `status?: number`
- **Request body**: —
- **Response**:
  ```typescript
  // {
  //   data: ShopOrder[];
  //   total_items: number;
  //   page: number;
  //   limit: number;
  // }
  ```

---

### GET /admin/mileage/orders/{order_id}

- **Auth**: Admin JWT
- **Path param**: `order_id: number`
- **Request body**: —
- **Response**: `ShopOrderDetail`

---
