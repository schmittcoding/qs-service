# Fix Session Expiry Bug

## Context

Users are being randomly logged out after ~30 minutes of activity, even when they are actively using the application. The cookie-based session (actix-session with CookieSessionStore) appears to expire too early or not renew correctly, causing unexpected 401 responses and forcing users back to the login screen.

## Agent Sequence

### 1. Debugger

**Goal:** Identify the root cause of premature session expiration in the actix-session cookie middleware.

**Prompt:**
You are debugging a session expiry bug in the gs-api Rust backend (actix-web 4, actix-session with CookieSessionStore).

**Symptom:** Users are randomly logged out after ~30 minutes of activity, even while actively making requests. No specific error logs have been observed yet.

**Key files to inspect:**
- `ran-access-point/src/main.rs` — `session_middleware()` function (lines 93–103). Pay close attention to:
  - `PersistentSession::default().session_ttl(cookie::time::Duration::days(1))` — this sets a 1-day TTL, but is the TTL being extended on each request (sliding window) or is it absolute from login?
  - `cookie_secure(true)` with `SameSite::Lax` — consider whether mixed HTTP/HTTPS traffic could cause the cookie to be dropped by the browser.
  - `cookie_domain(Some(...))` set from `WEB_DOMAIN` env var — mismatches here silently invalidate cookies on some requests.
- `ran-access-point/src/middleware/session/mod.rs` — `get_user_session()` — note it uses `println!` / `eprintln!` for errors (not `tracing`), so session failures are invisible in structured logs.
- `ran-access-point/src/routes/auth/mod.rs` — `authenticate()` calls `session.renew()` on login (correct), but the `validate` endpoint (GET `/validate`) reconstructs `expires_at` as `chrono::Local::now() + Duration::days(1)` client-side — this is cosmetic only and does not actually extend the real cookie TTL.

**Questions to answer through analysis:**
1. Does `actix-session` with `CookieSessionStore` and `PersistentSession` use a sliding TTL (extended on each request) or a fixed TTL from creation? If it is fixed, a user active for 30+ minutes would be logged out exactly at the 1-day mark — but the symptom is 30 minutes, which suggests either a different TTL is in effect or the cookie is being invalidated by something else.
2. Could `SESSION_SECRET` key rotation (key loaded from env var `SESSION_SECRET` at startup via `Key::from(secret.as_bytes())`) cause existing cookies to fail decryption after a server restart? This would explain "random" logouts — any deploy or restart invalidates all sessions.
3. Is there any middleware or handler that calls `session.clear()` or `session.purge()` outside of the explicit `POST /logout` handler?
4. Could `cookie_secure(true)` drop the cookie when requests come over HTTP (e.g., during local dev or a misconfigured reverse proxy)?

**Deliverable:** A root-cause diagnosis with the specific file, line, and mechanism responsible. List all contributing factors ranked by likelihood. Include a proposed fix for each.

**Handoff:** Pass the ranked list of root causes and proposed fixes to CodeGen. Specify exactly which files and lines need to change, and what the fix should be.

---

### 2. CodeGen

**Goal:** Implement the fixes identified by Debugger, following existing gs-api patterns exactly.

**Prompt:**
You are implementing fixes for the session expiry bug in gs-api. Apply only the changes specified in the Debugger's diagnosis. Do not refactor unrelated code.

**Context and patterns to follow:**
- `ran-access-point/src/main.rs` — `session_middleware()` is where session TTL and cookie settings are configured. Any changes to TTL behavior, sliding vs. fixed expiry, or cookie flags happen here.
- `ran-access-point/src/middleware/session/mod.rs` — `get_user_session()` currently uses `println!` and `eprintln!` which violate the project's logging standard (use `tracing::error!` / `tracing::debug!` instead). Fix this as part of this task since it's blocking visibility into session errors.
- `ran-access-point/src/routes/auth/mod.rs` — `validate` handler computes `expires_at` locally. If the Debugger identifies this as misleading or incorrect, update it to reflect the actual session TTL source of truth.

**Constraints:**
- Use `tracing::error!` and `tracing::debug!` — never `println!` / `eprintln!`.
- Do not introduce new abstractions or helpers beyond what is needed.
- Do not touch any handler that is not implicated in the bug.
- If the fix involves changing `session_ttl` or switching to a sliding-window strategy, apply it in `session_middleware()` in `main.rs` only.
- If the fix involves ensuring `SESSION_SECRET` stability across restarts, document the requirement in a code comment (the secret must be a stable, persisted value — not randomly generated each deploy).

**Deliverable:** The changed file contents (diffs or full updated functions). Do not write new files.

**Handoff:** Pass the full set of code changes to Tester and Reviewer.

---

### 3. Tester

**Goal:** Write or update tests that cover the fixed session expiry behavior.

**Prompt:**
You are writing tests for the session expiry fix in gs-api.

**Existing test file:** `ran-access-point/src/middleware/session/session_test.rs` — contains `test_get_user_session_found` and `test_get_user_session_not_found` using `actix_web::test`. Follow this exact pattern for new tests.

**Test cases to add:**
1. **Session survives multiple requests within TTL** — set a session, make several `GET /get` requests (simulating activity), confirm the session is still valid after N requests.
2. **Expired session returns 401** — if TTL was changed to a short value for testing, confirm that after TTL elapses the session is rejected with 401 and the body is `"No session data found"`.
3. **Session purge clears cookie** — call `POST /logout` (or simulate `session.purge()`), then verify a subsequent `GET /get` returns 401.
4. **Invalid or tampered cookie returns error, not panic** — pass a malformed cookie and assert the response is 401 or 500, never a panic.

**Constraints:**
- Use `actix_web::test` helpers (`init_service`, `TestRequest`, `call_service`, `read_body`).
- Keep tests in the same file: `session_test.rs`.
- Do not add new dependencies. Use only what is already in `Cargo.toml`.
- Tests that depend on time elapsing should use a very short TTL (e.g., `cookie::time::Duration::seconds(1)`) to keep them fast.

**Handoff:** Pass the new test code to Reviewer.

---

### 4. Reviewer

**Goal:** Verify the fix is correct, complete, and does not introduce regressions or security issues.

**Prompt:**
You are reviewing the session expiry bug fix for gs-api. Review the changes produced by CodeGen and the tests produced by Tester.

**Review checklist:**

Security:
- [ ] `cookie_secure(true)` is preserved — never weakened.
- [ ] `cookie_http_only(true)` is preserved.
- [ ] `cookie_content_security(CookieContentSecurity::Private)` is preserved (encryption, not just signing).
- [ ] `SESSION_SECRET` is not hardcoded anywhere. Confirm it still comes from the `SESSION_SECRET` env var.
- [ ] `session.renew()` is still called on login in `authenticate()` to prevent session fixation.

Correctness:
- [ ] The TTL behavior (sliding vs. fixed) is explicitly documented in a comment in `session_middleware()`.
- [ ] The `expires_at` value returned by `GET /validate` accurately reflects the actual session TTL, not a hardcoded client-side guess.
- [ ] `get_user_session()` uses `tracing::error!` / `tracing::debug!` — not `println!` / `eprintln!`.
- [ ] `session.purge()` in `POST /logout` is still the only place sessions are explicitly destroyed.

Pattern consistency:
- [ ] No new error types, traits, or helper functions were introduced that aren't needed.
- [ ] All logging uses `tracing` macros.
- [ ] Handler signatures and return types follow the existing pattern in `ran-access-point/src/routes/`.

Tests:
- [ ] All four test cases specified in Tester's prompt are present.
- [ ] Tests use `actix_web::test` helpers only.
- [ ] No `unwrap()` calls in non-test code introduced by this change.

**Deliverable:** An approval with any blocking issues noted, or a list of required changes before this fix can be merged.
