-- migrations/0009_add_password_reset_tokens_table.sql

-- Create password reset tokens table (already created manually, but adding for completeness)
CREATE TABLE IF NOT EXISTS users.password_reset_tokens (
    token_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users.users(user_id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_user_id ON users.password_reset_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_expires ON users.password_reset_tokens(expires_at);
CREATE UNIQUE INDEX IF NOT EXISTS idx_password_reset_tokens_user_unique ON users.password_reset_tokens(user_id) WHERE used = FALSE;
