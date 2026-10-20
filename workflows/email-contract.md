# Email Contract

> Generated from `ran-models/src/email/` and `ran-access-point/src/routes/email/`.
> Do not edit manually — re-run ContractGen to regenerate.

## Notes

There are no user-facing (non-admin) email route handlers. The directory
`ran-access-point/src/routes/email/` does not exist. All email endpoints are
admin-only and are documented in `workflows/email-admin-contract.md`.

The model module `ran-models/src/email/mod.rs` is confirmed re-exported from
`ran-models/src/lib.rs` as `pub mod email`.
