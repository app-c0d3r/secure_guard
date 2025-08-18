use sqlx::PgPool;
use uuid::Uuid;
use secureguard_shared::{User, CreateUserRequest, SecureGuardError, Result};
use crate::services::auth_service::AuthService;

pub struct UserService {
    pool: PgPool,
    auth_service: AuthService,
}

impl UserService {
    pub fn new(pool: PgPool, auth_service: AuthService) -> Self {
        Self { pool, auth_service }
    }

    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User> {
        if request.username.is_empty() || request.email.is_empty() || request.password.len() < 8 {
            return Err(SecureGuardError::ValidationError(
                "Invalid username, email, or password too short".to_string()
            ));
        }

        let password_hash = self.auth_service.hash_password(&request.password)?;

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users.users (username, password_hash, email)
            VALUES ($1, $2, $3)
            RETURNING user_id, username, email, created_at, updated_at, is_active
            "#,
            request.username,
            password_hash,
            request.email
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.constraint().is_some() => {
                SecureGuardError::ValidationError("Username or email already exists".to_string())
            }
            _ => SecureGuardError::DatabaseError(e.to_string()),
        })?;

        Ok(user)
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT user_id, username, email, created_at, updated_at, is_active FROM users.users WHERE username = $1 AND is_active = true",
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    pub async fn find_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT user_id, username, email, created_at, updated_at, is_active FROM users.users WHERE user_id = $1 AND is_active = true",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    pub async fn verify_credentials(&self, username: &str, password: &str) -> Result<Option<User>> {
        let row = sqlx::query!(
            "SELECT user_id, username, email, password_hash, created_at, updated_at, is_active FROM users.users WHERE username = $1 AND is_active = true",
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            if self.auth_service.verify_password(password, &row.password_hash)? {
                return Ok(Some(User {
                    user_id: row.user_id,
                    username: row.username,
                    email: row.email,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    is_active: row.is_active,
                }));
            }
        }

        Ok(None)
    }
}