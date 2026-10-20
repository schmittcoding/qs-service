# Email Verification — Frontend Contract

> Generated from `ran-models/src/users/`, `ran-access-point/src/routes/auth/`, and `ran-access-point/src/services/rate_limit.rs`.

## TypeScript Types

```typescript
// ProfileData now includes is_verified
export type ProfileData = {
  user_name: string;
  user_num: number;
  user_email: string;
  user_available: number;
  web_points: number;
  mileage_points: number;
  created_at: string; // ISO 8601
  last_updated: string; // ISO 8601
  is_verified: boolean;
  user_role: "admin" | "player";
};

// GET /api/v1/auth/verify-email query params
export type VerifyEmailParams = {
  token: string;
};

// GET /api/v1/auth/verify-email response
export type VerifyEmailResponse = {
  verified: boolean;
  message: string;
};
```

## API Contract

### GET /api/v1/auth/verify-email?token=\<jwt\>

- **Auth**: None (public — called from email link)
- **Query params**: `token` — JWT verification token
- **Response**:
  - `200`: `{ verified: true, message: "Email verified successfully" }`
  - `400`: `{ verified: false, message: "Verification link has expired. Please request a new one." }` or `{ verified: false, message: "Invalid verification link." }` or `{ verified: false, message: "Token does not match..." }`
  - `500`: `{ verified: false, message: "An error occurred during verification." }`

---

### POST /api/v1/auth/resend-verification

- **Auth**: User session (cookie)
- **Request body**: — (none)
- **Rate limiting**:
  - **IP-based**: 1 request per 30 seconds, burst of 3 (actix-governor). Returns `429` with no body on exceed.
  - **Per-user DB check**: max 3 verification emails within a rolling 60-minute window. Returns `429` with `{ message: "Too many verification email requests. Please try again later." }`.
- **Response**:
  - `200`: `{ message: "Verification email sent" }`
  - `400`: `{ message: "Email is already verified" }` (if `is_verified` is already `true`)
  - `429`: Rate limit exceeded (see above)
  - `401`: Not authenticated
  - `500`: `{ message: "Failed to send verification email" }`

---

### POST /api/v1/auth/change-email

- **Auth**: User session (cookie)
- **Request body**: `{ email: string, pincode: string }`
- **Behavior change**: After a successful email change, `is_verified` resets to `false` and a new verification email is automatically sent to the new address. The session is updated immediately — subsequent `/validate` calls will reflect `is_verified: false`.
- **Response**: `{ message: "Email updated successfully" }`

---

### POST /api/v1/auth/register

- **Auth**: None
- **Request body**: `RegistrationRequest`
- **Behavior change**: A verification email is automatically sent after successful registration. The new account starts with `is_verified: false`.
- **Response**: `{ message: "User has been registered successfully" }`

---

### GET /api/v1/auth/validate

- **Auth**: User session (cookie)
- **Response**: `{ user: ProfileData, expires_at: string }` — `ProfileData` now includes `is_verified: boolean`.

---

### POST /api/v1/auth/authenticate

- **Auth**: None
- **Request body**: `AuthRequest`
- **Response**: `ProfileData` — now includes `is_verified: boolean`.

---

## `is_verified` Field

Available in every response that returns `ProfileData`:
- `POST /authenticate` response body
- `GET /validate` response body under `user`
- Session state after login

The field is `boolean`. `false` for new accounts and after email changes until the user clicks the verification link.

---

## Frontend Implementation Guide

### 1. Verified Badge

Display a verified badge/indicator next to the username wherever the profile is shown. Read `is_verified` from the auth store (populated by `/authenticate` or `/validate`).

```
if user.is_verified → show verified icon/badge
if !user.is_verified → show unverified state (or hide badge)
```

---

### 2. Verification Snackbar (Unverified Accounts)

For logged-in users with `is_verified === false`, show a **persistent floating snackbar** across the portal:

- **Position**: Bottom of the viewport (or top, depending on design system)
- **Content**: "Please verify your email address to unlock all features."
- **Action button**: "Verify Now" — opens the verification dialog (see below)
- **Dismissible**: No — should persist across page navigation while unverified. Can be minimized but should remain visible.
- **Condition**: Only show when `user.is_verified === false`. Hide immediately when the user verifies (re-check via `/validate` after verification).

---

### 3. Resend Verification Dialog

Triggered by the snackbar "Verify Now" button:

1. Show a dialog/modal with:
   - Message: "A verification link will be sent to your registered email address: **{user_email}**"
   - "Send Verification Email" button
   - "Cancel" button

2. On "Send Verification Email" click:
   - Call `POST /api/v1/auth/resend-verification`
   - On `200`: Show success toast, start the 60-second cooldown timer
   - On `400` (already verified): Refresh user state via `/validate`, close dialog, hide snackbar
   - On `429`: Show "Please wait before requesting another email" message
   - On `500`: Show error toast

3. **60-second cooldown timer** (abuse prevention):
   - After a successful resend, disable the "Send Verification Email" button for 60 seconds
   - Show a countdown: "Resend available in {seconds}s"
   - **Must be persistent**: store the cooldown expiry timestamp in `localStorage` so refreshing the page does not reset it

   ```typescript
   // On successful resend:
   const expiresAt = Date.now() + 60_000;
   localStorage.setItem("verification_cooldown", expiresAt.toString());

   // On component mount / dialog open:
   const stored = localStorage.getItem("verification_cooldown");
   if (stored && Date.now() < Number(stored)) {
     const remaining = Math.ceil((Number(stored) - Date.now()) / 1000);
     // Start countdown from `remaining` seconds
   }
   ```

   - Note: The server also enforces rate limits (IP-based + per-user DB check), so even if the client-side timer is bypassed, the server will reject excessive requests.

---

### 4. Email Verification Redirect Page

Route: `/verify-email?token=<jwt>`

This is the page users land on after clicking the verification link in their email.

1. On page load, extract `token` from query params
2. Call `GET /api/v1/auth/verify-email?token={token}`
3. Display result based on response:

| `verified` | UI State |
|---|---|
| `true` | Success state: "Your email has been verified!" with a link/button to go to the dashboard or login page |
| `false` + expired message | Expired state: "Your verification link has expired." with a "Resend Verification Email" button (if logged in) or "Please log in to request a new link" |
| `false` + other message | Error state: Show the `message` from the response |

4. If the user is currently logged in, call `/validate` after successful verification to refresh the session state (this updates `is_verified` in the auth store and hides the snackbar).

---

### 5. Recharge Flow — Block Unverified Accounts

Before allowing a user to access the recharge/payment flow, check `is_verified`:

```typescript
if (!user.is_verified) {
  // Block access to recharge
  // Show message: "Please verify your email address before making a purchase."
  // Optionally include a "Verify Now" button that opens the verification dialog
  return;
}
// Proceed with normal recharge flow
```

Where to enforce:
- **Recharge page/component**: Guard at the top level — redirect or show a blocking overlay
- **"Recharge" navigation item or button**: Optionally show a tooltip "Email verification required" on hover for unverified users

---

## Rate Limit Summary

| Layer | Scope | Limit | Applies to |
|---|---|---|---|
| actix-governor (IP) | Per IP address | 1 req/30s, burst 3 | `POST /resend-verification` |
| DB check (per-user) | Per user_num | 3 emails per 60 minutes | `POST /resend-verification` |
| Frontend cooldown | Per browser | 60 seconds between clicks | "Send Verification Email" button |

The frontend 60-second cooldown is a UX guard only. The server enforces the real limits independently.

---

## State Management Checklist

- [ ] Add `is_verified: boolean` to the auth store's `ProfileData` type
- [ ] Read `is_verified` from `/authenticate` and `/validate` responses
- [ ] Conditionally render verified badge based on `is_verified`
- [ ] Show persistent snackbar when `is_verified === false`
- [ ] Implement resend dialog with `localStorage`-backed cooldown timer
- [ ] Create `/verify-email` route with token handling
- [ ] Guard recharge flow with `is_verified` check
- [ ] Refresh auth state after successful verification (call `/validate`)
