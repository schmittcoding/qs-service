# Recharge Admin Contract

> Generated from `ran-models/src/payments/mod.rs` and `ran-access-point/src/routes/admin/recharge/mod.rs`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// payments/mod.rs → WebTransactionResponse
export type WebTransactionResponse = {
  transaction_id: string;        // UUID
  user_num: number;
  user_name: string;
  transaction_type: number;      // 1 = GCash, 2 = PayPal
  transaction_time: string;      // ISO 8601 local datetime
  transaction_price: number;
  transaction_status: number;    // 0 = Pending, 1 = Approved, 2 = Declined
  transaction_amount: number;
  updated_at: string | null;     // ISO 8601 local datetime
  transaction_cost: number;
  proof_image: string | null;    // Base64-encoded image
  reference_code: string | null;
  admin_remarks: string | null;
};

// payments/mod.rs → RechargeInfo
export type RechargeInfo = {
  id: string;             // UUID
  type: number;           // recharge type
  amount: number;
  discount: number | null;
  points: number;
  status: number;
};

// payments/mod.rs → AddRechargeInfo (request body for add/update recharge)
export type AddRechargeInfo = {
  type: number;
  amount: number;
  discount: number | null;
  points: number;
  status: number;
};

// payments/mod.rs → GatewayInfo
export type GatewayInfo = {
  id: string;       // UUID
  provider: number;
  name: string;
  number: string;
  image: string;    // URL or base64
  status: number;
};

// payments/mod.rs → AddGatewayInfo (request body for add/update gateway)
export type AddGatewayInfo = {
  provider: number;
  name: string;
  number: string;
  image: string;
  status: number;
};
```

## Query Params (shared)

```typescript
// Used on paginated list endpoints (legacy — recharge-logs only)
export type PaginationParams = {
  page?: number;
  limit?: number;
  status?: number;   // 0 = Pending, 1 = Approved, 2 = Declined
  category?: number; // 1 = GCash, 2 = PayPal
  search?: string;
  start_date?: string;
  end_date?: string;
};

// Used on GET /recharge — supports multi-value filters
export type TransactionFilterParams = {
  page?: number;
  limit?: number;
  search?: string;
  method?: number[];  // transaction type: 1 = GCash, 2 = PayPal (multi-value)
  status?: number[];  // 0 = Pending, 1 = Approved, 2 = Declined (multi-value)
};
```

## API Contract

Base path: `/admin/v{N}/recharge`
Auth: **Admin JWT** (Bearer token — all endpoints are under `admin_route()`)

---

### GET /admin/v{N}/recharge

Paginated transaction list backed by `sp_GetWebTransactions`. Preferred over `/recharge-logs`.
Results are always sorted pending-first (`status = 0`), then by `TransactionTime DESC`.

- **Auth**: Admin JWT
- **Query**: `TransactionFilterParams` — `method` and `status` accept multiple values (e.g. `?status[]=0&status[]=1&method[]=1`)
- **Request body**: —
- **Response**:
```typescript
// { data: WebTransactionResponse[], page: number, limit: number, total_items: number }
```

---

### GET /admin/v{N}/recharge/recharge-logs

Paginated recharge log list. Filtering by `status` is done at DB level; filtering by `category` and `search` is done in-process.

- **Auth**: Admin JWT
- **Query**: `PaginationParams` (`page`, `limit`, `status`, `category`, `search`)
- **Request body**: —
- **Response**:
```typescript
// {
//   page: number,
//   limit: number,
//   total_pages: number,
//   total_items: number,
//   data: WebTransactionResponse[]
// }
```

---

### PUT /admin/v{N}/recharge/update-transaction/{id}

Approve or decline a transaction by UUID. Also fires a Discord webhook notification.

- **Auth**: Admin JWT
- **Path param**: `id` — transaction UUID
- **Query**: `status: number` (0 = Pending, 1 = Approved, 2 = Declined)
- **Request body**: —
- **Response**:
```typescript
// { message: "Transaction updated successfully" }
```

---

### GET /admin/v{N}/recharge/get-recharge-info

List recharge tiers with optional filtering; response is paginated in-process.

- **Auth**: Admin JWT
- **Query**: `PaginationParams` (`page`, `limit`, `status`, `category`)
- **Request body**: —
- **Response**:
```typescript
// { data: RechargeInfo[], total_items: number }
```

---

### POST /admin/v{N}/recharge/add-recharge-info

Create a new recharge tier. Uses inline SQL (not a stored procedure).

- **Auth**: Admin JWT
- **Request body**: `AddRechargeInfo`
- **Response**:
```typescript
// { message: "Added new recharge info successfully" }
```

---

### PUT /admin/v{N}/recharge/update-recharge-info/{id}

Update an existing recharge tier by UUID.

- **Auth**: Admin JWT
- **Path param**: `id` — recharge info UUID
- **Request body**: `AddRechargeInfo`
- **Response**: `RechargeInfo`

---

### GET /admin/v{N}/recharge/get-gateway-info

List all payment gateway accounts (GCash numbers, PayPal accounts, etc.).

- **Auth**: Admin JWT
- **Request body**: —
- **Response**: `GatewayInfo[]`

---

### POST /admin/v{N}/recharge/add-gateway-info

Add a new payment gateway account.

- **Auth**: Admin JWT
- **Request body**: `AddGatewayInfo`
- **Response**:
```typescript
// { message: "Added new gateway info successfully" }
```

---

### PUT /admin/v{N}/recharge/update-gateway-info/{id}

Update an existing payment gateway account by UUID.

- **Auth**: Admin JWT
- **Path param**: `id` — gateway info UUID
- **Request body**: `AddGatewayInfo`
- **Response**: `GatewayInfo`

---

## Notes

- `transaction_type` / `method` values: `1` = GCash, `2` = PayPal.
- `transaction_status` / `status` values: `0` = Pending, `1` = Approved, `2` = Declined.
- `GET /recharge` uses `TransactionFilterParams` (multi-value `method[]` and `status[]`). The legacy `GET /recharge-logs` still uses single-value `PaginationParams` with `category`.
- `GET /recharge` results are always sorted pending-first by the stored procedure — no client-side reordering needed.
- Multi-value query params use bracket notation: `?status[]=0&status[]=1` or `?method[]=1&method[]=2`.
- `proof_image` on `WebTransactionResponse` is Base64-encoded; consumers should render via `data:image/...;base64,<value>`.
- `add_recharge_info` uses inline SQL directly against the pool rather than a stored procedure — this is a deviation from the project standard.
- There is no `ran-models/src/recharge/` module; all types live under `ran-models/src/payments/`.
