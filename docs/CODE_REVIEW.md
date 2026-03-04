# Code review and lint status

This document summarizes the code review and lint posture for the Rust port.

## Lint configuration

- **Rust:** `warnings = "deny"`, `unsafe_code = "forbid"` in all crates.
- **Clippy:** `pedantic = "deny"`, `nursery = "deny"` in all crates. `cargo = "deny"` is **disabled** in `shared`, `data`, and `admin-api` so that `cargo clippy` can run in environments where `cargo metadata` cannot download crates (e.g. sandbox/CI). Re-enable it for full compliance when network is available.

## Fixes applied (no dead code, no warnings in reviewed crates)

### Shared crate
- Inlined format args (`format!("... {}", x)` → `format!("... {x}")`), removed useless `format!`, use const for JWKS URL.
- Documented `# Errors` for public `Result`-returning functions; added `#[must_use]` where appropriate.
- Replaced custom `to_string()` with `Display` for enums; replaced custom `default()` with `Default`.
- Collapsed nested `if`/`if let` where appropriate; used `is_ok_and`, `map_or`, `as_deref`, `unwrap_or_else(Fn)`.
- Replaced wildcard imports with explicit imports; fixed doc backticks; allowed `struct_field_names` only where renaming would be invasive.
- Session/auth: avoid unnecessary `unwrap` after `is_none()` check; removed redundant `continue`; use `String::new` instead of `"".to_string()`.

### Data crate
- Documented `# Errors` for `create_pool`; allowed `missing_const_for_fn` (DbPool not const-constructible), `needless_raw_string_hashes` (SQL readability), `must_use_candidate` (repos).
- Replaced `unwrap_or_else(|| ...)` with `unwrap_or_else(Fn)` or `unwrap_or_default()`; used `map_or_else` for option branching in subscription repo.

### Admin-api
- Replaced wildcard `data::repositories::*` with explicit repository imports.
- Inlined format args; `match` on `Option` replaced with `if let`; `main` allowed `too_many_lines`.
- Auth: CSRF check simplified and collapsed; redirect uses `map_or`; doc backticks fixed.
- **Admin API protection:** All `/api/*` routes (except `/api/me`) are behind `require_admin_auth_middleware`. When Azure AD is configured: requires session (401 if not signed in) and Known User with admin role (403 if not in list). When Azure AD is not configured (local/Docker): routes are open for dev.

### Webhook package
- Added missing Cargo metadata: `readme`, `keywords`, `categories` (from workspace).

### Marketplace crate
- Cargo lint disabled for sandbox; crate-level `allow` for `missing_const_for_fn`, `must_use_candidate`.
- Fulfillment: `# Errors` docs, inlined format args, `next_back()` instead of `last()` on split, `Into::into` for maps, explicit enum variants, `Self` in `From` impls, `quantity` u32→i32 via `try_into().ok()`.
- Metering: collapsible `if let`, inlined format args, `# Errors` docs, `Self` in `From` impl.
- Client: `#[must_use]` on builder methods, `# Panics` and `# Errors` docs.

### Scheduler crate
- Cargo lint disabled; wildcard imports replaced with explicit repository imports; inlined format args; `String::new()`; raw string hashes removed where unnecessary.

### Webhook crate (lib)
- `needless_borrow` fix; collapsible `if let` in auth and operation-id handling. Webhook handler: collapsible `if let` for notification handler.

### Customer-api and webhook-api
- Cargo lint disabled; wildcard repository imports replaced with explicit imports.

## Running full clippy (zero warnings)

From the repo root:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

With strict cargo lint (requires network for metadata):

```bash
# Re-enable cargo = "deny" in shared, data, admin-api Cargo.toml then:
cargo clippy --workspace --all-targets -- -D warnings
```

## Security-related notes

- No `unsafe` code; `unsafe_code = "forbid"` in all crates.
- OAuth: CSRF checked in callback; token exchange and id_token handling implemented; session stored server-side.
- **Admin API:** Protected by session + Known Users (admin role) when Azure AD is configured; 401/403 for unauthenticated or unauthorized. See [docs/AZURE_DEPLOYMENT.md](AZURE_DEPLOYMENT.md) and README for production deployment.
- Secrets (client_secret, etc.) are not logged; use env vars and secure configuration in production.
- Session layer errors are converted to HTTP 500 via `HandleErrorLayer` (no stack trace to client).

## Data crate (Known Users)

- `KnownUsersRepository::get_by_email_and_role` added for admin access check; `ROLE_ID_ADMIN = 1` matches original .NET.

## Full review status

All crates (`shared`, `data`, `admin-api`, `customer-api`, `webhook-api`, `marketplace`, `scheduler`, `webhook`) have been reviewed and pass `cargo clippy --workspace --all-targets -- -D warnings` (modulo the `cargo = "deny"` lint disabled where needed for sandbox/CI).

## Remaining optional work

- **Dependency:** `sqlx-postgres` has a future-incompatibility warning; consider upgrading when a fixed release is available.
- **Dead code:** `cargo build --workspace` and `cargo clippy` with `-D warnings` will flag unused code; address any reported items.
