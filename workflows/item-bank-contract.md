# Item Bank Contract

> Generated from `ran-models/src/admin/item_bank/` and `ran-access-point/src/routes/item_bank/`.
> Do not edit manually — re-run ContractGen to regenerate.

## Notes

There are no user-facing (non-admin) routes for the `item_bank` feature. The directory `ran-access-point/src/routes/item_bank/` does not exist. All item bank endpoints are registered exclusively under the admin route (`/admin/v{n}/item-bank/`) and require Admin JWT authentication.

See `workflows/item-bank-admin-contract.md` for all available endpoints and TypeScript types.
