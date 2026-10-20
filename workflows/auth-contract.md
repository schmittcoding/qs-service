# Auth Contract

> Generated from `ran-models/src/users/` and `ran-access-point/src/routes/auth/`.
> Do not edit manually — re-run ContractGen to regenerate.

## TypeScript Types

```typescript
// mod.rs → RegistrationRequest
export type RegistrationRequest = {
  full_name: string;
  username: string;
  password: string;
  pincode: string;
  email: string;
};

// mod.rs → AuthRequest
export type AuthRequest = {
  username: string;
  password: string;
};

// mod.rs → ProfileData
// Note: fields come from ProfileData.to_json() — user_type is omitted and user_role is added.
export type ProfileData = {
  user_name: string;
  user_num: number;
  user_email: string;
  user_available: number;
  web_points: number;
  mileage_points: number;
  created_at: string; // ISO 8601
  last_updated: string; // ISO 8601
  user_role: string; // "admin" | "player"
};

// mod.rs → ForgotPasswordRequest
export type ForgotPasswordRequest = {
  username: string;
  email: string;
};

// mod.rs → ChangeEmailRequest
export type ChangeEmailRequest = {
  email: string;
  pincode: string;
};

// mod.rs → ChangePasswordRequest
export type ChangePasswordRequest = {
  old_password: string;
  new_password: string;
  pincode: string;
};

// mod.rs → ChangePincodeRequest
export type ChangePincodeRequest = {
  old_pincode: string;
  new_pincode: string;
};

// mod.rs → ValidateUsernameRequest
export type ValidateUsernameRequest = {
  username: string;
};

// mod.rs → ValidateEmailRequest
export type ValidateEmailRequest = {
  email: string;
};
```

## API Contract

### POST /api/v1/auth/register

- **Auth**: None
- **Request body**: `RegistrationRequest`
- **Response**: `{ message: string }`

---

### POST /api/v1/auth/authenticate

- **Auth**: None
- **Request body**: `AuthRequest`
- **Response**: `ProfileData` (serialized via `to_json()` — see type definition above; sets a session cookie on success)

---

### GET /api/v1/auth/validate

- **Auth**: User session (cookie)
- **Request body**: —
- **Response**: `{ user: ProfileData, expires_at: string }` — `expires_at` is an RFC 3339 timestamp 24 hours from request time

---

### POST /api/v1/auth/logout

- **Auth**: User session (cookie)
- **Request body**: —
- **Response**: Empty 200 (session cookie is cleared)

---

### POST /api/v1/auth/forgot-password

- **Auth**: None
- **Request body**: `ForgotPasswordRequest`
- **Response**: `{ message: string }` — always returns the same message regardless of whether the account exists

---

### POST /api/v1/auth/change-email

- **Auth**: User session (cookie)
- **Request body**: `ChangeEmailRequest`
- **Response**: `{ message: string }`

---

### POST /api/v1/auth/change-password

- **Auth**: User session (cookie)
- **Request body**: `ChangePasswordRequest`
- **Response**: `{ message: string }`

---

### POST /api/v1/auth/change-pincode

- **Auth**: User session (cookie)
- **Request body**: `ChangePincodeRequest`
- **Response**: `{ message: string }`

---

### POST /api/v1/auth/reset-pincode

- **Auth**: User session (cookie)
- **Request body**: —
- **Response**: `{ message: string }` — a new pincode is generated and sent to the user's registered email

---

### POST /api/v1/auth/validate-username

- **Auth**: None
- **Request body**: `ValidateUsernameRequest`
- **Response**: `boolean` — `true` if the username is available, `false` if already taken

---

### POST /api/v1/auth/validate-email

- **Auth**: None
- **Request body**: `ValidateEmailRequest`
- **Response**: `boolean` — `true` if the email is available, `false` if already taken

---
