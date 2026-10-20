# Shop Admin Contract

> Generated from `ran-models/src/shop/`, `ran-models/src/item_shop.rs`, `ran-models/src/admin/` and `ran-access-point/src/routes/admin/shop/mod.rs`, `ran-access-point/src/routes/admin/item_shop.rs`, `ran-access-point/src/routes/admin/orders.rs`, `ran-access-point/src/routes/admin/mileage.rs`, `ran-access-point/src/routes/admin/purchases.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## Notes

- All admin endpoints are protected by `JwtMiddleware` (Bearer JWT). Every handler lives under the `admin_route()` scope (`/v{N}/...`).
- The admin shop has two route groups: `/v{N}/shop` (legacy — `AdminShopDataResponse`) and `/v{N}/item-shop` (newer — multipart upload, `ItemShopData`/`GetItemShopData`).
- `ShopPurchaseBase` is a re-export alias of `ItemBankEntry`.
- `DateTime<Local>` fields serialize as ISO 8601 strings.

## TypeScript Types

```typescript
// shop/mod.rs → AdminShopDataResponse  (returned by legacy /v{N}/shop endpoints)
export type AdminShopDataResponse = {
  product_num: number;
  item_main: number;
  item_sub: number;
  item_name: string;
  item_price: number;
  item_category: number;
  item_description: string;
  item_discount: number | null;
  item_discount_type: number | null;
  item_image: string | null; // base64-encoded
  item_status: number;
  item_stock: number;
  item_purchase_limit: number | null;
};

// shop/mod.rs → AddShopData  (request body for POST /v{N}/shop)
export type AddShopData = {
  item_main: number;
  item_sub: number;
  item_name: string;
  item_price: number;
  item_category: number;
  item_description: string;
  item_discount: number | null;
  item_discount_type: number | null;
  item_image: string | null; // base64-encoded, optional
  item_status: number;
  item_stock: number;
  item_purchase_limit: number | null;
  item_purchase_limit_type: number | null;
};

// shop/mod.rs → EditShopData  (request body for PUT /v{N}/shop/{id})
export type EditShopData = {
  item_main: number;
  item_sub: number;
  item_name: string;
  item_price: number;
  item_category: number;
  item_description: string;
  item_discount: number | null;
  item_discount_type: number | null;
  item_image: string | null; // base64-encoded, optional
  item_status: number;
  item_stock: number;
  item_purchase_limit: number | null;
  item_purchase_limit_type: number | null;
};

// item_shop.rs → ItemShopBase  (shared base embedded via #[serde(flatten)])
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

// item_shop.rs → ItemShopData  (returned by GET /v{N}/item-shop/{id} and edit/add responses)
export type ItemShopData = {
  product_num: number;
  item_image: string;
  item_gallery: ItemGalleryImage[];
} & ItemShopBase;

// item_shop.rs → GetItemShopData  (items in GET /v{N}/item-shop list)
export type GetItemShopData = {
  discounted_price: number;
  purchase_limit_text: string;
} & ItemShopData;

// shop/orders.rs → ShopOrder  (order header in orders list and detail)
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

// shop/orders.rs → ShopOrderItemWithDate  (used in GET /v{N}/orders/user/{user_num}/items)
export type ShopOrderItemWithDate = ShopOrderItem & {
  order_date: string; // ISO 8601
  order_status: number;
};

// shop/orders.rs → ShopOrderDetail
export type ShopOrderDetail = {
  order: ShopOrder;
  items: ShopOrderItem[];
};

// admin/item_bank/entry.rs → ItemBankEntry  (also exported as ShopPurchaseBase)
export type ItemBankEntry = {
  purchase_id: number;
  user_num: number;
  user_name: string;
  product_num: number;
  purchase_date: string; // ISO 8601
  last_change_date: string; // ISO 8601
  purchase_status: number;
  item_name: string;
  item_main: number;
  item_sub: number;
};

// shop/mileage.rs → MileageConfig  (returned by GET /v{N}/mileage/config)
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

// shop/mileage.rs → UpdateMileageConfigRequest  (request body for PUT /v{N}/mileage/config)
export type UpdateMileageConfigRequest = {
  mileage_rate: number;
  effective_from: string | null;
  effective_to: string | null;
  description: string | null;
};

// admin/mod.rs → AdminItemShopFilterParams  (query params for GET /v{N}/item-shop)
export type AdminItemShopFilterParams = {
  page?: number;
  limit?: number;
  shop_type?: number;
  search?: string;
  category?: number[];
  status?: number[];
  purchase_limit?: number[];
};
```

## API Contract

---

### Legacy Shop Endpoints (`/v{N}/shop`)

---

### GET /v{N}/shop

- **Auth**: Admin JWT
- **Request body**: —
- **Query params**: `page?: number`, `limit?: number`, `status?: number`, `category?: string` (comma-separated `i16` list)
- **Response**:
```typescript
// {
//   page: number;
//   limit: number;
//   total_pages: number;
//   total_items: number;
//   data: AdminShopDataResponse[];
// }
```

---

### GET /v{N}/shop/{id}

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `id: number`
- **Response**: `AdminShopDataResponse`

---

### POST /v{N}/shop

- **Auth**: Admin JWT
- **Request body**: `AddShopData`
- **Response**: `{ message: "Item added successfully" }`

---

### PUT /v{N}/shop/{id}

- **Auth**: Admin JWT
- **Request body**: `EditShopData`
- **Path param**: `id: number`
- **Response**: `AdminShopDataResponse`

---

### DELETE /v{N}/shop/delete-item/{id}

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `id: number`
- **Response**: `AdminShopDataResponse`

---

### POST /v{N}/shop/migrate-images

- **Auth**: Admin JWT
- **Request body**: —
- **Response**: `true`

---

### Newer Item-Shop Endpoints (`/v{N}/item-shop`)

---

### GET /v{N}/item-shop

- **Auth**: Admin JWT
- **Request body**: —
- **Query params**: `AdminItemShopFilterParams`
- **Response**:
```typescript
// {
//   data: GetItemShopData[];
//   page: number;
//   limit: number;
//   total_items: number;
// }
```

---

### GET /v{N}/item-shop/{id}

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `id: number`
- **Response**: `ItemShopData` (with `item_gallery` populated)

---

### POST /v{N}/item-shop

- **Auth**: Admin JWT
- **Request body**: `multipart/form-data` — `file` (image, required), `gallery[]` (up to 5 images, optional), `data` (JSON `ItemShopBase`)
- **Response**: `{ data: ItemShopData, message: string }`

---

### PUT /v{N}/item-shop/{id}

- **Auth**: Admin JWT
- **Request body**: `multipart/form-data` — `file` (image, optional), `gallery[]` (new images, optional), `data` (JSON — `UpdateItemShopRequest`: `ItemShopBase` fields + `item_image?: string`, `removed_gallery_ids?: string[]`)
- **Path param**: `id: number`
- **Response**: `{ data: ItemShopData, message: string }`

---

### Order Endpoints (`/v{N}/orders`)

---

### GET /v{N}/orders

- **Auth**: Admin JWT
- **Request body**: —
- **Query params**: `page?: number`, `limit?: number`, `search?: string`, `status?: number`
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

### GET /v{N}/orders/{order_id}

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `order_id: number`
- **Response**: `ShopOrderDetail`

---

### GET /v{N}/orders/user/{user_num}

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `user_num: number`
- **Query params**: `page?: number`, `limit?: number`
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

### GET /v{N}/orders/user/{user_num}/items

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `user_num: number`
- **Query params**: `page?: number`, `limit?: number`
- **Response**:
```typescript
// {
//   data: ShopOrderItemWithDate[];
//   total_items: number;
//   page: number;
//   limit: number;
// }
```

---

### Mileage Endpoints (`/v{N}/mileage`)

---

### GET /v{N}/mileage/config

- **Auth**: Admin JWT
- **Request body**: —
- **Response**: `MileageConfig`

---

### PUT /v{N}/mileage/config

- **Auth**: Admin JWT
- **Request body**: `UpdateMileageConfigRequest`
- **Response**:
```typescript
// {
//   status: "success";
//   message: string;
//   data: MileageConfig;
// }
```

---

### GET /v{N}/mileage/orders

- **Auth**: Admin JWT
- **Request body**: —
- **Query params**: `page?: number`, `limit?: number`, `search?: string`, `status?: number`
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

### GET /v{N}/mileage/orders/{order_id}

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `order_id: number`
- **Response**: `ShopOrderDetail`

---

### Purchases Endpoints (`/v{N}/purchases`)

---

### GET /v{N}/purchases

- **Auth**: Admin JWT
- **Request body**: —
- **Query params**: `page?: number`, `limit?: number`, `search?: string`, `status?: number`
- **Response**:
```typescript
// {
//   data: ItemBankEntry[];
//   page: number;
//   limit: number;
//   total_items: number;
// }
```

---

### GET /v{N}/purchases/{id}

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `id: number`
- **Response**: `ItemBankEntry`

---

### GET /v{N}/purchases/{user_num}/user

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `user_num: number`
- **Query params**: `page?: number`, `limit?: number`, `status?: number`
- **Response**:
```typescript
// {
//   data: ItemBankEntry[];
//   total_items: number;
//   page: number;
//   limit: number;
// }
```
