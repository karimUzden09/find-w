# Integration Tests Guide

This project uses integration tests that validate the full request flow:

1. HTTP route matching in `axum`
2. request extractors (including JWT auth)
3. handlers and response mapping
4. SQL queries against real Postgres

The only excluded API area is `notes` (as requested).

## Where tests live

- `tests/common/mod.rs`: shared test harness (`TestApp`)
- `tests/core_api.rs`: health, db-health, me, docs/openapi
- `tests/auth_api.rs`: register/login/refresh/logout
- `tests/groups_api.rs`: create/list/delete groups
- `tests/vk_tokens_api.rs`: add/delete VK tokens and storage checks

## How tests run

### 1) Isolated DB per test (`#[sqlx::test]`)

Each test function is annotated with `#[sqlx::test]` and receives `PgPool`:

```rust
#[sqlx::test]
async fn some_test(pool: PgPool) { ... }
```

`sqlx` creates an isolated temporary database, applies migrations, and provides a pool for the test.
This keeps tests independent and avoids polluting your main application database.

### 2) In-process HTTP calls (`Router::oneshot`)

`TestApp` builds the real router using `build_router(state)` and sends requests directly into it:

```rust
let response = app.clone().oneshot(request).await?;
```

No TCP port is opened. This is still a real HTTP request path through:

1. router
2. middleware/extractors
3. handler
4. DB

## Prerequisites

1. Running Postgres
2. `DATABASE_URL` points to Postgres
3. The DB user must be able to create/drop temporary test databases

Example:

```bash
export DATABASE_URL=postgres://app:app@localhost:5432/app
```

## Running tests

Run all integration tests:

```bash
cargo test --tests
```

Run one test file:

```bash
cargo test --test auth_api
```

Run one test function:

```bash
cargo test --test auth_api refresh_rotates_tokens_and_old_refresh_becomes_invalid
```

## Test harness (`tests/common/mod.rs`)

`TestApp` provides helpers:

1. `post_json(path, body, bearer)` -> `(StatusCode, Value)`
2. `get_json(path, bearer)` -> `(StatusCode, Value)`
3. `get_text(path, bearer)` -> `(StatusCode, String)`
4. `delete_json(path, body, bearer)` -> `(StatusCode, Value)`
5. `delete(path, bearer)` -> `StatusCode`
6. `register_and_login()` -> test user with `access_token` and `refresh_token`

This keeps tests short and consistent.

## What exactly is covered

### Core

1. `GET /health` returns `ok`
2. `GET /db-health` returns `ok`
3. `GET /me` requires auth
4. docs endpoints respond (`/docs`, `/api-docs/openapi.json`)

### Auth

1. register + login + `/me` happy path
2. duplicate email conflict
3. wrong password unauthorized
4. refresh rotation invalidates old refresh token
5. logout revokes refresh token

### Groups

1. create/list/delete group
2. ownership isolation between users
3. payload validation and auth requirement

### VK Tokens

1. add/delete token flow
2. dedup behavior (`inserted`/`skipped`)
3. encrypted storage check (`token_encrypted` is not plaintext)
4. internal repo read/decrypt method check
5. payload validation and auth requirement

## How to add a new test case (tutorial)

Example: add a negative case for `POST /groups` with invalid payload.

### Step 1: choose file

Open `tests/groups_api.rs`.

### Step 2: create a test with isolated DB

```rust
#[sqlx::test]
async fn groups_create_rejects_invalid_payload(pool: PgPool) { ... }
```

### Step 3: build app and auth user

```rust
let app = TestApp::new(pool);
let user = app.register_and_login().await;
```

### Step 4: send request

```rust
let (status, body) = app
    .post_json("/groups", json!({ "group_id": 0 }), Some(&user.access_token))
    .await;
```

### Step 5: assert behavior

```rust
assert_eq!(status, StatusCode::BAD_REQUEST);
assert_eq!(body.get("error").and_then(Value::as_str), Some("BAD_REQUEST"));
```

### Step 6: run only this file first

```bash
cargo test --test groups_api
```

### Step 7: run whole integration suite

```bash
cargo test --tests
```

## Recommended style for new tests

1. Test one behavior per function
2. Keep names explicit (`what_happens_when_condition`)
3. Use harness helpers instead of repeating request boilerplate
4. Assert both status and key response fields
5. For security-sensitive logic, add direct DB assertion when needed

## Common troubleshooting

1. `error communicating with database`
   Ensure Postgres is running and `DATABASE_URL` is valid.
2. permission errors creating test DB
   Grant DB user `CREATEDB`.
3. SQLx macro mismatch after schema changes
   Run migrations before tests:

```bash
sqlx migrate run
```
