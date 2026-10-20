# Email Admin Contract

> Generated from `ran-models/src/email/` and `ran-access-point/src/routes/admin/email/`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// mod.rs → EmailTemplate
export type EmailTemplate = {
  template_id: string;        // UUID
  template_name: string;
  template_category: string;
  subject_line: string;
  html_body: string;
  status: number;             // i16: 1 = Active, 0 = Inactive
  version: number;            // i32
  created_by: string;
  created_at: string;         // ISO 8601 (DateTime<Local>)
  updated_at: string;         // ISO 8601 (DateTime<Local>)
};

// mod.rs → CreateEmailTemplateRequest
export type CreateEmailTemplateRequest = {
  template_name: string;               // 1–100 chars
  template_category: string | null;    // max 50 chars
  subject_line: string;                // 1–200 chars
  html_body: string;                   // 1–100 KB
  status: number | null;               // i16, optional
};

// mod.rs → UpdateEmailTemplateRequest
export type UpdateEmailTemplateRequest = {
  template_name: string | null;        // 1–100 chars
  template_category: string | null;    // max 50 chars
  subject_line: string | null;         // 1–200 chars
  html_body: string | null;            // 1–100 KB
  status: number | null;               // i16, optional
};

// mod.rs → EmailTemplateFilterParams  (query string)
export type EmailTemplateFilterParams = {
  status: number | null;    // i16
  category: string | null;
  search: string | null;
  page: number | null;      // i16, default 1
  limit: number | null;     // i16, default 10
};

// mod.rs → PreviewTemplateRequest
export type PreviewTemplateRequest = {
  variables: Record<string, string> | null;
};

// mod.rs → EmailCampaign
export type EmailCampaign = {
  campaign_id: string;                // UUID
  campaign_name: string;
  template_id: string;                // UUID
  template_name: string;
  subject_line: string;
  html_body: string | null;
  segment_type: number;               // i16: 0 = All, 1 = Verified, 2 = Online within N days
  segment_param: number | null;       // i32, required when segment_type = 2 (days)
  status: number;                     // i16: 0 = Draft, 1 = Scheduled, 2 = Queued, 3 = Sending, 4 = Completed, 5 = Failed, 6 = Cancelled
  scheduled_at: string | null;        // ISO 8601
  started_at: string | null;          // ISO 8601
  completed_at: string | null;        // ISO 8601
  total_recipients: number;           // i32
  total_sent: number;                 // i32
  total_failed: number;               // i32
  created_by: string;
  created_at: string;                 // ISO 8601
  updated_at: string;                 // ISO 8601
};

// mod.rs → CreateEmailCampaignRequest
export type CreateEmailCampaignRequest = {
  campaign_name: string;              // 1–100 chars
  template_id: string;                // UUID
  segment_type: number;               // i16: 0 = All, 1 = Verified, 2 = Online within N days
  segment_param: number | null;       // i32, required when segment_type = 2
  scheduled_at: string | null;        // ISO 8601
};

// mod.rs → CampaignFilterParams  (query string)
export type CampaignFilterParams = {
  status: number | null;   // i16
  page: number | null;     // i16, default 1
  limit: number | null;    // i16, default 10
};

// mod.rs → CampaignRecipient
export type CampaignRecipient = {
  campaign_id: string;          // UUID
  user_num: number;             // i64
  user_email: string;
  user_name: string;
  send_status: number;          // i16
  attempt_count: number;        // i16
  sent_at: string | null;       // ISO 8601
  error_message: string | null;
};

// mod.rs → CampaignProgress
export type CampaignProgress = {
  campaign_id: string;       // UUID
  status: number;            // i16
  total_recipients: number;  // i32
  total_sent: number;        // i32
  total_failed: number;      // i32
  total_pending: number;     // i32
};

// mod.rs → CampaignProgressEvent  (WebSocket broadcast frame)
export type CampaignProgressEvent = {
  campaign_id: string;       // UUID
  status: number;            // i16
  total_recipients: number;  // i32
  total_sent: number;        // i32
  total_failed: number;      // i32
  total_pending: number;     // i32
};
```

---

## API Contract

All endpoints require **Admin JWT** (`Authorization: Bearer <token>`) via
`JwtMiddleware`. The base path prefix is `/admin/v1/email`.

---

### Templates — `/admin/v1/email/templates`

---

#### POST /admin/v1/email/templates

- **Auth**: Admin JWT
- **Request body**: `CreateEmailTemplateRequest`
- **Response**:
  ```
  // { success: true, data: EmailTemplate }
  ```

---

#### GET /admin/v1/email/templates

- **Auth**: Admin JWT
- **Request body**: —
- **Query params**: `EmailTemplateFilterParams`
- **Response**:
  ```
  // { data: EmailTemplate[], page: number, limit: number, total_items: number }
  ```

---

#### GET /admin/v1/email/templates/{id}

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `id` — UUID of the template
- **Response**:
  ```
  // { success: true, data: EmailTemplate }
  ```

---

#### PUT /admin/v1/email/templates/{id}

- **Auth**: Admin JWT
- **Request body**: `UpdateEmailTemplateRequest`
- **Path param**: `id` — UUID of the template
- **Response**:
  ```
  // { success: true, data: EmailTemplate }
  ```

---

#### DELETE /admin/v1/email/templates/{id}

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `id` — UUID of the template
- **Response**:
  ```
  // { success: true, data: EmailTemplate, message: "Template deactivated" }
  ```
  Note: this is a soft-delete (sets status to inactive), not a hard delete.

---

#### POST /admin/v1/email/templates/{id}/preview

- **Auth**: Admin JWT
- **Request body**: `PreviewTemplateRequest` (optional — may be omitted entirely)
- **Path param**: `id` — UUID of the template
- **Response**:
  ```
  // { success: true, data: { subject: string, html: string } }
  ```
  The server merges any supplied `variables` with default preview variables and
  renders the template before returning. The rendered HTML is safe to inject
  directly into an iframe for preview.

---

### Campaigns — `/admin/v1/email/campaigns`

---

#### POST /admin/v1/email/campaigns

- **Auth**: Admin JWT
- **Request body**: `CreateEmailCampaignRequest`
- **Response**:
  ```
  // { success: true, data: EmailCampaign }
  ```
  Validation notes:
  - `segment_type` must be `0`, `1`, or `2`.
  - `segment_param` (days) is required when `segment_type` is `2`.

---

#### GET /admin/v1/email/campaigns

- **Auth**: Admin JWT
- **Request body**: —
- **Query params**: `CampaignFilterParams`
- **Response**:
  ```
  // { data: EmailCampaign[], page: number, limit: number, total_items: number }
  ```

---

#### GET /admin/v1/email/campaigns/{id}

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `id` — UUID of the campaign
- **Response**:
  ```
  // { success: true, data: EmailCampaign }
  ```

---

#### POST /admin/v1/email/campaigns/{id}/queue

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `id` — UUID of the campaign
- **Response**:
  ```
  // { success: true, message: string, total_recipients: number }
  ```
  Snapshots recipients and transitions the campaign to **Queued** (`status = 2`).
  Only campaigns in **Draft** (`0`) or **Scheduled** (`1`) state can be queued.
  Returns immediately — the background worker handles actual sending.

---

#### POST /admin/v1/email/campaigns/{id}/cancel

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `id` — UUID of the campaign
- **Response**:
  ```
  // { success: true, message: "Campaign cancelled" }
  ```
  Only **Draft** (`0`), **Scheduled** (`1`), or **Queued** (`2`) campaigns can
  be cancelled. Sets status to `6` (Cancelled).

---

#### GET /admin/v1/email/campaigns/{id}/progress

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `id` — UUID of the campaign
- **Response**:
  ```
  // { success: true, data: CampaignProgress }
  ```
  HTTP fallback for clients that cannot use WebSocket.

---

### WebSocket — Campaign Progress

#### GET /admin/v1/email/campaigns/ws

- **Auth**: Admin JWT passed as query parameter `token=<JWT>` (not a header —
  browser WebSocket APIs cannot set `Authorization` headers)
- **Query param**: `token` — a valid admin JWT string
- **Protocol**: WebSocket upgrade
- **Server → client frames**: `CampaignProgressEvent` serialised as JSON text
  frames. Emitted by the background campaign worker whenever send counts change.
- **Client → server**: only `Ping` frames are processed (server replies with
  `Pong`). All other client messages are ignored.
- **Disconnect**: send a `Close` frame or simply drop the connection.

Example connect URL:
```
wss://api.example.com/admin/v1/email/campaigns/ws?token=eyJ...
```

---

## Status Code Reference

| Domain | Value | Meaning |
|--------|-------|---------|
| Template `status` | `1` | Active |
| Template `status` | `0` | Inactive (soft-deleted) |
| Campaign `status` | `0` | Draft |
| Campaign `status` | `1` | Scheduled |
| Campaign `status` | `2` | Queued |
| Campaign `status` | `3` | Sending |
| Campaign `status` | `4` | Completed |
| Campaign `status` | `5` | Failed |
| Campaign `status` | `6` | Cancelled |
| Campaign `segment_type` | `0` | All users |
| Campaign `segment_type` | `1` | Verified users only |
| Campaign `segment_type` | `2` | Online within last N days (`segment_param` = N) |
