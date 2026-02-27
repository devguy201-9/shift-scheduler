# Shift Scheduler – AIO Engineer Technical Assessment

A production-oriented **microservices-based shift scheduling system** built with Rust, following Clean Architecture principles.

This project implements the requirements described in the AIO Engineer Technical Assessment document.

---

## Architecture Overview

```
shift-scheduler/
├── Cargo.toml                  # Workspace root
├── docker-compose.yml
├── data-service/               # Staff & Group management
│   ├── Cargo.toml
│   └── src/
├── scheduling-service/         # Async schedule generation
│   ├── Cargo.toml
│   └── src/
├── shared/                     # Shared types & utilities
│   ├── Cargo.toml
│   └── src/
└── sample-data/
    ├── staff.json
    └── groups.json
```

## Tech Stack

- Rust (Edition 2021)
- Axum
- Tokio
- PostgreSQL
- SQLx (compile-time checked queries)
- Redis
- Serde / Serde JSON
- Utoipa (OpenAPI / Swagger)
- Docker & Docker Compose

---

# Data Service

Responsible for:

- Staff CRUD
- Staff Group CRUD (hierarchical)
- Group membership management
- Batch import from JSON
- Recursive resolved-member query
- Redis caching for read-heavy endpoints

## Entities

### Staff
- id
- name
- email (unique)
- position
- status (ACTIVE / INACTIVE)
- timestamps

### Staff Group
- id
- name
- parent_group_id (self reference)

### Group Membership
- staff_id
- group_id

## Key Endpoint

### Get Resolved Members (including nested groups)

```
GET /api/v1/groups/{id}/resolved-members
```

Returns all ACTIVE staff under a group, including members of nested subgroups.

---

# Scheduling Service

Responsible for asynchronous shift schedule generation.

## Flow

1. Client submits scheduling request
2. Job saved with status `PENDING`
3. Async worker processes job
4. Fetch resolved members from Data Service
5. Apply scheduling rules
6. Persist assignments
7. Mark job as `COMPLETED` or `FAILED`

---

## Shift Types

- MORNING
- EVENING
- DAY_OFF

---

## Scheduling Rules

| Rule | Config Key | Default |
|------|------------|----------|
| Minimum days off per week | `min_day_off_per_week` | 1 |
| Maximum days off per week | `max_day_off_per_week` | 2 |
| Disallow MORNING after EVENING | `no_morning_after_evening` | true |
| Maximum daily shift difference | `max_daily_shift_diff` | 1 |

All rules are configurable via configuration file.

---

## API

### Submit Schedule Generation

```
POST /api/v1/schedules
```

Request:

```json
{
  "staff_group_id": "group-123",
  "period_begin_date": "2025-05-19"
}
```

Response (202 Accepted):

```json
{
  "schedule_id": "job-abc123",
  "status": "PENDING"
}
```

---

### Check Job Status

```
GET /api/v1/schedules/{schedule_id}/status
```

Status values:
- PENDING
- PROCESSING
- COMPLETED
- FAILED

---

### Retrieve Generated Schedule

```
GET /api/v1/schedules/{schedule_id}/result
```

Response:

```json
{
  "schedule_id": "job-abc123",
  "period_begin_date": "2025-05-19",
  "staff_group_id": "group-123",
  "assignments": [
    { "staff_id": "staff-1", "date": "2025-05-19", "shift": "MORNING" }
  ]
}
```

---

# Clean Architecture

Each service is structured using layered architecture:

```
presentation/
application/
domain/
infrastructure/
```

- Domain layer contains pure business logic
- Application layer orchestrates use cases
- Infrastructure layer implements traits (DB, Redis, HTTP client)
- Presentation layer exposes HTTP APIs
- Dependency inversion achieved via Rust traits

---

# Testing

- Unit tests for scheduling rules and domain logic
- Integration tests for API endpoints
- Data Service dependencies mocked in Scheduling Service
- Code formatted with `cargo fmt`
- Lint clean with `cargo clippy -- -D warnings`

---

# Running the System

## Clone Repository

```bash
git clone https://github.com/devguy201-9/shift-scheduler.git
cd shift-scheduler
```

## Start Entire System

```bash
docker-compose up --build
```

This starts:

- data-service
- scheduling-service
- postgresql
- redis

The system boots with a single command as required.

---

# Swagger Documentation

After running:

Data Service:
```
http://localhost:8080/swagger-ui
```

Scheduling Service:
```
http://localhost:8081/swagger-ui
```

---

# Sample Data

Located in:

```
sample-data/
```

- staff.json
- groups.json

Used for batch import endpoints.

---

# Database Tables

## Data Service

- staff
- staff_groups
- group_memberships

## Scheduling Service

- schedule_jobs
- shift_assignments

Constraints include:
- Unique staff email
- Foreign keys
- Indexed job status
- Composite index (staff_id, date)

---

# Production-Oriented Considerations

- Compile-time verified SQL queries (SQLx)
- Async job processing
- Graceful error handling
- Redis caching for read-heavy endpoints
- Dockerized multi-service environment
- Configurable scheduling rules
- Clear separation of concerns
