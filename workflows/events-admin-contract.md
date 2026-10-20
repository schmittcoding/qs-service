# Events Admin Contract

> Generated from `ran-models/src/events/` and `ran-access-point/src/routes/admin/events/`.
> Do not edit manually — re-run ContractGen to regenerate.

## Notes

- All admin event routes are mounted under the admin scope (e.g., `/admin/v1/events`). All paths below are shown relative to that prefix as `/admin/events/...`.
- Auth for every endpoint is **Admin JWT** (Bearer token via `JwtMiddleware`). The handler additionally calls `get_admin_id(&req)` to extract the admin UUID from JWT claims and returns `401` if missing or `400` if the UUID is malformed.
- `EventDefinitionAdmin` uses `#[serde(flatten)]` on `base: EventDefinition`, so all `EventDefinition` fields appear at the top level alongside `registration_count` and `match_count`.
- `CategoryConfig` is a discriminated union with `"type"` as the tag. See the user contract for the full breakdown. The admin contract repeats only types specific to admin request bodies; all response types reference the shared model definitions.
- `CreateEventRequest` and `UpdateEventRequest` supply category-specific settings inside `category_config` — there are no top-level `match_duration_min`, `tally_delay_min`, or `min_guild_members` fields.
- `tower_points_per_tick` does not exist — do not send or expect it.
- `GvgKoth` is a legacy category. `POST /admin/events` and `PUT /admin/events/{event_id}` reject it with `400 LEGACY_CATEGORY`.
- Match creation (`POST /admin/events/{event_id}/matches` and `POST /admin/events/{event_id}/matches/generate`) returns `400` with `{ "status": "error", "message": "Matches are only supported for GvG, KOTH, and GvG+KOTH events" }` when the event category is `level_cap_race` or `speed_run`.
- CSV export endpoints return `text/csv` with a `Content-Disposition: attachment` header — not JSON.
- Admin routes are split into sub-modules: `crud`, `match_management`, `gimmicks`, `participants`, `registration`, `tally`, `export`, `deductions`.

---

## TypeScript Types

```typescript
// Shared types — defined in full in events-contract.md; repeated here for reference.

// category.rs → EventCategory
export type EventCategory =
  | "gvg"
  | "koth"
  | "gvg_koth"
  | "level_cap_race"
  | "speed_run";

// category_config.rs → GvgConfig / KothConfig / GvgKothConfig / LevelCapRaceConfig / SpeedRunConfig / CategoryConfig
export type GvgConfig = {
  min_guild_members: number;
  match_duration_min: number;
  tally_delay_min: number;
  event_map_mid: number;
  event_map_sid: number;
};
export type KothConfig = {
  match_duration_min: number;
  tally_delay_min: number;
  event_map_mid: number;
  event_map_sid: number;
};
export type GvgKothConfig = {
  min_guild_members: number;
  match_duration_min: number;
  tally_delay_min: number;
  event_map_mid: number;
  event_map_sid: number;
};
export type LevelCapRaceConfig = {
  max_account_age_days: number;
  target_level: number;
};
export type SpeedRunConfig = {
  dungeon_map_mid: number;
  dungeon_map_sid: number;
  boss_mob_id: number | null;
};
export type CategoryConfig =
  | ({ type: "gvg" } & GvgConfig)
  | ({ type: "koth" } & KothConfig)
  | ({ type: "gvg_koth" } & GvgKothConfig)
  | ({ type: "level_cap_race" } & LevelCapRaceConfig)
  | ({ type: "speed_run" } & SpeedRunConfig);

// event.rs → EventDefinition
export type EventDefinition = {
  event_id: string;           // UUID
  event_name: string;
  event_slug: string;
  season: number;
  registration_open: string;  // ISO NaiveDateTime
  registration_close: string; // ISO NaiveDateTime
  event_start: string;        // ISO NaiveDateTime
  event_end: string;          // ISO NaiveDateTime
  min_level: number;
  is_active: boolean;
  event_category: EventCategory;
  category_config: CategoryConfig | null;
  created_at: string;         // ISO NaiveDateTime
};

// event.rs → EventDefinitionAdmin  (#[serde(flatten)] on base)
// All EventDefinition fields appear at the top level.
export type EventDefinitionAdmin = EventDefinition & {
  registration_count: number;
  match_count: number;
};

// event.rs → CreateEventRequest
export type CreateEventRequest = {
  event_name: string;
  event_slug: string;
  season: number;
  registration_open: string;   // ISO NaiveDateTime
  registration_close: string;  // ISO NaiveDateTime
  event_start: string;         // ISO NaiveDateTime
  event_end: string;           // ISO NaiveDateTime
  min_level: number;
  event_category?: EventCategory | null;
  category_config?: CategoryConfig | null;
};

// event.rs → UpdateEventRequest  (all fields optional — omit to leave unchanged)
export type UpdateEventRequest = {
  event_name?: string | null;
  event_slug?: string | null;
  registration_open?: string | null;   // ISO NaiveDateTime
  registration_close?: string | null;  // ISO NaiveDateTime
  event_start?: string | null;         // ISO NaiveDateTime
  event_end?: string | null;           // ISO NaiveDateTime
  min_level?: number | null;
  event_category?: EventCategory | null;
  category_config?: CategoryConfig | null;
};

// event.rs → CreateMatchRequest
export type CreateMatchRequest = {
  match_label?: string | null;
  match_date: string;       // ISO NaiveDate (YYYY-MM-DD)
  start_time: string;       // ISO NaiveDateTime
  tower_points_th?: number; // defaults to 0
  tower_points_nuc?: number;
  tower_points_faci?: number;
};

// event.rs → CreateMatchesRequest  (bulk variant — send when payload has "matches" key)
export type CreateMatchesRequest = {
  matches: CreateMatchRequest[];
};

// event.rs → GenerateMatchesRequest
export type GenerateMatchesRequest = {
  start_date: string;        // ISO NaiveDate
  end_date: string;          // ISO NaiveDate
  days_of_week: string[];    // e.g. ["Monday", "Friday"]
  start_hour: number;        // 0–23
  start_minute: number;      // 0–59
  label_prefix?: string;     // defaults to "Week"
};

// event.rs → UpdateMatchRequest  (all fields optional)
export type UpdateMatchRequest = {
  match_label?: string | null;
  start_time?: string | null;       // ISO NaiveDateTime
  tower_points_th?: number | null;
  tower_points_nuc?: number | null;
  tower_points_faci?: number | null;
};

// event.rs → SetTowerResultsRequest
export type SetTowerResultsRequest = {
  tower_winner_th?: number | null;    // ChaSchool INT; null = no winner
  tower_winner_nuc?: number | null;
  tower_winner_faci?: number | null;
};

// match_schedule.rs → MatchSchedule
export type MatchSchedule = {
  match_id: string;                   // UUID
  event_id: string;                   // UUID
  match_label: string | null;
  match_date: string;                 // ISO NaiveDate
  start_time: string;                 // ISO NaiveDateTime
  end_time: string;                   // ISO NaiveDateTime
  tally_scheduled: string;            // ISO NaiveDateTime
  tally_status: string;               // "Pending" | "Waiting" | "Processing" | "Completed" | "Failed"
  tally_started_at: string | null;
  tally_completed_at: string | null;
  rows_tallied: number | null;
  kills_counted: number | null;
  kills_rejected: number | null;
  tower_points_th: number;
  tower_points_nuc: number;
  tower_points_faci: number;
  tower_winner_th: number | null;
  tower_winner_nuc: number | null;
  tower_winner_faci: number | null;
};

// event.rs → EventDashboard (response shape for GET /{event_id}/dashboard)
export type RegistrationStats = {
  total_registered: number;
  active: number;
  withdrawn: number;
  disqualified: number;
  guild_count: number;
  school_breakdown: Array<{ cha_school: number; count: number }>;
};
export type MatchStats = {
  total_matches: number;
  completed: number;
  pending: number;
  next_match: {
    match_id: string;         // UUID
    label: string | null;
    start_time: string;       // ISO NaiveDateTime
  } | null;
};
export type DeductionStats = {
  total_deductions: number;
  total_points_deducted: number;
  reverted: number;
};

// deduction.rs → AdminDeduction
export type AdminDeduction = {
  deduction_id: string;         // UUID
  event_id: string;             // UUID
  target_type: string;          // "GUILD" | "PLAYER" | "SCHOOL"
  target_id: number | null;
  target_cha_school: number | null;
  points: number;
  reason: string;
  admin_user_id: string;        // UUID
  applied_at: string;           // ISO NaiveDateTime
  match_id: string | null;      // UUID
  is_reverted: boolean;
  reverted_at: string | null;   // ISO NaiveDateTime
  reverted_by: string | null;   // UUID
};

// deduction.rs → DeductionRequest
export type DeductionRequest = {
  target_type: string;               // "GUILD" | "PLAYER" | "SCHOOL"
  target_id?: number | null;         // required when target_type is "GUILD" or "PLAYER"
  target_cha_school?: number | null; // required when target_type is "SCHOOL"
  points: number;                    // must be > 0
  reason: string;                    // must be non-empty
  match_id?: string | null;          // UUID — optional match association
};

// gimmick.rs → GimmickType  (#[serde(rename_all = "PascalCase")])
export type GimmickType = "DoublePoints" | "NullDeaths";

// gimmick.rs → EventMatchGimmick
export type EventMatchGimmick = {
  gimmick_id: string;           // UUID
  match_id: string;             // UUID
  gimmick_type: GimmickType;
  gimmick_start_time: string;   // ISO NaiveDateTime
  gimmick_end_time: string;     // ISO NaiveDateTime
  created_at: string;           // ISO NaiveDateTime
};

// gimmick.rs → CreateGimmickRequest
export type CreateGimmickRequest = {
  gimmick_type: GimmickType;
  gimmick_start_time: string;   // ISO NaiveDateTime
  gimmick_end_time: string;     // ISO NaiveDateTime
};

// registration.rs → EventRegistration
export type EventRegistration = {
  registration_id: string;    // UUID
  event_id: string;           // UUID
  cha_num: number;
  user_num: number;
  guild_num: number | null;
  cha_school: number | null;
  cha_class: number;
  cha_level: number;
  status: number;             // 1=Active, 2=Withdrawn, 3=Disqualified
  registered_at: string;      // ISO NaiveDateTime
  updated_at: string | null;
  ip_address: string | null;
};

// admin/events/registration.rs → AdminUpdateRegistrationRequest  (local to handler file)
export type AdminUpdateRegistrationRequest = {
  status?: number | null;      // 1=Active, 2=Withdrawn, 3=Disqualified
  guild_num?: number | null;   // null clears guild assignment; omit to leave unchanged
};
```

---

## API Contract

### GET /admin/events

- **Auth**: Admin JWT
- **Request body**: —
- **Response**:
  ```
  { success: true; events: EventDefinitionAdmin[] }
  ```

---

### POST /admin/events

- **Auth**: Admin JWT
- **Request body**: `CreateEventRequest`
- **Notes**: `gvg_koth` category is rejected as a legacy category.
- **Response**:
  ```
  // 201 Created
  { success: true; event: EventDefinition }

  // 400 — legacy category
  { success: false; code: "LEGACY_CATEGORY"; message: string }

  // 400 — date validation
  { success: false; code: "INVALID_DATES"; message: string }

  // 409 — slug already exists
  { success: false; code: "CONFLICT"; message: "An event with this slug already exists" }
  ```

---

### GET /admin/events/{event_id}

- **Auth**: Admin JWT
- **Request body**: —
- **Path param**: `event_id` — UUID
- **Response**:
  ```
  // 200
  { success: true; event: EventDefinitionAdmin; matches: MatchSchedule[] }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }
  ```

---

### PUT /admin/events/{event_id}

- **Auth**: Admin JWT
- **Request body**: `UpdateEventRequest`
- **Notes**: `gvg_koth` is rejected as a legacy category. `event_slug` cannot be changed once the event is live (`event_start <= now`).
- **Response**:
  ```
  // 200
  { success: true; event: EventDefinition }

  // 400 — legacy category, date validation, or slug locked
  { success: false; code: "LEGACY_CATEGORY" | "INVALID_DATES" | "LOCKED"; message: string }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }

  // 409 — slug conflict
  { success: false; code: "CONFLICT"; message: string }
  ```

---

### PATCH /admin/events/{event_id}/activate

- **Auth**: Admin JWT
- **Request body**: —
- **Response**:
  ```
  // 200
  { success: true; event: EventDefinition }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }
  ```

---

### PATCH /admin/events/{event_id}/deactivate

- **Auth**: Admin JWT
- **Request body**: —
- **Response**:
  ```
  // 200
  { success: true; event: EventDefinition }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }
  ```

---

### GET /admin/events/{event_id}/dashboard

- **Auth**: Admin JWT
- **Request body**: —
- **Response**:
  ```
  // 200
  {
    success: true;
    event: EventDefinition;
    registration_stats: RegistrationStats;
    match_stats: MatchStats;
    deduction_stats: DeductionStats;
  }
  ```

---

### POST /admin/events/{event_id}/matches

- **Auth**: Admin JWT
- **Request body**: `CreateMatchRequest` (single) or `CreateMatchesRequest` (bulk — detected by presence of `"matches"` key in the JSON body)
- **Notes**: Returns `400` for `level_cap_race` and `speed_run` events: `{ "status": "error", "message": "Matches are only supported for GvG, KOTH, and GvG+KOTH events" }`.
- **Response**:
  ```
  // 201 Created
  { success: true; created: number; matches: MatchSchedule[] }

  // 400 — wrong category
  { status: "error"; message: "Matches are only supported for GvG, KOTH, and GvG+KOTH events" }

  // 400 — invalid body
  { success: false; code: "VALIDATION_FAILED"; message: string }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }

  // 409 — match time overlap
  { success: false; code: "MATCH_OVERLAP"; message: string }

  // 422 — match locked
  { success: false; code: "MATCH_LOCKED"; message: string }
  ```

---

### POST /admin/events/{event_id}/matches/generate

- **Auth**: Admin JWT
- **Request body**: `GenerateMatchesRequest`
- **Notes**: Returns `400` for `level_cap_race` and `speed_run` events: `{ "status": "error", "message": "Matches are only supported for GvG, KOTH, and GvG+KOTH events" }`.
- **Response**:
  ```
  // 201 Created
  { success: true; created: number; matches: MatchSchedule[] }

  // 400 — wrong category
  { status: "error"; message: "Matches are only supported for GvG, KOTH, and GvG+KOTH events" }

  // 400 — date/validation errors
  { success: false; code: "OUT_OF_EVENT_RANGE" | "NO_MATCHES"; message: string }

  // 409 — overlap
  { success: false; code: "MATCH_OVERLAP"; message: string }
  ```

---

### GET /admin/events/{event_id}/matches

- **Auth**: Admin JWT
- **Request body**: —
- **Response**:
  ```
  { success: true; matches: MatchSchedule[] }
  ```

---

### PUT /admin/events/matches/{match_id}

- **Auth**: Admin JWT
- **Request body**: `UpdateMatchRequest`
- **Response**:
  ```
  // 200
  { success: true; match: MatchSchedule }

  // 400 — out of event range
  { success: false; code: "OUT_OF_EVENT_RANGE" | "OUT_OF_MATCH_RANGE"; message: string }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }

  // 422 — match locked (already tallied)
  { success: false; code: "MATCH_LOCKED"; message: string }
  ```

---

### DELETE /admin/events/matches/{match_id}

- **Auth**: Admin JWT
- **Request body**: —
- **Response**:
  ```
  // 200
  { success: true; message: "Match deleted" }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }

  // 422 — match locked
  { success: false; code: "LOCKED"; message: string }
  ```

---

### POST /admin/events/{event_id}/matches/{match_id}/retally

- **Auth**: Admin JWT
- **Request body**: —
- **Notes**: Resets the match tally state so the scheduler picks it up again. Only allowed when match status permits re-tally.
- **Response**:
  ```
  // 200
  { success: true; message: string; match: MatchSchedule }

  // 400 — status does not allow re-tally
  { success: false; code: "INVALID_STATUS"; message: string }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }
  ```

---

### PATCH /admin/events/matches/{match_id}/set-waiting

- **Auth**: Admin JWT
- **Request body**: —
- **Notes**: Manually transitions a match to `Waiting` so the tally scheduler picks it up. Only allowed from `Pending` or `Failed` status.
- **Response**:
  ```
  // 200
  { success: true; message: string; matchId: string }

  // 422 — invalid current status
  { success: false; code: "INVALID_STATUS"; message: string }
  ```

---

### PUT /admin/events/matches/{match_id}/tower-results

- **Auth**: Admin JWT
- **Request body**: `SetTowerResultsRequest`
- **Notes**: Saves which school (ChaSchool INT) won each tower and transitions the match to `Waiting` for tally.
- **Response**:
  ```
  // 200
  { success: true; message: string; match: MatchSchedule }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }

  // 422 — invalid match status
  { success: false; code: "INVALID_STATUS"; message: string }
  ```

---

### POST /admin/events/{event_id}/matches/{match_id}/gimmicks

- **Auth**: Admin JWT
- **Request body**: `CreateGimmickRequest`
- **Response**:
  ```
  // 201 Created
  { success: true; gimmick: EventMatchGimmick }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }
  ```

---

### GET /admin/events/{event_id}/matches/{match_id}/gimmicks

- **Auth**: Admin JWT
- **Request body**: —
- **Response**:
  ```
  { success: true; gimmicks: EventMatchGimmick[] }
  ```

---

### DELETE /admin/events/matches/{match_id}/gimmicks/{gimmick_id}

- **Auth**: Admin JWT
- **Request body**: —
- **Response**:
  ```
  // 200
  { success: true; message: "Gimmick deleted" }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }
  ```

---

### GET /admin/events/{event_id}/deductions

- **Auth**: Admin JWT
- **Request body**: —
- **Query params**: `target_type?: string` — filter by `"GUILD"`, `"PLAYER"`, or `"SCHOOL"`
- **Response**:
  ```
  { success: true; deductions: AdminDeduction[] }
  ```

---

### POST /admin/events/{event_id}/deductions

- **Auth**: Admin JWT
- **Request body**: `DeductionRequest`
- **Notes**: `target_id` required for `GUILD`/`PLAYER`; `target_cha_school` required for `SCHOOL`; `points` must be > 0; `reason` must be non-empty.
- **Response**:
  ```
  // 200
  { success: true; deduction: AdminDeduction }

  // 400 — validation failure
  { success: false; code: "VALIDATION_FAILED"; message: string }
  ```

---

### PUT /admin/events/{event_id}/deductions/{deduction_id}/revert

- **Auth**: Admin JWT
- **Request body**: —
- **Path params**: `event_id` UUID, `deduction_id` UUID
- **Response**:
  ```
  // 200
  { success: true; deduction: AdminDeduction }

  // 400 — already reverted
  { success: false; code: "ALREADY_REVERTED"; message: string }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }
  ```

---

### GET /admin/events/{event_id}/participants/characters

- **Auth**: Admin JWT
- **Request body**: —
- **Response**:
  ```
  {
    success: true;
    total: number;
    participants: Array<{
      registration_id: string | null;  // UUID string
      cha_num: number;
      cha_name: string;
      user_num: number;
      guild_num: number | null;
      guild_name: string | null;
      cha_school: number | null;
      cha_class: number;
      cha_level: number;
      status: number;
      registered_at: string | null;    // ISO NaiveDateTime string
      updated_at: string | null;
    }>;
  }
  ```

---

### GET /admin/events/{event_id}/participants/guilds

- **Auth**: Admin JWT
- **Request body**: —
- **Response**:
  ```
  {
    success: true;
    total: number;
    participants: Array<{
      guild_num: number;
      guild_name: string | null;
      cha_school: number | null;
      member_count: number;
      active_count: number;
      withdrawn_count: number;
      disqualified_count: number;
    }>;
  }
  ```

---

### PUT /admin/events/{event_id}/registrations/{cha_num}

- **Auth**: Admin JWT
- **Request body**: `AdminUpdateRegistrationRequest`
- **Notes**: At least one of `status` or `guild_num` must be provided. Send `guild_num: null` to clear the guild assignment.
- **Response**:
  ```
  // 200
  { success: true; message: "Registration updated"; registration: EventRegistration }

  // 400 — no fields provided or validation error
  { success: false; code: "VALIDATION_FAILED"; message: string }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }
  ```

---

### DELETE /admin/events/{event_id}/registrations/{cha_num}

- **Auth**: Admin JWT
- **Request body**: —
- **Notes**: Permanently removes the participant record regardless of ownership (no user check).
- **Response**:
  ```
  // 200
  { success: true; message: "Participant removed" }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }
  ```

---

### GET /admin/events/{event_id}/export/characters

- **Auth**: Admin JWT
- **Request body**: —
- **Response**: `text/csv` download with `Content-Disposition: attachment; filename="<EventName>_characters_<datetime>.csv"`
- **CSV columns**: `RegistrationID, ChaNum, ChaName, UserNum, GuildNum, GuildName, ChaSchool, ChaClass, ChaLevel, Status, RegisteredAt, UpdatedAt`

---

### GET /admin/events/{event_id}/export/guilds

- **Auth**: Admin JWT
- **Request body**: —
- **Response**: `text/csv` download with `Content-Disposition: attachment; filename="<EventName>_guilds_<datetime>.csv"`
- **CSV columns**: `GuildNum, GuildName, ChaSchool, MemberCount, ActiveCount, WithdrawnCount, DisqualifiedCount`

---

### GET /admin/events/{event_id}/export/matches/{match_id}/guild

- **Auth**: Admin JWT
- **Request body**: —
- **Response**: `text/csv` download with `Content-Disposition: attachment; filename="<EventName>_guild_match_results_<datetime>.csv"`
- **CSV columns**: `MatchID, MatchLabel, MatchDate, StartTime, EndTime, TallyStatus, RowsTallied, KillsCounted, KillsRejected, GuildNum, GuildName, School, MatchKills, MatchDeaths, MatchTower`

---

### GET /admin/events/{event_id}/export/matches/{match_id}/koth

- **Auth**: Admin JWT
- **Request body**: —
- **Response**: `text/csv` download with `Content-Disposition: attachment; filename="<EventName>_koth_match_results_<datetime>.csv"`
- **CSV columns**: `MatchID, MatchLabel, MatchDate, StartTime, EndTime, TallyStatus, ChaNum, ChaName, ChaClass, GuildNum, GuildName, MatchKills, MatchDeaths`

---
