use crate::application::error::AppError;
use crate::application::traits::StaffRepository;
use crate::domain::staff::Staff;
use async_trait::async_trait;
use shared::types::StaffStatus;
use sqlx::PgPool;
use uuid::Uuid;

pub struct StaffRepositoryPg {
    pool: PgPool,
}

impl StaffRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StaffRepository for StaffRepositoryPg {
    async fn create(&self, staff: Staff) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO staff (id, name, email, position, status)
             VALUES ($1, $2, $3, $4, $5)",
            staff.id,
            staff.name,
            staff.email,
            staff.position,
            staff.status as StaffStatus
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Staff>, AppError> {
        let row = sqlx::query_as!(
            Staff,
            r#"
            SELECT id, name, email, position,
                   status as "status: StaffStatus"
            FROM staff
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(row)
    }
    async fn update(&self, staff: Staff) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
        UPDATE staff
        SET name = $2,
            email = $3,
            position = $4,
            status = $5,
            updated_at = now()
        WHERE id = $1
        "#,
            staff.id,
            staff.name,
            staff.email,
            staff.position,
            staff.status as _
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }
    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
        DELETE FROM staff
        WHERE id = $1
        "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    async fn create_batch(&self, staff_list: Vec<Staff>) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(AppError::from)?;

        for staff in staff_list {
            sqlx::query!(
                r#"
            INSERT INTO staff (id, name, email, position, status)
            VALUES ($1, $2, $3, $4, $5)
            "#,
                staff.id,
                staff.name,
                staff.email,
                staff.position,
                staff.status as _
            )
            .execute(&mut *tx)
            .await
            .map_err(AppError::from)?;
        }

        tx.commit().await.map_err(AppError::from)?;

        Ok(())
    }
}
