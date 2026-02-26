use crate::application::traits::ScheduleRepository;
use crate::domain::schedule::{ScheduleJob, ShiftAssignment};
use axum::async_trait;
use chrono::NaiveDate;
use shared::types::{JobStatus, ShiftType};
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
use uuid::Uuid;

pub struct ScheduleRepositoryPg {
    pool: PgPool,
}

impl ScheduleRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Update status
    async fn update_status(
        &self,
        id: Uuid,
        status: JobStatus,
        error: Option<&str>,
    ) -> anyhow::Result<()> {
        let result = sqlx::query!(
            r#"
            UPDATE schedule_jobs
            SET status = $2,
                error_message = $3,
                updated_at = now()
            WHERE id = $1
            "#,
            id,
            status as JobStatus,
            error
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            anyhow::bail!("Job not found");
        }

        Ok(())
    }
}

#[async_trait]
impl ScheduleRepository for ScheduleRepositoryPg {
    // Create JOB
    async fn insert_job(
        &self,
        id: Uuid,
        staff_group_id: Uuid,
        period_begin_date: NaiveDate,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO schedule_jobs
            (id, staff_group_id, period_begin_date, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, now(), now())
            "#,
            id,
            staff_group_id,
            period_begin_date,
            JobStatus::Pending as JobStatus,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Fetch Pending JOB
    async fn fetch_pending(&self) -> anyhow::Result<Option<ScheduleJob>> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query!(
            r#"
            SELECT id, staff_group_id, period_begin_date, status as "status: JobStatus",
                   error_message, created_at, updated_at
            FROM schedule_jobs
            WHERE status = 'PENDING'
            ORDER BY created_at
            FOR UPDATE SKIP LOCKED
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        let job = if let Some(r) = row {
            // update status -> PROCESSING in the same transaction
            sqlx::query!(
                r#"
            UPDATE schedule_jobs
            SET status = $2,
                updated_at = now()
            WHERE id = $1
            "#,
                r.id,
                JobStatus::Processing as JobStatus
            )
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;

            Some(ScheduleJob {
                id: r.id,
                staff_group_id: r.staff_group_id,
                period_begin_date: r.period_begin_date,
                status: JobStatus::Processing,
                error_message: r.error_message,
                created_at: Some(r.created_at),
                updated_at: Some(r.updated_at),
            })
        } else {
            tx.rollback().await?;
            None
        };

        Ok(job)
    }

    async fn mark_processing(&self, id: Uuid) -> anyhow::Result<()> {
        self.update_status(id, JobStatus::Processing, None).await
    }

    async fn mark_completed(&self, id: Uuid) -> anyhow::Result<()> {
        self.update_status(id, JobStatus::Completed, None).await
    }

    async fn mark_failed(&self, id: Uuid, error: &str) -> anyhow::Result<()> {
        self.update_status(id, JobStatus::Failed, Some(error)).await
    }

    // Save Assignments
    async fn save_assignments(
        &self,
        job_id: Uuid,
        assignments: Vec<ShiftAssignment>,
    ) -> anyhow::Result<()> {
        if assignments.is_empty() {
            return Ok(());
        }

        let mut tx: Transaction<Postgres> = self.pool.begin().await?;

        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO shift_assignments (id, schedule_id, staff_id, date, shift_type) ",
        );

        builder.push_values(assignments.iter(), |mut b, a| {
            b.push_bind(a.id)
                .push_bind(job_id)
                .push_bind(a.staff_id)
                .push_bind(a.date)
                .push_bind(a.shift.to_string());
        });

        builder.build().execute(&mut *tx).await?;

        tx.commit().await?;
        Ok(())
    }

    // Get status
    async fn get_status(&self, id: Uuid) -> anyhow::Result<Option<JobStatus>> {
        let row = sqlx::query!(
            r#"
        SELECT status as "status: JobStatus"
        FROM schedule_jobs
        WHERE id = $1
        "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.status))
    }

    // Get result
    async fn get_result(&self, id: Uuid) -> anyhow::Result<Vec<ShiftAssignment>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, staff_id, date, shift_type as "shift_type: ShiftType"
            FROM shift_assignments
            WHERE schedule_id = $1
            ORDER BY date
            "#,
            id
        )
        .fetch_all(&self.pool)
        .await?;

        let assignments = rows
            .into_iter()
            .map(|r| ShiftAssignment {
                id: r.id,
                staff_id: r.staff_id,
                date: r.date,
                shift: r.shift_type,
            })
            .collect();

        Ok(assignments)
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ScheduleJob>> {
        let row = sqlx::query!(
            r#"
        SELECT
            id,
            staff_group_id,
            period_begin_date,
            status as "status: JobStatus",
            error_message,
            created_at,
            updated_at
        FROM schedule_jobs
        WHERE id = $1
        "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ScheduleJob {
            id: r.id,
            staff_group_id: r.staff_group_id,
            period_begin_date: r.period_begin_date,
            status: r.status,
            error_message: r.error_message,
            created_at: Some(r.created_at),
            updated_at: Some(r.updated_at),
        }))
    }
}
