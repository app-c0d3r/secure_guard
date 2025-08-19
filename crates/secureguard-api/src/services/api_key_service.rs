use super::subscription_service::SubscriptionService;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use secureguard_shared::{
    ApiKey, CreateApiKeyRequest, CreateApiKeyResponse, CreateRegistrationTokenRequest,
    CreateRegistrationTokenResponse, RegistrationToken, Result, SecureGuardError,
};
use sqlx::PgPool;
use tracing;
use uuid::Uuid;

pub struct ApiKeyService {
    pool: PgPool,
    subscription_service: SubscriptionService,
}

impl ApiKeyService {
    pub fn new(pool: PgPool) -> Self {
        let subscription_service = SubscriptionService::new(pool.clone());
        Self {
            pool,
            subscription_service,
        }
    }

    /// Generate a new API key for a user
    pub async fn create_api_key(
        &self,
        user_id: Uuid,
        request: CreateApiKeyRequest,
    ) -> Result<CreateApiKeyResponse> {
        // Check subscription limits BEFORE creating API key
        let api_key_limit_check = self
            .subscription_service
            .can_create_api_key(user_id)
            .await?;
        if !api_key_limit_check.allowed {
            return Err(SecureGuardError::SubscriptionLimitExceeded(
                format!("API key creation failed: {}. Please upgrade your subscription to create more API keys.", 
                    api_key_limit_check.message)
            ));
        }

        // Generate a secure API key: sg_{prefix}_{random}
        let key_id = Uuid::new_v4();
        let prefix = format!("sg_{}", &key_id.to_string().replace("-", "")[0..6]);
        let random_suffix = uuid::Uuid::new_v4().to_string().replace("-", "")[0..20].to_string();
        let full_api_key = format!("{}_{}", prefix, random_suffix);

        // Hash the API key for storage
        let key_hash = hash(&full_api_key, DEFAULT_COST).map_err(|e| {
            SecureGuardError::InternalError(format!("Failed to hash API key: {}", e))
        })?;

        // Calculate expiration if requested
        let expires_at = request
            .expires_in_days
            .map(|days| Utc::now() + Duration::days(days as i64));

        // Insert into database
        let api_key = sqlx::query_as!(
            ApiKey,
            r#"
            INSERT INTO users.api_keys (key_id, user_id, key_hash, key_name, key_prefix, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING key_id, user_id, key_name, key_prefix, is_active, expires_at, last_used, usage_count, created_at
            "#,
            key_id,
            user_id,
            key_hash,
            request.key_name,
            prefix,
            expires_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        // Increment API key count in subscription tracking
        if let Err(e) = self
            .subscription_service
            .increment_api_key_count(user_id)
            .await
        {
            // Log the error but don't fail creation - key is already created
            tracing::warn!("Failed to update API key count for user {}: {}", user_id, e);
        }

        tracing::info!(
            target: "secureguard_api",
            security = "api_key_management",
            audit = "api_key_created",
            event = "api_key_created",
            user_id = %user_id,
            key_id = %api_key.key_id,
            key_name = %api_key.key_name,
            key_prefix = %api_key.key_prefix,
            expires_at = ?api_key.expires_at,
            status = "success",
            "API key created successfully"
        );

        Ok(CreateApiKeyResponse {
            key_id: api_key.key_id,
            api_key: full_api_key, // Return full key only once
            key_prefix: api_key.key_prefix,
            key_name: api_key.key_name,
            expires_at: api_key.expires_at,
        })
    }

    /// Validate API key and return user_id if valid
    pub async fn validate_api_key(&self, api_key: &str) -> Result<(Uuid, Uuid)> {
        // Returns (user_id, key_id)
        // Extract prefix from API key
        let parts: Vec<&str> = api_key.split('_').collect();
        if parts.len() != 3 || parts[0] != "sg" {
            return Err(SecureGuardError::AuthenticationError(
                "Invalid API key format".to_string(),
            ));
        }
        let prefix = format!("{}_{}", parts[0], parts[1]);

        // Find API key by prefix
        let stored_key = sqlx::query!(
            r#"
            SELECT key_id, user_id, key_hash, is_active, expires_at, usage_count
            FROM users.api_keys 
            WHERE key_prefix = $1 AND is_active = TRUE
            "#,
            prefix
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        let stored_key = match stored_key {
            Some(key) => key,
            None => {
                tracing::warn!(
                    target: "secureguard_api",
                    security = "api_key_validation",
                    audit = "invalid_api_key_attempt",
                    event = "api_key_validation_failed",
                    key_prefix = %prefix,
                    reason = "key_not_found",
                    status = "failed",
                    "API key validation failed - key not found or inactive"
                );
                return Err(SecureGuardError::AuthenticationError("Invalid API key".to_string()));
            }
        };

        // Check if key is expired
        if let Some(expires_at) = stored_key.expires_at {
            if Utc::now() > expires_at {
                tracing::warn!(
                    target: "secureguard_api",
                    security = "api_key_validation",
                    audit = "expired_api_key_attempt",
                    event = "api_key_validation_failed",
                    key_id = %stored_key.key_id,
                    user_id = %stored_key.user_id,
                    key_prefix = %prefix,
                    expired_at = %expires_at.format("%Y-%m-%d %H:%M:%S UTC"),
                    reason = "key_expired",
                    status = "failed",
                    "API key validation failed - key expired"
                );
                return Err(SecureGuardError::AuthenticationError(
                    "API key has expired".to_string(),
                ));
            }
        }

        // Verify the API key
        let is_valid = verify(api_key, &stored_key.key_hash).map_err(|e| {
            tracing::error!(
                target: "secureguard_api",
                security = "api_key_validation",
                event = "api_key_verification_error",
                key_id = %stored_key.key_id,
                user_id = %stored_key.user_id,
                error = %e,
                status = "error",
                "API key verification error"
            );
            SecureGuardError::InternalError(format!("Failed to verify API key: {}", e))
        })?;

        if !is_valid {
            tracing::warn!(
                target: "secureguard_api",
                security = "api_key_validation",
                audit = "invalid_api_key_attempt",
                event = "api_key_validation_failed",
                key_id = %stored_key.key_id,
                user_id = %stored_key.user_id,
                key_prefix = %prefix,
                reason = "invalid_key_hash",
                usage_count = stored_key.usage_count,
                status = "failed",
                "API key validation failed - invalid key hash"
            );
            return Err(SecureGuardError::AuthenticationError(
                "Invalid API key".to_string(),
            ));
        }

        // Update last used timestamp and usage count
        sqlx::query!(
            "UPDATE users.api_keys SET last_used = $1, usage_count = $2 WHERE key_id = $3",
            Utc::now(),
            stored_key.usage_count + 1,
            stored_key.key_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        tracing::info!(
            target: "secureguard_api",
            security = "api_key_validation",
            audit = "successful_api_key_validation",
            event = "api_key_validated",
            key_id = %stored_key.key_id,
            user_id = %stored_key.user_id,
            key_prefix = %prefix,
            usage_count = stored_key.usage_count + 1,
            status = "success",
            "API key validated successfully"
        );

        Ok((stored_key.user_id, stored_key.key_id))
    }

    /// List user's API keys (without the actual key values)
    pub async fn list_user_api_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>> {
        let keys = sqlx::query_as!(
            ApiKey,
            r#"
            SELECT key_id, user_id, key_name, key_prefix, is_active, expires_at, last_used, usage_count, created_at
            FROM users.api_keys 
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(keys)
    }

    /// Revoke (deactivate) an API key
    pub async fn revoke_api_key(&self, user_id: Uuid, key_id: Uuid) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE users.api_keys SET is_active = FALSE WHERE key_id = $1 AND user_id = $2",
            key_id,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(SecureGuardError::NotFound("API key not found".to_string()));
        }

        // Decrement API key count in subscription tracking
        if let Err(e) = self
            .subscription_service
            .decrement_api_key_count(user_id)
            .await
        {
            // Log the error but don't fail revocation - key is already revoked
            tracing::warn!(
                "Failed to update API key count after revocation for user {}: {}",
                user_id,
                e
            );
        }

        Ok(())
    }

    /// Create a one-time registration token
    pub async fn create_registration_token(
        &self,
        user_id: Uuid,
        request: CreateRegistrationTokenRequest,
    ) -> Result<CreateRegistrationTokenResponse> {
        let token_id = Uuid::new_v4();
        let registration_token =
            format!("rt_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let expires_at = Utc::now() + Duration::hours(24); // 24 hour expiration

        // Hash the token
        let token_hash = hash(&registration_token, DEFAULT_COST)
            .map_err(|e| SecureGuardError::InternalError(format!("Failed to hash token: {}", e)))?;

        // Insert into database
        sqlx::query!(
            r#"
            INSERT INTO users.registration_tokens (token_id, user_id, token_hash, device_name, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            token_id,
            user_id,
            token_hash,
            request.device_name,
            expires_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(CreateRegistrationTokenResponse {
            token_id,
            registration_token, // Return full token only once
            device_name: request.device_name,
            expires_at,
        })
    }

    /// Validate and consume a registration token
    pub async fn validate_and_consume_token(&self, token: &str) -> Result<(Uuid, String)> {
        // Returns (user_id, device_name)
        if !token.starts_with("rt_") {
            return Err(SecureGuardError::AuthenticationError(
                "Invalid token format".to_string(),
            ));
        }

        // Find the token
        let stored_token = sqlx::query!(
            r#"
            SELECT token_id, user_id, token_hash, device_name, expires_at, is_used
            FROM users.registration_tokens 
            WHERE expires_at > $1 AND is_used = FALSE
            "#,
            Utc::now()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        // Find matching token by verifying hash
        let mut valid_token = None;
        for token_record in stored_token {
            if verify(token, &token_record.token_hash).unwrap_or(false) {
                valid_token = Some(token_record);
                break;
            }
        }

        let token_record = valid_token.ok_or_else(|| {
            SecureGuardError::AuthenticationError("Invalid or expired token".to_string())
        })?;

        // Mark token as used
        sqlx::query!(
            "UPDATE users.registration_tokens SET is_used = TRUE, used_at = $1 WHERE token_id = $2",
            Utc::now(),
            token_record.token_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok((token_record.user_id, token_record.device_name))
    }

    /// List user's registration tokens
    pub async fn list_user_registration_tokens(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<RegistrationToken>> {
        let tokens = sqlx::query_as!(
            RegistrationToken,
            r#"
            SELECT token_id, user_id, device_name, expires_at, is_used, used_at, created_at
            FROM users.registration_tokens 
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(tokens)
    }
}
