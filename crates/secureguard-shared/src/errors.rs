use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecureGuardError {
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Authorization failed")]
    AuthorizationFailed,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Agent not found")]
    AgentNotFound,
    
    #[error("Tenant not found")]
    TenantNotFound,
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Subscription limit exceeded: {0}")]
    SubscriptionLimitExceeded(String),
    
    #[error("Feature not available: {0}")]
    FeatureNotAvailable(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub type Result<T> = std::result::Result<T, SecureGuardError>;