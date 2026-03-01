# Shift Scheduler

A Rust workspace implementing a basic shift scheduling system with async job processing, recursive group resolution, and Redis caching.

---

# Overview

This project contains three crates:

- `data-service`  
  Provides staff and group management APIs, recursive group member resolution (via CTE), and Redis caching.

- `scheduling-service`  
  Handles schedule job creation and background processing for generating 28-day shift assignments.

- `shared`  
  Common enums and types shared between services.

Both services use:

- `sqlx` with compile-time query checking (`query!`, `query_as!`)
- SQLX offline mode via `.sqlx`
- PostgreSQL
- Redis

---

# Project Structure

Each service follows the same structure:

```
presentation/
application/
domain/
infrastructure/
```

This keeps HTTP handlers, business logic, and persistence code clearly separated without introducing heavy architectural patterns.

---

# Environment Configuration

The application selects the environment file based on the `APP_ENV` variable.

---

## .env.local (for `cargo run`)

```
APP_ENV=local
DATABASE_URL=postgres://postgres:postgres@localhost:5433/shift
REDIS_URL=redis://localhost:6379
DATA_SERVICE_URL=http://localhost:8080
```

---

## .env.test (for `cargo test`)

```
APP_ENV=test
DATABASE_URL=postgres://postgres:postgres@localhost:5433/shift
REDIS_URL=redis://localhost:6379
DATA_SERVICE_URL=http://localhost:8080
```

---

## .env.docker (for Docker Compose)

Inside Docker, service names must be used instead of `localhost`.

```
APP_ENV=docker
DATABASE_URL=postgres://postgres:postgres@postgres:5432/shift
REDIS_URL=redis://redis:6379
DATA_SERVICE_URL=http://data-service:8080
```

---

# Prerequisites

- Rust (stable toolchain)
- Docker + Docker Compose
- sqlx-cli

Install sqlx-cli:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

---

# SQLX & Migration Workflow (IMPORTANT)

This project uses:

```
sqlx::query!
sqlx::query_as!
```

These macros validate SQL at compile time.

Whenever you:

- change a query
- modify schema
- add a migration

You must run:

```bash
cargo sqlx migrate run --source data-service/migrations
cargo sqlx prepare --workspace -- --all-targets
```

Then commit the `.sqlx/` directory.

Correct order:

```
migrate → prepare → commit → docker build
```

Docker builds use:

```
ENV SQLX_OFFLINE=true
```

So `.sqlx` must be up to date before building containers.

---

# Running the Application

## Option 1 — Run with Docker (Recommended)

Docker Compose sets:

```
APP_ENV=docker
```

From repository root:

```bash
docker-compose up --build
```

This starts:

- PostgreSQL
- Redis
- data-service → http://localhost:8080
- scheduling-service → http://localhost:8081

Swagger:

- http://localhost:8080/swagger-ui
- http://localhost:8081/swagger-ui

To reset everything:

```bash
docker-compose down -v
```

---

## Option 2 — Run Locally (`cargo run`)

### 1. Start infrastructure only

```bash
docker-compose up postgres redis -d
```

### 2. Run migrations

```bash
cargo sqlx migrate run --source data-service/migrations
```

### 3. Generate SQLX cache (if schema/query changed)

```bash
cargo sqlx prepare --workspace -- --all-targets
```

### 4. Start services

PowerShell:

```bash
$env:APP_ENV="local"
cargo run -p data_service
```

New terminal:

```bash
$env:APP_ENV="local"
cargo run -p scheduling_service
```

Linux/macOS:

```bash
export APP_ENV=local
cargo run -p data_service
```

---

# Running Tests

Make sure Postgres and Redis are running locally.

PowerShell:

```bash
$env:APP_ENV="test"
cargo test --workspace
```

Linux/macOS:

```bash
export APP_ENV=test
cargo test --workspace
```

Tests automatically load `.env.test`.

---

# API Overview

## Data Service

Staff endpoints:

- POST `/api/v1/staff`
- GET `/api/v1/staff/{id}`
- PUT `/api/v1/staff/{id}`
- DELETE `/api/v1/staff/{id}`
- POST `/api/v1/staff/batch`

Group endpoints:

- POST `/api/v1/groups`
- PUT `/api/v1/groups/{id}`
- DELETE `/api/v1/groups/{id}`
- POST `/api/v1/groups/batch`
- POST `/api/v1/groups/{group_id}/members/{staff_id}`
- DELETE `/api/v1/groups/{group_id}/members/{staff_id}`
- GET `/api/v1/groups/{id}/resolved-members`

`resolved-members`:

- Uses recursive CTE
- Returns active staff from nested subgroups
- Cached in Redis (TTL 60 seconds)

---

## Scheduling Service

Endpoints:

- POST `/api/v1/schedules`
- GET `/api/v1/schedules/{id}/status`
- GET `/api/v1/schedules/{id}/result`

Example request:

```json
{
  "staff_group_id": "group-uuid",
  "period_begin_date": "2025-01-06"
}
```

`period_begin_date` must be a Monday.

---

# Async Job Processing

`POST /schedules`:

- Creates a job with status `PENDING`
- Returns `202 Accepted`

Worker loop:

1. Select one pending job (`FOR UPDATE SKIP LOCKED`)
2. Mark as `PROCESSING`
3. Fetch resolved members from data-service
4. Generate 28-day assignments
5. Persist assignments
6. Mark as `COMPLETED` or `FAILED`

Single-process, polling-based worker.

---

# Scheduling Rules

Located in:

```
scheduling-service/config.yaml
```

Config keys:

- `min_day_off_per_week`
- `max_day_off_per_week`
- `no_morning_after_evening`
- `max_daily_shift_diff`

Algorithm is heuristic and rule-driven (not globally optimized).

---

# Redis Caching (data-service)

Resolved members cache:

- Key: `group:{id}:resolved_members`
- TTL: 60 seconds
- Invalidated on:
  - group update
  - group delete
  - member add/remove

Cache operations are best-effort.

---

# Testing Coverage

Run:

```bash
cargo test --workspace
```

Includes:

- data-service integration tests
- scheduling-service API validation tests
- scheduling rule unit tests

---

# Known Limitations

- No authentication layer
- Single worker process
- No distributed job queue
- No metrics or tracing
- Heuristic scheduling only
- Basic error standardization
- File-based configuration

---

# Sample Data

Located under:

```
sample-data/
```

- `staff.json`
- `group.json`