# Payments Contract

> Generated from `ran-models/src/payments/` and `ran-access-point/src/routes/payments/`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// gcash/mod.rs → GCashTransaction
export type GCashTransaction = {
  reference_number: string;
  proof_image: string | null;
};

// wise/mod.rs → WiseTransaction
export type WiseTransaction = {
  reference_number: string;
};

// paypal/mod.rs → PaypalCheckoutPayload
export type PaypalCheckoutPayload = Record<string, never>;

// mod.rs → TransactionPayload (tagged union, serde tag = "type")
// Sent as request body to POST /transaction/{id} for GCash (type=1) and Wise (type=3).
// PayPal (type=2) and PayMongo (type=4) require no body.
export type TransactionPayload =
  | { type: "gcash" } & GCashTransaction
  | { type: "wise" } & WiseTransaction;

// mod.rs → WebTransaction (internal; used to build WebTransactionResponse)
export type WebTransaction = {
  transaction_id: string;       // UUID
  user_num: number;
  user_name: string;
  transaction_type: number;
  transaction_time: string;     // ISO 8601 (local timezone)
  transaction_price: number;
  transaction_status: number;
  transaction_amount: number;
  updated_at: string | null;    // ISO 8601 (local timezone)
  transaction_cost: number;
  proof_image: number[] | null; // raw bytes — not returned directly to clients
  reference_code: string | null;
  admin_remarks: string | null;
};

// mod.rs → WebTransactionResponse
// proof_image is base64-encoded when present.
export type WebTransactionResponse = {
  transaction_id: string;       // UUID
  user_num: number;
  user_name: string;
  transaction_type: number;
  transaction_time: string;     // ISO 8601 (local timezone)
  transaction_price: number;
  transaction_status: number;
  transaction_amount: number;
  updated_at: string | null;    // ISO 8601 (local timezone)
  transaction_cost: number;
  proof_image: string | null;   // base64-encoded image data
  reference_code: string | null;
  admin_remarks: string | null;
};

// mod.rs → Transaction
export type Transaction = {
  user_num: number;
  user_name: string;
  type: number;
  amount: number;
  points: number;
};

// mod.rs → RechargeInfo
export type RechargeInfo = {
  id: string;           // UUID
  type: number;
  amount: number;
  discount: number | null;
  points: number;
  status: number;
};

// mod.rs → AddRechargeInfo
export type AddRechargeInfo = {
  type: number;
  amount: number;
  discount: number | null;
  points: number;
  status: number;
};

// mod.rs → AddGatewayInfo
export type AddGatewayInfo = {
  provider: number;
  name: string;
  number: string;
  image: string;
  status: number;
};

// mod.rs → GatewayInfo
export type GatewayInfo = {
  id: string;           // UUID
  provider: number;
  name: string;
  number: string;
  image: string;
  status: number;
};

// lib.rs → PaginationParams (query parameters)
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

### GET /payments/get-recharge-info

- **Auth**: None
- **Request body**: —
- **Response**: `RechargeInfo[]`
  - Returns only active recharge packages (status = 1).

---

### GET /payments/get-recharge-info/{id}

- **Auth**: None
- **Request body**: —
- **Path param**: `id` — UUID of the recharge package
- **Response**: `RechargeInfo`

---

### POST /payments/transaction/{id}

- **Auth**: User session (cookie)
- **Path param**: `id` — UUID of the recharge package to process
- **Request body**: `TransactionPayload | null`
  - Required for `transaction_type = 1` (GCash) and `transaction_type = 3` (Wise).
  - Omit body for `transaction_type = 2` (PayPal) and `transaction_type = 4` (PayMongo).
- **Response** (varies by payment type):
  - GCash / Wise:
    ```
    // { message: string; data: GCashTransaction | WiseTransaction; recharge_info: RechargeInfo }
    ```
  - PayPal:
    ```
    // { approval_url: string }
    ```
  - PayMongo:
    ```
    // { message: string; checkout_session_id: string; status: string; approval_url: string; recharge_info: RechargeInfo }
    ```

---

### GET /payments/get-gateway-info

- **Auth**: None
- **Request body**: —
- **Response**: `GatewayInfo[]`
  - Returns only active gateways (status = 1).

---

### GET /payments/get-recharge-logs

- **Auth**: User session (cookie)
- **Query params**: `PaginationParams` (`page`, `limit`, `status`, `category`)
- **Request body**: —
- **Response**:
  ```
  // {
  //   page: number;
  //   limit: number;
  //   total_pages: number;
  //   total_items: number;
  //   data: WebTransactionResponse[];
  // }
  ```

---

### POST /payments/webhook/paypal

- **Auth**: None (called by PayPal — no user session or JWT)
- **Request body**: `unknown` (raw PayPal webhook event JSON)
- **Response**:
  ```
  // { message: string }
  ```
  - Handles event types: `CHECKOUT.ORDER.APPROVED`, `CHECKOUT.CAPTURE.COMPLETED`, `PAYMENT.CAPTURE.COMPLETED`, `PAYMENT.CAPTURE.DENIED`, `PAYMENT.CAPTURE.DECLINED`.
  - All other event types are acknowledged with HTTP 200.

---

### POST /payments/webhook/paymongo

- **Auth**: None (called by PayMongo — no user session or JWT)
- **Request body**: `PayMongoWebhookPayload` (raw bytes)
- **Response**: `"Webhook received and processed"` (plain text body, HTTP 200)
  - Handles event types: `payment.paid`, `payment.failed`.
