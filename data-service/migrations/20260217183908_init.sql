CREATE TYPE staff_status AS ENUM ('ACTIVE', 'INACTIVE');


CREATE TABLE staff
(
    id         UUID PRIMARY KEY,
    name       TEXT         NOT NULL,
    email      TEXT UNIQUE  NOT NULL,
    position   TEXT         NOT NULL,
    status     staff_status NOT NULL,
    created_at TIMESTAMP    NOT NULL DEFAULT now(),
    updated_at TIMESTAMP    NOT NULL DEFAULT now()
);
CREATE INDEX idx_staff_status ON staff (status);

CREATE TABLE staff_groups
(
    id              UUID PRIMARY KEY,
    name            TEXT      NOT NULL,
    parent_group_id UUID      REFERENCES staff_groups (id) ON DELETE SET NULL,
    created_at      TIMESTAMP NOT NULL DEFAULT now(),
    updated_at      TIMESTAMP NOT NULL DEFAULT now()
);
CREATE INDEX idx_staff_groups_parent ON staff_groups (parent_group_id);

CREATE TABLE group_memberships
(
    staff_id UUID REFERENCES staff (id) ON DELETE CASCADE,
    group_id UUID REFERENCES staff_groups (id) ON DELETE CASCADE,
    PRIMARY KEY (staff_id, group_id)
);
CREATE INDEX idx_group_memberships_group ON group_memberships (group_id);
CREATE INDEX idx_group_memberships_staff ON group_memberships (staff_id);

CREATE TYPE job_status AS ENUM ('PENDING', 'PROCESSING', 'COMPLETED', 'FAILED');
CREATE TABLE schedule_jobs
(
    id                UUID PRIMARY KEY,
    staff_group_id    UUID       NOT NULL REFERENCES staff_groups (id),
    period_begin_date DATE       NOT NULL,
    status            job_status NOT NULL,
    error_message     TEXT,
    created_at        TIMESTAMP  NOT NULL DEFAULT now(),
    updated_at        TIMESTAMP  NOT NULL DEFAULT now()
);
CREATE INDEX idx_schedule_jobs_status_created_at
    ON schedule_jobs (status, created_at);
CREATE INDEX idx_schedule_jobs_group
    ON schedule_jobs(staff_group_id);

CREATE TYPE shift_type_enum AS ENUM ('MORNING', 'EVENING', 'DAY_OFF');
CREATE TABLE shift_assignments
(
    id          UUID PRIMARY KEY,
    schedule_id UUID REFERENCES schedule_jobs (id) ON DELETE CASCADE,
    staff_id    UUID            NOT NULL REFERENCES staff (id),
    date        DATE            NOT NULL,
    shift_type  shift_type_enum NOT NULL,
    UNIQUE (schedule_id, staff_id, date)
);
CREATE INDEX idx_shift_assignments_schedule ON shift_assignments (schedule_id);
CREATE INDEX idx_shift_assignments_staff ON shift_assignments (staff_id);
