use crate::services::auth_service::AuthService;
use chrono::{Duration, Utc};
use secureguard_shared::{CreateUserRequest, Result, SecureGuardError, User};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

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
                "Invalid username, email, or password too short".to_string(),
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
        // Check if account is locked
        let now = Utc::now();
        let row = sqlx::query!(
            r#"
            SELECT user_id, username, email, password_hash, created_at, updated_at, is_active,
                   must_change_password, failed_login_attempts, account_locked_until, role
            FROM users.users 
            WHERE username = $1 AND is_active = true
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            // Check if account is locked
            if let Some(locked_until) = row.account_locked_until {
                if locked_until > now {
                    return Err(SecureGuardError::ValidationError(
                        "Account is temporarily locked due to too many failed login attempts"
                            .to_string(),
                    ));
                }
            }

            // Verify password
            if self
                .auth_service
                .verify_password(password, &row.password_hash)?
            {
                // Handle successful login
                sqlx::query!("SELECT users.handle_successful_login($1)", username)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

                return Ok(Some(User {
                    user_id: row.user_id,
                    username: row.username,
                    email: row.email,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    is_active: row.is_active,
                }));
            } else {
                // Handle failed login
                sqlx::query!("SELECT users.handle_failed_login($1)", username)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;
            }
        }

        Ok(None)
    }

    pub async fn must_change_password(&self, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            "SELECT must_change_password FROM users.users WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(result.map(|r| r.must_change_password).unwrap_or(false))
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        // Validate password strength
        if !self.validate_password_strength(new_password).await? {
            return Err(SecureGuardError::ValidationError(
                "Password does not meet security requirements".to_string(),
            ));
        }

        // Get current user data
        let user_data = sqlx::query!(
            "SELECT password_hash, password_history FROM users.users WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .ok_or(SecureGuardError::UserNotFound)?;

        // Verify old password
        if !self
            .auth_service
            .verify_password(old_password, &user_data.password_hash)?
        {
            return Err(SecureGuardError::AuthenticationFailed);
        }

        // Check password history
        let history: Vec<String> =
            serde_json::from_value(user_data.password_history.unwrap_or(Value::Array(vec![])))
                .unwrap_or_default();

        for old_hash in &history {
            if self.auth_service.verify_password(new_password, old_hash)? {
                return Err(SecureGuardError::ValidationError(
                    "Password has been used recently. Please choose a different password"
                        .to_string(),
                ));
            }
        }

        // Hash new password
        let new_hash = self.auth_service.hash_password(new_password)?;

        // Update password history (keep last 5)
        let mut new_history = history;
        new_history.insert(0, user_data.password_hash);
        new_history.truncate(5);

        // Update password
        sqlx::query!(
            r#"
            UPDATE users.users 
            SET password_hash = $1, 
                must_change_password = FALSE, 
                password_last_changed = now(),
                password_history = $2,
                updated_at = now()
            WHERE user_id = $3
            "#,
            new_hash,
            serde_json::to_value(new_history).unwrap(),
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn validate_password_strength(&self, password: &str) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            SELECT users.validate_password_strength($1, 
                (SELECT min_length FROM users.password_policies LIMIT 1),
                (SELECT require_uppercase FROM users.password_policies LIMIT 1),
                (SELECT require_lowercase FROM users.password_policies LIMIT 1),
                (SELECT require_numbers FROM users.password_policies LIMIT 1),
                (SELECT require_special_chars FROM users.password_policies LIMIT 1)
            ) as is_valid
            "#,
            password
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(result.is_valid.unwrap_or(false))
    }

    pub async fn get_password_policy(&self) -> Result<PasswordPolicy> {
        let policy = sqlx::query!(
            r#"
            SELECT min_length, require_uppercase, require_lowercase, 
                   require_numbers, require_special_chars, max_age_days
            FROM users.password_policies LIMIT 1
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(PasswordPolicy {
            min_length: policy.min_length,
            require_uppercase: policy.require_uppercase,
            require_lowercase: policy.require_lowercase,
            require_numbers: policy.require_numbers,
            require_special_chars: policy.require_special_chars,
            max_age_days: policy.max_age_days,
        })
    }

    pub async fn request_password_reset(&self, email: &str) -> Result<()> {
        // Check if user exists
        let user = sqlx::query!(
            "SELECT user_id FROM users.users WHERE email = $1 AND is_active = true",
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        if let Some(user) = user {
            // Generate reset token (use a secure random token)
            let reset_token = Uuid::new_v4().to_string();
            let expires_at = Utc::now() + Duration::hours(1); // Token expires in 1 hour

            // Store reset token in database
            sqlx::query!(
                r#"
                INSERT INTO users.password_reset_tokens (user_id, token, expires_at)
                VALUES ($1, $2, $3)
                ON CONFLICT (user_id) DO UPDATE SET
                    token = EXCLUDED.token,
                    expires_at = EXCLUDED.expires_at,
                    created_at = now()
                "#,
                user.user_id,
                reset_token,
                expires_at
            )
            .execute(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

            // TODO: Send email with reset link
            // For now, we'll just log the token (in production, send email)
            tracing::info!("Password reset token for {}: {}", email, reset_token);
        }

        Ok(())
    }

    pub async fn confirm_password_reset(&self, token: &str, new_password: &str) -> Result<()> {
        // Validate password strength
        if !self.validate_password_strength(new_password).await? {
            return Err(SecureGuardError::ValidationError(
                "Password does not meet security requirements".to_string(),
            ));
        }

        // Find valid reset token
        let reset_data = sqlx::query!(
            r#"
            SELECT prt.user_id, u.password_history 
            FROM users.password_reset_tokens prt
            JOIN users.users u ON prt.user_id = u.user_id
            WHERE prt.token = $1 AND prt.expires_at > now() AND prt.used = false
            "#,
            token
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .ok_or(SecureGuardError::ValidationError(
            "Invalid or expired reset token".to_string(),
        ))?;

        // Check password history
        let history: Vec<String> =
            serde_json::from_value(reset_data.password_history.unwrap_or(Value::Array(vec![])))
                .unwrap_or_default();

        for old_hash in &history {
            if self.auth_service.verify_password(new_password, old_hash)? {
                return Err(SecureGuardError::ValidationError(
                    "Password has been used recently. Please choose a different password"
                        .to_string(),
                ));
            }
        }

        // Hash new password
        let new_hash = self.auth_service.hash_password(new_password)?;

        // Update user password and mark token as used
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        sqlx::query!(
            r#"
            UPDATE users.users 
            SET password_hash = $1, 
                must_change_password = FALSE, 
                password_last_changed = now(),
                updated_at = now()
            WHERE user_id = $2
            "#,
            new_hash,
            reset_data.user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        sqlx::query!(
            "UPDATE users.password_reset_tokens SET used = true WHERE token = $1",
            token
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct PasswordPolicy {
    pub min_length: i32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_age_days: i32,
}
