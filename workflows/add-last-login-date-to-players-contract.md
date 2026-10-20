# API Contract: Player Endpoints with Last Login Date

**Document Version:** 1.0  
**Status:** Active  
**Last Updated:** 2026-05-26

---

## Overview

This document describes the API contract for two admin player endpoints that now include the `last_login_date` field. This new field tracks when a player last logged into the system, enabling admins to identify inactive accounts and analyze player engagement.

**Endpoints:**
- `GET /admin/v1/players` — List players with pagination and filtering
- `GET /admin/v1/players/{id}` — Get detailed information for a single player

---

## TypeScript Interfaces

### UserAccountInfo

Complete player account information returned by both endpoints.

```typescript
interface UserAccountInfo {
  /** Unique identifier for the user (UUID format) */
  user_id: string;

  /** Internal numeric user identifier */
  user_num: number;

  /** Username (login name) */
  user_name: string;

  /** User type/category code (1 = regular user, etc.) */
  user_type: number;

  /** Login state (0 = offline, 1 = online) */
  user_login_state: number;

  /** Account availability (0 = disabled, 1 = active) */
  user_available: number;

  /** Block status (0 = not blocked, positive integer = blocked with block ID) */
  user_block: number;

  /** ISO 8601 timestamp when account was blocked (e.g., "2025-12-01T14:30:00+09:00") */
  user_block_date: string;

  /** ISO 8601 timestamp when account was created (e.g., "2023-01-15T10:00:00+09:00") */
  user_create_date: string;

  /** ISO 8601 timestamp of the last login (NEW FIELD - e.g., "2026-05-25T18:45:00+09:00") */
  last_login_date: string;

  /** Email address associated with the account */
  user_email: string;

  /** Web points balance (double precision) */
  user_web_points: number;

  /** Email verification status (0 = not verified, 1 = verified) */
  user_verified: number;
}
```

### GetPlayersListResponse

Response structure for the list endpoint with pagination metadata.

```typescript
interface GetPlayersListResponse {
  /** Array of player account information objects */
  data: UserAccountInfo[];

  /** Current page number (1-indexed) */
  page: number;

  /** Items returned per page */
  limit: number;

  /** Total count of all matching records in the database */
  total_items: number;
}
```

---

## Field Documentation Table

| Field | Type | Description | Notes |
|-------|------|-------------|-------|
| `user_id` | `string` (UUID) | Unique identifier for the user account | Globally unique across all databases |
| `user_num` | `number` | Internal numeric ID used in game database | Primary key in game operations |
| `user_name` | `string` | Username (player's login name) | Lowercase, 3-32 chars typically |
| `user_type` | `number` | User category/type code | `1` = regular user; consult schema for other values |
| `user_login_state` | `number` | Current login status | `0` = offline, `1` = currently online |
| `user_available` | `number` | Account availability flag | `0` = account disabled, `1` = active |
| `user_block` | `number` | Block status identifier | `0` = not blocked, `> 0` = block record ID |
| `user_block_date` | `string` (ISO 8601) | When the account was last blocked | Can be epoch if never blocked |
| `user_create_date` | `string` (ISO 8601) | Account creation timestamp | Server timezone (Local) |
| `last_login_date` | `string` (ISO 8601) | **When the player last logged in** | **NEW FIELD** — Can be epoch if never logged in |
| `user_email` | `string` | Email address for the account | Used for password reset, notifications |
| `user_web_points` | `number` | Account's web points balance | Double precision (decimal) |
| `user_verified` | `number` | Email verification status | `0` = not verified, `1` = verified |

---

## Endpoint: GET /admin/v1/players

**List all players with optional filtering and pagination.**

### Request

**Method:** `GET`  
**Path:** `/admin/v1/players`  
**Auth:** JWT Bearer token required

### Query Parameters

| Parameter | Type | Required | Description | Example |
|-----------|------|----------|-------------|---------|
| `page` | `number` | No | Page number (1-indexed, default: 1) | `page=2` |
| `limit` | `number` | No | Items per page (default: 10) | `limit=20` |
| `category` | `number[]` | No | Filter by user type(s) | `category=1&category=2` |
| `status` | `number[]` | No | Filter by block status | `status=0` for active, `status=1` for blocked |
| `search` | `string` | No | Search by username | `search=player_name` |

### Response

**Status:** `200 OK`

**Content-Type:** `application/json`

**Body:**

```json
{
  "data": [
    {
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "user_num": 12345,
      "user_name": "adventurer_001",
      "user_type": 1,
      "user_login_state": 1,
      "user_available": 1,
      "user_block": 0,
      "user_block_date": "1970-01-01T00:00:00+09:00",
      "user_create_date": "2023-01-15T10:30:00+09:00",
      "last_login_date": "2026-05-25T18:45:30+09:00",
      "user_email": "player@example.com",
      "user_web_points": 1500.50,
      "user_verified": 1
    },
    {
      "user_id": "660e8400-e29b-41d4-a716-446655440001",
      "user_num": 12346,
      "user_name": "warrior_king",
      "user_type": 1,
      "user_login_state": 0,
      "user_available": 1,
      "user_block": 0,
      "user_block_date": "1970-01-01T00:00:00+09:00",
      "user_create_date": "2022-06-20T14:15:00+09:00",
      "last_login_date": "2026-05-10T22:10:00+09:00",
      "user_email": "warrior@example.com",
      "user_web_points": 3200.00,
      "user_verified": 1
    },
    {
      "user_id": "770e8400-e29b-41d4-a716-446655440002",
      "user_num": 12347,
      "user_name": "shadow_mage",
      "user_type": 1,
      "user_login_state": 0,
      "user_available": 1,
      "user_block": 1,
      "user_block_date": "2026-03-01T09:00:00+09:00",
      "user_create_date": "2021-11-05T08:00:00+09:00",
      "last_login_date": "2026-02-28T20:30:00+09:00",
      "user_email": "mage@example.com",
      "user_web_points": 0.00,
      "user_verified": 0
    }
  ],
  "page": 1,
  "limit": 10,
  "total_items": 1250
}
```

### Error Responses

**Status:** `400 Bad Request`

```json
{
  "error": "Invalid query parameters"
}
```

**Status:** `401 Unauthorized`

```json
{
  "error": "Missing or invalid JWT token"
}
```

**Status:** `500 Internal Server Error`

```json
{
  "error": "Failed to fetch players list."
}
```

---

## Endpoint: GET /admin/v1/players/{id}

**Get detailed information for a specific player.**

### Request

**Method:** `GET`  
**Path:** `/admin/v1/players/{id}`  
**Auth:** JWT Bearer token required

### Path Parameters

| Parameter | Type | Required | Description | Example |
|-----------|------|----------|-------------|---------|
| `id` | `number` | Yes | Player's `user_num` (numeric ID) | `12345` |

### Response

**Status:** `200 OK`

**Content-Type:** `application/json`

**Body:**

```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "user_num": 12345,
  "user_name": "adventurer_001",
  "user_type": 1,
  "user_login_state": 1,
  "user_available": 1,
  "user_block": 0,
  "user_block_date": "1970-01-01T00:00:00+09:00",
  "user_create_date": "2023-01-15T10:30:00+09:00",
  "last_login_date": "2026-05-25T18:45:30+09:00",
  "user_email": "player@example.com",
  "user_web_points": 1500.50,
  "user_verified": 1
}
```

### Error Responses

**Status:** `401 Unauthorized`

```json
{
  "error": "Missing or invalid JWT token"
}
```

**Status:** `404 Not Found`

```json
{
  "error": "Player not found"
}
```

**Status:** `500 Internal Server Error`

```json
{
  "error": "Failed to fetch player data."
}
```

---

## Integration Notes for Frontend Team

### DateTime Handling

All datetime fields (`user_block_date`, `user_create_date`, `last_login_date`) are serialized as ISO 8601 strings with timezone offset (e.g., `2026-05-25T18:45:30+09:00`). The server uses the local timezone (Asia/Seoul or UTC+9 by default).

**Recommended TypeScript parsing:**

```typescript
// Using the native Date constructor
const lastLogin = new Date("2026-05-25T18:45:30+09:00");

// Using a library like date-fns or dayjs
import { parseISO } from 'date-fns';
const lastLogin = parseISO("2026-05-25T18:45:30+09:00");
```

### NULL/Default Datetime Handling

If a player has never logged in or a date field is NULL in the database, the field will contain the epoch timestamp: `"1970-01-01T00:00:00+09:00"`. Check for this value to determine if the field is effectively empty.

**Frontend check example:**

```typescript
const epochDate = new Date("1970-01-01T00:00:00+09:00");

function isUnsetDate(dateString: string): boolean {
  return new Date(dateString).getTime() === epochDate.getTime();
}

// Usage
if (isUnsetDate(player.last_login_date)) {
  console.log("Player has never logged in");
}
```

### Last Login Date (NEW)

The `last_login_date` field tracks the most recent login event for each player. Use this to:

- **Identify inactive accounts:** Compare against current date to find players who haven't logged in for 30+ days
- **Engagement analysis:** Build reports on active vs. inactive user segments
- **Account health dashboards:** Display "last seen" information in admin UIs

**Example usage:**

```typescript
function getInactivityDays(lastLoginDate: string): number {
  const lastLogin = new Date(lastLoginDate);
  const now = new Date();
  const diffMs = now.getTime() - lastLogin.getTime();
  return Math.floor(diffMs / (1000 * 60 * 60 * 24));
}

// Usage
const inactivityDays = getInactivityDays(player.last_login_date);
if (inactivityDays > 30) {
  console.log(`Player ${player.user_name} is inactive (${inactivityDays} days)`);
}
```

### Filtering and Pagination

The list endpoint supports multi-value filtering via repeated query parameters:

```
GET /admin/v1/players?category=1&category=2&status=0&page=1&limit=20
```

This retrieves players with `user_type` of 1 or 2, block status 0, page 1, 20 items per page.

**Frontend pagination helper:**

```typescript
interface ListFilters {
  page?: number;
  limit?: number;
  category?: number[];
  status?: number[];
  search?: string;
}

function buildPlayersUrl(filters: ListFilters): string {
  const params = new URLSearchParams();
  
  if (filters.page) params.append('page', filters.page.toString());
  if (filters.limit) params.append('limit', filters.limit.toString());
  
  filters.category?.forEach(cat => params.append('category', cat.toString()));
  filters.status?.forEach(st => params.append('status', st.toString()));
  
  if (filters.search) params.append('search', filters.search);
  
  return `/admin/v1/players?${params.toString()}`;
}
```

### Block Status Field

The `user_block` field indicates if an account is banned:

| Value | Meaning |
|-------|---------|
| `0` | Not blocked (active account) |
| `> 0` | Block record ID (account is banned) |

Use in conjunction with `user_available` to determine full account state:

```typescript
function getAccountStatus(player: UserAccountInfo): string {
  if (player.user_available === 0) return "Disabled";
  if (player.user_block > 0) return "Banned";
  return "Active";
}
```

### Response Field Guarantees

All fields are guaranteed to be present in the response and non-null. However:

- DateTime fields default to epoch (`1970-01-01T00:00:00+09:00`) if never set
- String fields default to empty string (`""`) if NULL in database
- Numeric fields default to `0` if NULL in database

### Timezone Note

All timestamps from the API use the server's local timezone (typically UTC+9 for Asia/Seoul). When storing or comparing dates on the frontend, ensure consistent timezone handling:

```typescript
// Always parse with timezone awareness
const ts = new Date("2026-05-25T18:45:30+09:00");

// Be careful with toISOString() — it converts to UTC
console.log(ts.toISOString()); // "2026-05-25T09:45:30.000Z"

// Use a library for timezone-aware operations
import { formatInTimeZone } from 'date-fns-tz';
const koreaTime = formatInTimeZone(ts, 'Asia/Seoul', 'yyyy-MM-dd HH:mm:ss');
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-05-26 | Initial contract with `last_login_date` field added |

---

## Change Log: last_login_date Addition

**Deployment Date:** 2026-05-26

### What Changed

The `UserAccountInfo` struct now includes a `last_login_date` field that tracks the most recent login timestamp for each player. This field is included in both list and detail endpoints.

### Database Updates

- New column `LastLoginDate` (NaiveDateTime) added to the players table
- Stored procedures `sp_GetPlayersList` and `sp_GetPlayerById` updated to return the new column
- Field populated automatically by game server login events

### Backend Changes

- `UserAccountInfo` struct in `ran-models/src/admin/players/mod.rs` updated
- Field deserialized from `NaiveDateTime` database rows to ISO 8601 strings
- Default value is epoch timestamp if NULL or never set

### Frontend Migration

**No breaking changes.** The new field is additive and safe to ignore if not yet implemented.

To use the new field:

1. Update TypeScript type definitions to include `last_login_date`
2. Parse datetime using native Date or utility library
3. Handle epoch dates (`1970-01-01T00:00:00+09:00`) as "never set"
4. Use in reports and UI dashboards for inactivity tracking

---

## Related Endpoints

- `GET /admin/v1/players/{id}/transaction-history` — Transaction and recharge history
- `GET /admin/v1/players/{id}/ban-history` — Account ban/block history
- `GET /admin/v1/players/{id}/characters` — Characters owned by player
- `POST /admin/v1/players/{id}/ban-account` — Ban/unban player account

---

## Questions or Issues?

For API contract clarifications, contact the backend team. For frontend integration support, refer to the `Integration Notes` section above.
