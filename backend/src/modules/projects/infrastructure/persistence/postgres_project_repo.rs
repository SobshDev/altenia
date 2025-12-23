use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use super::models::ProjectRow;
use crate::modules::organizations::domain::OrgId;
use crate::modules::projects::domain::{
    Project, ProjectDomainError, ProjectId, ProjectName, ProjectRepository, RetentionDays,
};

pub struct PostgresProjectRepository {
    pool: Arc<PgPool>,
}

impl PostgresProjectRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn row_to_project(row: ProjectRow) -> Result<Project, ProjectDomainError> {
        let id = ProjectId::new(row.id);
        let org_id = OrgId::new(row.organization_id);
        let name = ProjectName::new(row.name)?;
        let retention_days = RetentionDays::new(row.retention_days)?;

        Ok(Project::reconstruct(
            id,
            org_id,
            name,
            row.description,
            retention_days,
            row.created_at,
            row.updated_at,
            row.deleted_at,
        ))
    }
}

#[async_trait]
impl ProjectRepository for PostgresProjectRepository {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, ProjectDomainError> {
        let row: Option<ProjectRow> = sqlx::query_as(
            r#"
            SELECT id, organization_id, name, description, retention_days,
                   created_at, updated_at, deleted_at
            FROM projects
            WHERE id = $1
            "#,
        )
        .bind(id.as_str())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| ProjectDomainError::InternalError(e.to_string()))?;

        row.map(Self::row_to_project).transpose()
    }

    async fn find_by_org(&self, org_id: &OrgId) -> Result<Vec<Project>, ProjectDomainError> {
        let rows: Vec<ProjectRow> = sqlx::query_as(
            r#"
            SELECT id, organization_id, name, description, retention_days,
                   created_at, updated_at, deleted_at
            FROM projects
            WHERE organization_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id.as_str())
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| ProjectDomainError::InternalError(e.to_string()))?;

        rows.into_iter().map(Self::row_to_project).collect()
    }

    async fn save(&self, project: &Project) -> Result<(), ProjectDomainError> {
        sqlx::query(
            r#"
            INSERT INTO projects (id, organization_id, name, description, retention_days,
                                  created_at, updated_at, deleted_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                retention_days = EXCLUDED.retention_days,
                updated_at = EXCLUDED.updated_at,
                deleted_at = EXCLUDED.deleted_at
            "#,
        )
        .bind(project.id().as_str())
        .bind(project.organization_id().as_str())
        .bind(project.name().as_str())
        .bind(project.description())
        .bind(project.retention_days().value())
        .bind(project.created_at())
        .bind(project.updated_at())
        .bind(project.deleted_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| ProjectDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn exists_by_name_and_org(
        &self,
        name: &str,
        org_id: &OrgId,
    ) -> Result<bool, ProjectDomainError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM projects
            WHERE LOWER(name) = LOWER($1) AND organization_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(name)
        .bind(org_id.as_str())
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| ProjectDomainError::InternalError(e.to_string()))?;

        Ok(count.0 > 0)
    }

    async fn exists_by_name_and_org_excluding(
        &self,
        name: &str,
        org_id: &OrgId,
        exclude_id: &ProjectId,
    ) -> Result<bool, ProjectDomainError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM projects
            WHERE LOWER(name) = LOWER($1) AND organization_id = $2
              AND id != $3 AND deleted_at IS NULL
            "#,
        )
        .bind(name)
        .bind(org_id.as_str())
        .bind(exclude_id.as_str())
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| ProjectDomainError::InternalError(e.to_string()))?;

        Ok(count.0 > 0)
    }
}
