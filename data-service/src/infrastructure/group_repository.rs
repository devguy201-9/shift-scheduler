use crate::application::error::AppError;
use crate::application::traits::GroupRepository;
use crate::domain::{group::StaffGroup, staff::Staff};
use async_trait::async_trait;
use shared::types::StaffStatus;
use sqlx::PgPool;
use uuid::Uuid;

pub struct GroupRepositoryPg {
    pool: PgPool,
}

impl GroupRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GroupRepository for GroupRepositoryPg {
    async fn create(&self, group: StaffGroup) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO staff_groups (id, name, parent_group_id)
             VALUES ($1, $2, $3)",
            group.id,
            group.name,
            group.parent_group_id
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    async fn resolve_members(&self, group_id: Uuid) -> Result<Vec<Staff>, AppError> {
        let members = sqlx::query_as!(
            Staff,
            r#"
            WITH RECURSIVE subgroups AS (
                SELECT id FROM staff_groups WHERE id = $1
                UNION
                SELECT sg.id
                FROM staff_groups sg
                JOIN subgroups s ON sg.parent_group_id = s.id
            )
            SELECT s.id, s.name, s.email, s.position,
                   s.status as "status: StaffStatus"
            FROM staff s
            JOIN group_memberships gm ON gm.staff_id = s.id
            JOIN subgroups sg ON sg.id = gm.group_id
            WHERE s.status = 'ACTIVE'
            "#,
            group_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(members)
    }

    async fn add_member(&self, group_id: Uuid, staff_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
        INSERT INTO group_memberships (staff_id, group_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
            staff_id,
            group_id
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }

    async fn remove_member(&self, group_id: Uuid, staff_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
        DELETE FROM group_memberships
        WHERE staff_id = $1 AND group_id = $2
        "#,
            staff_id,
            group_id
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Membership not found".into()));
        }

        Ok(())
    }

    async fn update(&self, group: StaffGroup) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
        UPDATE staff_groups
        SET name = $2,
            parent_group_id = $3,
            updated_at = now()
        WHERE id = $1
        "#,
            group.id,
            group.name,
            group.parent_group_id
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Group not found".into()));
        }

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!("DELETE FROM staff_groups WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Group not found".into()));
        }

        Ok(())
    }

    async fn create_batch(&self, groups: Vec<StaffGroup>) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;

        for group in groups {
            sqlx::query!(
                r#"
            INSERT INTO staff_groups (id, name, parent_group_id)
            VALUES ($1, $2, $3)
            "#,
                group.id,
                group.name,
                group.parent_group_id
            )
            .execute(&mut *tx)
            .await
            .map_err(AppError::from)?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<StaffGroup>, AppError> {
        let group = sqlx::query_as!(
            StaffGroup,
            r#"
        SELECT id, name, parent_group_id
        FROM staff_groups
        WHERE id = $1
        "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(group)
    }

    async fn is_member(&self, group_id: Uuid, staff_id: Uuid) -> Result<bool, AppError> {
        let row = sqlx::query!(
            r#"
        SELECT EXISTS(
            SELECT 1
            FROM group_memberships
            WHERE group_id = $1
              AND staff_id = $2
        ) as "exists!"
        "#,
            group_id,
            staff_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(row.exists)
    }
}
