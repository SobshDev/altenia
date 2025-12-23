use std::sync::Arc;

use chrono::{Duration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::domain::UserId;
use crate::modules::organizations::domain::{
    OrgId, OrgRole, OrganizationMemberRepository, OrganizationRepository,
};
use crate::modules::projects::application::dto::*;
use crate::modules::projects::domain::{
    ApiKey, ApiKeyId, ApiKeyName, ApiKeyPrefix, ApiKeyRepository, Project, ProjectDomainError,
    ProjectId, ProjectName, ProjectRepository, RetentionDays,
};

/// Project service - orchestrates all project and API key use cases
pub struct ProjectService<PR, AR, OR, MR, ID>
where
    PR: ProjectRepository,
    AR: ApiKeyRepository,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    project_repo: Arc<PR>,
    api_key_repo: Arc<AR>,
    org_repo: Arc<OR>,
    member_repo: Arc<MR>,
    id_generator: Arc<ID>,
}

impl<PR, AR, OR, MR, ID> ProjectService<PR, AR, OR, MR, ID>
where
    PR: ProjectRepository,
    AR: ApiKeyRepository,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    pub fn new(
        project_repo: Arc<PR>,
        api_key_repo: Arc<AR>,
        org_repo: Arc<OR>,
        member_repo: Arc<MR>,
        id_generator: Arc<ID>,
    ) -> Self {
        Self {
            project_repo,
            api_key_repo,
            org_repo,
            member_repo,
            id_generator,
        }
    }

    /// Verify user is a member of the organization with sufficient permissions
    async fn verify_org_membership(
        &self,
        org_id: &OrgId,
        user_id: &str,
        require_admin: bool,
    ) -> Result<OrgRole, ProjectDomainError> {
        let user_id = UserId::new(user_id.to_string());

        // Verify org exists
        let org = self
            .org_repo
            .find_by_id(org_id)
            .await
            .map_err(|e| ProjectDomainError::InternalError(e.to_string()))?
            .ok_or(ProjectDomainError::NotOrgMember)?;

        if org.is_deleted() {
            return Err(ProjectDomainError::NotOrgMember);
        }

        // Verify membership
        let membership = self
            .member_repo
            .find_by_org_and_user(org_id, &user_id)
            .await
            .map_err(|e| ProjectDomainError::InternalError(e.to_string()))?
            .ok_or(ProjectDomainError::NotOrgMember)?;

        if require_admin && !membership.role().can_update_org() {
            return Err(ProjectDomainError::InsufficientPermissions);
        }

        Ok(*membership.role())
    }

    /// Verify user has access to a project
    async fn verify_project_access(
        &self,
        project_id: &ProjectId,
        user_id: &str,
        require_admin: bool,
    ) -> Result<(Project, OrgRole), ProjectDomainError> {
        let project = self
            .project_repo
            .find_by_id(project_id)
            .await?
            .ok_or(ProjectDomainError::ProjectNotFound)?;

        if project.is_deleted() {
            return Err(ProjectDomainError::ProjectNotFound);
        }

        let role = self
            .verify_org_membership(project.organization_id(), user_id, require_admin)
            .await?;

        Ok((project, role))
    }

    /// Generate a cryptographically secure API key
    fn generate_api_key(&self) -> String {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        let key_part = base64_encode(&bytes);
        format!("alt_pk_{}", key_part)
    }

    /// Hash an API key using SHA256
    fn hash_api_key(&self, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    // ==================== Project Operations ====================

    /// Create a new project
    pub async fn create_project(
        &self,
        cmd: CreateProjectCommand,
    ) -> Result<ProjectResponse, ProjectDomainError> {
        let org_id = OrgId::new(cmd.org_id.clone());

        // 1. Verify user has admin access to org
        self.verify_org_membership(&org_id, &cmd.requesting_user_id, true)
            .await?;

        // 2. Validate name
        let name = ProjectName::new(cmd.name)?;

        // 3. Check name uniqueness in org
        if self
            .project_repo
            .exists_by_name_and_org(name.as_str(), &org_id)
            .await?
        {
            return Err(ProjectDomainError::ProjectAlreadyExists);
        }

        // 4. Validate retention days
        let retention_days = cmd
            .retention_days
            .map(RetentionDays::new)
            .transpose()?
            .unwrap_or_default();

        // 5. Create project
        let project_id = ProjectId::new(self.id_generator.generate());
        let project = Project::new(
            project_id.clone(),
            org_id,
            name,
            cmd.description,
            retention_days,
        );

        // 6. Save project
        self.project_repo.save(&project).await?;

        Ok(ProjectResponse {
            id: project.id().as_str().to_string(),
            name: project.name().as_str().to_string(),
            description: project.description().map(|s| s.to_string()),
            org_id: project.organization_id().as_str().to_string(),
            retention_days: project.retention_days().value(),
            created_at: project.created_at(),
            updated_at: project.updated_at(),
        })
    }

    /// List all projects in an organization
    pub async fn list_projects(
        &self,
        org_id: &str,
        requesting_user_id: &str,
    ) -> Result<Vec<ProjectResponse>, ProjectDomainError> {
        let org_id = OrgId::new(org_id.to_string());

        // Verify user is a member (any role)
        self.verify_org_membership(&org_id, requesting_user_id, false)
            .await?;

        // Get all projects
        let projects = self.project_repo.find_by_org(&org_id).await?;

        Ok(projects
            .into_iter()
            .filter(|p| !p.is_deleted())
            .map(|p| ProjectResponse {
                id: p.id().as_str().to_string(),
                name: p.name().as_str().to_string(),
                description: p.description().map(|s| s.to_string()),
                org_id: p.organization_id().as_str().to_string(),
                retention_days: p.retention_days().value(),
                created_at: p.created_at(),
                updated_at: p.updated_at(),
            })
            .collect())
    }

    /// Get project details
    pub async fn get_project(
        &self,
        project_id: &str,
        requesting_user_id: &str,
    ) -> Result<ProjectResponse, ProjectDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        let (project, _role) = self
            .verify_project_access(&project_id, requesting_user_id, false)
            .await?;

        Ok(ProjectResponse {
            id: project.id().as_str().to_string(),
            name: project.name().as_str().to_string(),
            description: project.description().map(|s| s.to_string()),
            org_id: project.organization_id().as_str().to_string(),
            retention_days: project.retention_days().value(),
            created_at: project.created_at(),
            updated_at: project.updated_at(),
        })
    }

    /// Update a project
    pub async fn update_project(
        &self,
        cmd: UpdateProjectCommand,
    ) -> Result<ProjectResponse, ProjectDomainError> {
        let project_id = ProjectId::new(cmd.project_id.clone());

        // 1. Verify admin access
        let (mut project, _role) = self
            .verify_project_access(&project_id, &cmd.requesting_user_id, true)
            .await?;

        // 2. Validate and check name uniqueness if changing
        let new_name = if let Some(name_str) = cmd.name {
            let name = ProjectName::new(name_str)?;
            if self
                .project_repo
                .exists_by_name_and_org_excluding(
                    name.as_str(),
                    project.organization_id(),
                    &project_id,
                )
                .await?
            {
                return Err(ProjectDomainError::ProjectAlreadyExists);
            }
            Some(name)
        } else {
            None
        };

        // 3. Validate retention days if changing
        let new_retention = cmd.retention_days.map(RetentionDays::new).transpose()?;

        // 4. Update project
        project.update(new_name, cmd.description, new_retention);
        self.project_repo.save(&project).await?;

        Ok(ProjectResponse {
            id: project.id().as_str().to_string(),
            name: project.name().as_str().to_string(),
            description: project.description().map(|s| s.to_string()),
            org_id: project.organization_id().as_str().to_string(),
            retention_days: project.retention_days().value(),
            created_at: project.created_at(),
            updated_at: project.updated_at(),
        })
    }

    /// Delete a project (soft delete)
    pub async fn delete_project(
        &self,
        cmd: DeleteProjectCommand,
    ) -> Result<(), ProjectDomainError> {
        let project_id = ProjectId::new(cmd.project_id);

        // Verify admin access
        let (mut project, _role) = self
            .verify_project_access(&project_id, &cmd.requesting_user_id, true)
            .await?;

        // Soft delete
        project.soft_delete()?;
        self.project_repo.save(&project).await?;

        Ok(())
    }

    // ==================== API Key Operations ====================

    /// Create a new API key
    pub async fn create_api_key(
        &self,
        cmd: CreateApiKeyCommand,
    ) -> Result<ApiKeyCreatedResponse, ProjectDomainError> {
        let project_id = ProjectId::new(cmd.project_id.clone());

        // 1. Verify admin access
        let (project, _role) = self
            .verify_project_access(&project_id, &cmd.requesting_user_id, true)
            .await?;

        // 2. Validate name
        let name = ApiKeyName::new(cmd.name)?;

        // 3. Generate key
        let plain_key = self.generate_api_key();
        let key_hash = self.hash_api_key(&plain_key);
        let key_prefix = ApiKeyPrefix::from_key(&plain_key);

        // 4. Calculate expiry
        let expires_at = cmd
            .expires_in_days
            .map(|days| Utc::now() + Duration::days(days));

        // 5. Create API key
        let api_key_id = ApiKeyId::new(self.id_generator.generate());
        let api_key = ApiKey::new(
            api_key_id,
            ProjectId::new(project.id().as_str().to_string()),
            name,
            key_prefix.clone(),
            key_hash,
            expires_at,
        );

        // 6. Save API key
        self.api_key_repo.save(&api_key).await?;

        Ok(ApiKeyCreatedResponse {
            id: api_key.id().as_str().to_string(),
            name: api_key.name().as_str().to_string(),
            key_prefix: key_prefix.as_str().to_string(),
            plain_key,
            created_at: api_key.created_at(),
            expires_at: api_key.expires_at(),
        })
    }

    /// List all API keys for a project
    pub async fn list_api_keys(
        &self,
        project_id: &str,
        requesting_user_id: &str,
    ) -> Result<Vec<ApiKeyResponse>, ProjectDomainError> {
        let project_id = ProjectId::new(project_id.to_string());

        // Verify access (any role can view)
        self.verify_project_access(&project_id, requesting_user_id, false)
            .await?;

        // Get all API keys
        let api_keys = self.api_key_repo.find_by_project(&project_id).await?;

        Ok(api_keys
            .into_iter()
            .map(|k| ApiKeyResponse {
                id: k.id().as_str().to_string(),
                name: k.name().as_str().to_string(),
                key_prefix: k.key_prefix().as_str().to_string(),
                created_at: k.created_at(),
                expires_at: k.expires_at(),
                is_active: k.is_valid(),
            })
            .collect())
    }

    /// Revoke an API key
    pub async fn revoke_api_key(&self, cmd: RevokeApiKeyCommand) -> Result<(), ProjectDomainError> {
        let project_id = ProjectId::new(cmd.project_id);

        // 1. Verify admin access
        self.verify_project_access(&project_id, &cmd.requesting_user_id, true)
            .await?;

        // 2. Get and revoke API key
        let api_key_id = ApiKeyId::new(cmd.api_key_id);
        let mut api_key = self
            .api_key_repo
            .find_by_id(&api_key_id)
            .await?
            .ok_or(ProjectDomainError::ApiKeyNotFound)?;

        // Verify key belongs to this project
        if api_key.project_id().as_str() != project_id.as_str() {
            return Err(ProjectDomainError::ApiKeyNotFound);
        }

        api_key.revoke();
        self.api_key_repo.save(&api_key).await?;

        Ok(())
    }

    /// Validate an API key (for ingestion)
    pub async fn validate_api_key(
        &self,
        plain_key: &str,
    ) -> Result<(ProjectId, Project), ProjectDomainError> {
        // 1. Hash the key
        let key_hash = self.hash_api_key(plain_key);

        // 2. Find API key by hash
        let api_key = self
            .api_key_repo
            .find_by_hash(&key_hash)
            .await?
            .ok_or(ProjectDomainError::ApiKeyInvalid)?;

        // 3. Check if valid
        if api_key.is_revoked() {
            return Err(ProjectDomainError::ApiKeyRevoked);
        }
        if api_key.is_expired() {
            return Err(ProjectDomainError::ApiKeyExpired);
        }

        // 4. Get project
        let project = self
            .project_repo
            .find_by_id(api_key.project_id())
            .await?
            .ok_or(ProjectDomainError::ProjectNotFound)?;

        if project.is_deleted() {
            return Err(ProjectDomainError::ProjectNotFound);
        }

        Ok((
            ProjectId::new(project.id().as_str().to_string()),
            project,
        ))
    }
}

/// Simple base64 encoding (URL-safe without padding)
fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}
