# Planning/Mgmt System for Sports Clubs - [WIP]

In short: manage teams, plan games, etc. \
Frontend: pending \
Backend - Rust (axum, sqlx) \
DB: Postgres

## Development

### IMPORTANT WIP NOTES

- For now the DB schema is WIP, all migrations may be completely rolled back and replaced
- Not all basic security features are implemented

### Dev Prerequisites

- Rust
- Docker
- SQLX (for Rust)

### Start Dev Server

- start Postgres service
  - `cd pg`
  - `docker compose up`
- migrate DB: `sqlx migrate run`
- start backend _with initial values_ \
  `INITIAL_CLUB=super_club INITIAL_USER='super_user' INITIAL_PASSWORD='dev_password93837&§!' RUST_LOG=debug,axum::rejection=trace  RUST_BACKTRACE=1 cargo watch -w src -x run`
  - (initial values only needed on first start, or after a DB-reset)

### API-Testing

- ensure your initial values are under `test/.env`
- ensure, your server is running - these tests require it
- `cd test`
- `npm run test`
