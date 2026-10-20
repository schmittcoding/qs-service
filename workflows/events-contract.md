# Events Contract

> Generated from `ran-models/src/events/` and `ran-access-point/src/routes/events/`.
> Do not edit manually — re-run ContractGen to regenerate.

## Notes

- All user-facing event routes are mounted under `/events`.
- Slug-scoped endpoints use the path pattern `/events/{event_slug}/...`.
- Auth for registration/withdrawal/guild-member endpoints requires a valid user session cookie (`get_user_session`).
- Public read endpoints (list, leaderboard, participants, snapshots, match-status, matches) require **no auth**.
- `CategoryConfig` is a discriminated union serialized with `"type"` as the discriminant tag (`#[serde(tag = "type", rename_all = "snake_case")]`). The five variants and their fields are documented below.
- `EventDefinition` no longer carries top-level `match_duration_min`, `tally_delay_min`, or `min_guild_members`. All category-specific configuration lives inside `category_config`.
- `tower_points_per_tick` has been removed entirely — do not send or expect it.
- `MatchWithGimmicks` uses `#[serde(flatten)]` on `match_schedule`, so all `MatchSchedule` fields appear at the top level alongside `gimmicks`.

---

## TypeScript Types

```typescript
// category.rs → EventCategory  (#[serde(rename_all = "snake_case")])
export type EventCategory =
  | "gvg"
  | "koth"
  | "gvg_koth"
  | "level_cap_race"
  | "speed_run";

// category_config.rs → GvgConfig
export type GvgConfig = {
  min_guild_members: number;
  match_duration_min: number;
  tally_delay_min: number;
  event_map_mid: number;
  event_map_sid: number;
};

// category_config.rs → KothConfig
export type KothConfig = {
  match_duration_min: number;
  tally_delay_min: number;
  event_map_mid: number;
  event_map_sid: number;
};

// category_config.rs → GvgKothConfig
export type GvgKothConfig = {
  min_guild_members: number;
  match_duration_min: number;
  tally_delay_min: number;
  event_map_mid: number;
  event_map_sid: number;
};

// category_config.rs → LevelCapRaceConfig
export type LevelCapRaceConfig = {
  max_account_age_days: number;
  target_level: number;
};

// category_config.rs → SpeedRunConfig
export type SpeedRunConfig = {
  dungeon_map_mid: number;
  dungeon_map_sid: number;
  boss_mob_id: number | null;
};

// category_config.rs → CategoryConfig  (#[serde(tag = "type", rename_all = "snake_case")])
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
  updated_at: string | null;  // ISO NaiveDateTime
  ip_address: string | null;
};

// registration.rs → RegisterRequest
export type RegisterRequest = {
  cha_num: number;
};

// registration.rs → GvgRegisterRequest
export type GvgRegisterRequest = {
  guild_num: number;
  member_cha_nums: number[];
};

// match_schedule.rs → MatchSchedule
export type MatchSchedule = {
  match_id: string;                   // UUID
  event_id: string;                   // UUID
  match_label: string | null;
  match_date: string;                 // ISO NaiveDate (YYYY-MM-DD)
  start_time: string;                 // ISO NaiveDateTime
  end_time: string;                   // ISO NaiveDateTime
  tally_scheduled: string;            // ISO NaiveDateTime
  tally_status: string;               // "Pending" | "Waiting" | "Processing" | "Completed" | "Failed"
  tally_started_at: string | null;    // ISO NaiveDateTime
  tally_completed_at: string | null;  // ISO NaiveDateTime
  rows_tallied: number | null;
  kills_counted: number | null;
  kills_rejected: number | null;
  tower_points_th: number;
  tower_points_nuc: number;
  tower_points_faci: number;
  tower_winner_th: number | null;     // ChaSchool INT of winning school; null = no winner
  tower_winner_nuc: number | null;
  tower_winner_faci: number | null;
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

// match_schedule.rs → MatchWithGimmicks  (#[serde(flatten)] on match_schedule)
// All MatchSchedule fields appear at the top level alongside gimmicks.
export type MatchWithGimmicks = MatchSchedule & {
  gimmicks: EventMatchGimmick[];
};

// snapshot.rs → EventSnapshot
export type EventSnapshot = {
  snapshot_id: string;      // UUID
  event_id: string;         // UUID
  match_id: string | null;  // UUID
  snapshot_time: string;    // ISO NaiveDateTime
  label: string | null;
  created_at: string;       // ISO NaiveDateTime
};

// snapshot.rs → SnapshotGuildScore
export type SnapshotGuildScore = {
  snapshot_id: string;    // UUID
  guild_num: number;
  guild_name: string;
  cha_school: number;
  total_kills: number;
  total_deaths: number;
  tower_points: number;
  deduction: number;
  computed_score: number;
  rank: number;
  rank_change: number | null;  // positive = moved up, negative = moved down, null = new entry
  member_count: number;
};

// snapshot.rs → SnapshotKOTHScore
export type SnapshotKOTHScore = {
  snapshot_id: string;        // UUID
  cha_num: number;
  cha_name: string;
  cha_class: number;
  cha_school: number | null;
  guild_num: number | null;
  guild_name: string | null;
  total_kills: number;
  total_deaths: number;
  deduction: number;
  rank: number;
  rank_change: number | null;
};

// snapshot.rs → SnapshotSchoolScore
export type SnapshotSchoolScore = {
  snapshot_id: string;  // UUID
  cha_school: number;
  total_kills: number;
  total_deaths: number;
  total_tower: number;
  deduction: number;
  computed_score: number;
  rank: number;
  rank_change: number | null;
};

// snapshot.rs → SnapshotLevelCapRaceScore
export type SnapshotLevelCapRaceScore = {
  snapshot_id: string;        // UUID
  cha_num: number;
  cha_name: string;
  cha_class: number;
  cha_school: number | null;
  guild_num: number | null;
  guild_name: string | null;
  start_level: number;
  current_level: number;
  levels_gained: number;
  rank: number;
  rank_change: number | null;
};

// guild_score.rs → GuildEventScore
export type GuildEventScore = {
  rank: number;
  event_id: string;   // UUID
  guild_num: number;
  guild_name: string;
  cha_school: number;
  total_kills: number;
  total_deaths: number;
  tower_points: number;
  deduction: number;
  computed_score: number;
};

// school_score.rs → SchoolEventScore
export type SchoolEventScore = {
  rank: number;
  event_id: string;   // UUID
  cha_school: number;
  total_kills: number;
  total_deaths: number;
  total_tower: number;
  deduction: number;
  computed_score: number;
};

// koth_score.rs → KOTHScore
export type KOTHScore = {
  rank: number;
  event_id: string;         // UUID
  cha_num: number;
  cha_name: string;
  cha_class: number;
  cha_school: number | null;
  guild_num: number | null;
  guild_name: string | null;
  total_kills: number;
  total_deaths: number;
  deduction: number;
};

// level_cap_race_score.rs → LevelCapRaceScore
export type LevelCapRaceScore = {
  rank: number;
  event_id: string;         // UUID
  cha_num: number;
  cha_name: string;
  cha_class: number;
  cha_school: number | null;
  guild_num: number | null;
  guild_name: string | null;
  start_level: number;
  current_level: number;
  levels_gained: number;
};

// speed_run_score.rs → SpeedRunScore
export type SpeedRunScore = {
  rank: number;
  event_id: string;         // UUID
  cha_num: number;
  cha_name: string;
  cha_class: number;
  cha_school: number | null;
  guild_num: number | null;
  guild_name: string | null;
  completion_time_seconds: number | null;
};

// Pagination envelope (inline — no named struct)
export type PaginationMeta = {
  page: number;
  page_size: number;
  total_items: number;
};
```

---

## API Contract

### GET /events/health

- **Auth**: None
- **Request body**: —
- **Response**:
  ```
  // 200 healthy
  { success: true; message: "Event system healthy" }

  // 500 unhealthy
  { success: false; message: "Event system unhealthy" }
  ```

---

### GET /events

- **Auth**: None
- **Request body**: —
- **Response**:
  ```
  { success: true; events: EventDefinition[] }
  ```

---

### GET /events/{event_slug}

- **Auth**: None
- **Request body**: —
- **Path param**: `event_slug` — string slug
- **Response**:
  ```
  // 200
  { success: true; event: EventDefinition; is_registration_available: boolean }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }
  ```

---

### GET /events/{event_slug}/characters

- **Auth**: User session (cookie)
- **Request body**: —
- **Notes**: Returns all characters owned by the authenticated user with an `eligible` flag computed per event category and level requirements.
- **Response**:
  ```
  // 200
  {
    success: true;
    event_id: string;            // UUID
    min_level: number;
    registration_open: string;   // ISO NaiveDateTime
    registration_close: string;  // ISO NaiveDateTime
    characters: Array<{
      cha_num: number;
      cha_name: string;
      cha_class: number;          // mapped event class (1–9), not raw game class
      cha_class_name: string;
      cha_school: number;
      cha_level: number;
      guild_num: number;
      guild_name: string | null;
      guild_leader_name: string | null;
      guild_leader_school: number | null;
      eligible: boolean;
    }>;
  }
  ```

---

### POST /events/{event_slug}/register

- **Auth**: User session (cookie)
- **Request body**: `RegisterRequest`
- **Response**:
  ```
  // 201 Created
  { success: true; message: string; registration: EventRegistration }

  // 400 — various validation failures
  { success: false; code: "CLOSED" | "LEVEL_REQ" | "NO_GUILD" | "ACCT_LIMIT" | "SCHOOL_MISMATCH" | "LEADER_SCHOOL_MISMATCH" | "VALIDATION_FAILED"; message: string }

  // 403
  { success: false; code: "UNAUTHORIZED"; message: string }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }

  // 409
  { success: false; code: "DUPLICATE"; message: string }
  ```

---

### POST /events/{event_slug}/register/guild

- **Auth**: User session (cookie)
- **Request body**: `GvgRegisterRequest`
- **Notes**: Caller must be the guild master of `guild_num`. Only valid for `gvg` and `gvg_koth` events.
- **Response**:
  ```
  // 201 Created
  { success: true; message: string; registrations: EventRegistration[] }

  // 400
  { success: false; code: "WRONG_CATEGORY" | "CLOSED" | "VALIDATION_FAILED"; message: string }

  // 403
  { success: false; code: "NOT_GUILD_MASTER"; message: string }

  // 409
  { success: false; code: "DUPLICATE"; message: string }
  ```

---

### GET /events/{event_slug}/registration-status

- **Auth**: User session (cookie)
- **Request body**: —
- **Response**:
  ```
  // 200
  { success: true; event_id: string; registrations: EventRegistration[] }
  ```

---

### DELETE /events/{event_slug}/register/{cha_num}

- **Auth**: User session (cookie)
- **Request body**: —
- **Path param**: `cha_num` — integer
- **Response**:
  ```
  // 200
  { success: true; message: "Registration withdrawn" }

  // 400 — no active registration
  { success: false; code: "NOT_FOUND"; message: string }

  // 403
  { success: false; code: "UNAUTHORIZED"; message: string }

  // 404
  { success: false; code: "NOT_FOUND"; message: string }
  ```

---

### GET /events/{event_slug}/guild-members

- **Auth**: User session (cookie)
- **Request body**: —
- **Notes**: Caller must be a guild master. Returns guild members eligible for this event with their `already_registered` flag.
- **Response**:
  ```
  // 200
  {
    success: true;
    event_id: string;   // UUID
    guild_num: number;
    min_level: number;
    members: Array<{
      cha_num: number;
      cha_name: string;
      cha_class: number;
      cha_school: number;
      cha_level: number;
      guild_name: string;
      already_registered: boolean;
    }>;
  }

  // 403
  { success: false; code: "NOT_GUILD_MASTER"; message: string }
  ```

---

### GET /events/{event_slug}/match-status

- **Auth**: None
- **Request body**: —
- **Response**:
  ```
  // match ongoing
  {
    success: true;
    match_ongoing: true;
    status: "LIVE" | "TALLYING_NOW" | "TALLYING_SOON";
    match_id: string;         // UUID
    match_label: string | null;
    start_time: string;       // ISO NaiveDateTime
    end_time: string;         // ISO NaiveDateTime
    tally_scheduled: string;  // ISO NaiveDateTime
    message: string;
  }

  // no active match
  { success: true; match_ongoing: false; status: "IDLE" }
  ```

---

### GET /events/{event_slug}/matches

- **Auth**: None
- **Request body**: —
- **Response**:
  ```
  { success: true; matches: MatchWithGimmicks[] }
  ```

---

### GET /events/{event_slug}/leaderboard

- **Auth**: None
- **Request body**: —
- **Query params**: `page?: number`, `page_size?: number` (default 20, max 100), `cha_class?: number`
- **Notes**: Dispatched by `event_category`. SpeedRun returns `501`.
- **Response by category**:
  ```
  // gvg
  { success: true; event_category: "gvg"; leaderboard_type: "guild"; pagination: PaginationMeta; rankings: GuildEventScore[] }

  // koth
  { success: true; event_category: "koth"; leaderboard_type: "character"; pagination: PaginationMeta; cha_class: number | null; rankings: KOTHScore[] }

  // level_cap_race
  { success: true; event_category: "level_cap_race"; leaderboard_type: "character"; pagination: PaginationMeta; rankings: LevelCapRaceScore[] }

  // gvg_koth
  {
    success: true;
    event_category: "gvg_koth";
    leaderboard_type: "combined";
    guild_rankings: GuildEventScore[];
    school_rankings: SchoolEventScore[];
    koth_rankings: KOTHScore[];
    pagination: { page: number; page_size: number; guild_total: number; koth_total: number };
  }

  // speed_run — 501
  { success: false; code: "NOT_IMPLEMENTED"; message: string }
  ```

---

### GET /events/{event_slug}/leaderboard/guild

- **Auth**: None
- **Request body**: —
- **Query params**: `page?: number`, `page_size?: number` (default 20, max 100)
- **Response**:
  ```
  { success: true; pagination: PaginationMeta; rankings: GuildEventScore[] }
  ```

---

### GET /events/{event_slug}/leaderboard/school

- **Auth**: None
- **Request body**: —
- **Response**:
  ```
  { success: true; rankings: SchoolEventScore[] }
  ```

---

### GET /events/{event_slug}/leaderboard/koth

- **Auth**: None
- **Request body**: —
- **Query params**: `page?: number`, `page_size?: number` (default 20, max 100), `cha_class?: number`
- **Response**:
  ```
  { success: true; pagination: PaginationMeta; cha_class: number | null; rankings: KOTHScore[] }
  ```

---

### GET /events/{event_slug}/participants/characters

- **Auth**: None
- **Request body**: —
- **Query params**: `page?: number`, `limit?: number` (default 20, max 100), `cha_class?: number`
- **Response**:
  ```
  {
    success: true;
    pagination: PaginationMeta;
    cha_class: number | null;
    participants: Array<{
      rank: number;
      cha_num: number;
      cha_name: string;
      cha_class: number;
      cha_school: number | null;
      cha_level: number;
      guild_num: number | null;
      guild_name: string | null;
    }>;
  }
  ```

---

### GET /events/{event_slug}/participants/guilds

- **Auth**: None
- **Request body**: —
- **Query params**: `page?: number`, `limit?: number` (default 20, max 100)
- **Response**:
  ```
  {
    success: true;
    pagination: PaginationMeta;
    participants: Array<{
      rank: number;
      guild_num: number;
      guild_name: string | null;
      cha_school: number | null;
      member_count: number;
    }>;
  }
  ```

---

### GET /events/{event_slug}/snapshots

- **Auth**: None
- **Request body**: —
- **Response**:
  ```
  { success: true; snapshots: EventSnapshot[] }
  ```

---

### GET /events/{event_slug}/snapshots/{snapshot_id}/guild

- **Auth**: None
- **Request body**: —
- **Path param**: `snapshot_id` — UUID string
- **Response**:
  ```
  { success: true; rankings: SnapshotGuildScore[] }
  ```

---

### GET /events/{event_slug}/snapshots/{snapshot_id}/koth

- **Auth**: None
- **Request body**: —
- **Path param**: `snapshot_id` — UUID string
- **Query params**: `cha_class?: number`
- **Response**:
  ```
  { success: true; cha_class: number | null; rankings: SnapshotKOTHScore[] }
  ```

---

### GET /events/{event_slug}/snapshots/{snapshot_id}/school

- **Auth**: None
- **Request body**: —
- **Path param**: `snapshot_id` — UUID string
- **Response**:
  ```
  { success: true; rankings: SnapshotSchoolScore[] }
  ```

---

### GET /events/{event_slug}/snapshots/{snapshot_id}/level-cap-race

- **Auth**: None
- **Request body**: —
- **Path param**: `snapshot_id` — UUID string
- **Response**:
  ```
  { success: true; rankings: SnapshotLevelCapRaceScore[] }
  ```

---
