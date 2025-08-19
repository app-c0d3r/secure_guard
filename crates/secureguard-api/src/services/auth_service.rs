use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use secureguard_shared::{Result, SecureGuardError};
use serde::{Deserialize, Serialize};
use tracing;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Clone)]
pub struct AuthService {
    jwt_secret: String,
}

impl AuthService {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self::new("test-secret-key".to_string())
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| SecureGuardError::InternalError(e.to_string()))?;

        Ok(password_hash.to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| SecureGuardError::InternalError(e.to_string()))?;

        let argon2 = Argon2::default();
        let is_valid = argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();

        // Security audit logging
        if is_valid {
            tracing::info!(
                target: "secureguard_api",
                security = "password_verification",
                event = "password_verified",
                status = "success",
                "Password verification successful"
            );
        } else {
            tracing::warn!(
                target: "secureguard_api", 
                security = "password_verification",
                event = "password_verification_failed",
                status = "failed",
                "Password verification failed - invalid password"
            );
        }

        Ok(is_valid)
    }

    pub fn generate_token(&self, user_id: Uuid) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(24);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        match encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        ) {
            Ok(token) => {
                tracing::info!(
                    target: "secureguard_api",
                    security = "token_generation",
                    event = "token_generated",
                    user_id = %user_id,
                    expires_at = %exp.format("%Y-%m-%d %H:%M:%S UTC"),
                    status = "success",
                    "JWT token generated successfully"
                );
                Ok(token)
            }
            Err(e) => {
                tracing::error!(
                    target: "secureguard_api",
                    security = "token_generation",
                    event = "token_generation_failed",
                    user_id = %user_id,
                    error = %e,
                    status = "failed",
                    "JWT token generation failed"
                );
                Err(SecureGuardError::InternalError(e.to_string()))
            }
        }
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .map(|data| data.claims)
        .map_err(|_| SecureGuardError::AuthenticationFailed)
    }
}
