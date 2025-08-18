-- migrations/003_add_user_agent_linking.sql
-- Add user API key management and direct user-agent linking

-- User API Keys table for secure agent registration
CREATE TABLE users.api_keys (
    key_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users.users(user_id) ON DELETE CASCADE,
    key_hash TEXT NOT NULL, -- bcrypt hash of the actual API key
    key_name VARCHAR(100) NOT NULL, -- User-friendly name like "Home PC", "Work Laptop"
    key_prefix VARCHAR(10) NOT NULL, -- First 8 chars for identification (sg_abc12345...)
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at TIMESTAMPTZ, -- Optional expiration date
    last_used TIMESTAMPTZ, -- Track when key was last used
    usage_count INTEGER NOT NULL DEFAULT 0, -- Track how many times used
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    UNIQUE(key_prefix) -- Ensure prefixes are unique for easy identification
);

-- One-time registration tokens (alternative to API keys)
CREATE TABLE users.registration_tokens (
    token_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users.users(user_id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL, -- bcrypt hash of the actual token
    device_name VARCHAR(100) NOT NULL, -- Expected device name
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '24 hours'),
    is_used BOOLEAN NOT NULL DEFAULT FALSE,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Add user relationship to agents
ALTER TABLE agents.endpoints 
ADD COLUMN user_id UUID REFERENCES users.users(user_id) ON DELETE SET NULL,
ADD COLUMN device_name VARCHAR(100), -- User-friendly device name
ADD COLUMN registered_via_key_id UUID REFERENCES users.api_keys(key_id) ON DELETE SET NULL,
ADD COLUMN registered_via_token_id UUID REFERENCES users.registration_tokens(token_id) ON DELETE SET NULL;

-- Create indexes for performance
CREATE INDEX idx_api_keys_user_id ON users.api_keys(user_id);
CREATE INDEX idx_api_keys_prefix ON users.api_keys(key_prefix);
CREATE INDEX idx_api_keys_active ON users.api_keys(is_active) WHERE is_active = TRUE;

CREATE INDEX idx_registration_tokens_user_id ON users.registration_tokens(user_id);
CREATE INDEX idx_registration_tokens_expires ON users.registration_tokens(expires_at);
CREATE INDEX idx_registration_tokens_unused ON users.registration_tokens(is_used) WHERE is_used = FALSE;

CREATE INDEX idx_agents_user_id ON agents.endpoints(user_id);
CREATE INDEX idx_agents_device_name ON agents.endpoints(device_name);

-- Update existing agents to have user_id based on tenant relationship
-- This is a placeholder - you may need to adjust based on your tenant-user relationship
-- UPDATE agents.endpoints SET user_id = (
--     SELECT user_id FROM users.users WHERE users.tenant_id = agents.endpoints.tenant_id LIMIT 1
-- );

-- Add some useful views
CREATE VIEW users.user_devices AS
SELECT 
    u.user_id,
    u.username,
    u.email,
    a.agent_id,
    a.device_name,
    a.hardware_fingerprint,
    a.status,
    a.last_heartbeat,
    a.version,
    a.created_at as registered_at,
    ak.key_name as registered_via_api_key,
    rt.device_name as registered_via_token
FROM users.users u
LEFT JOIN agents.endpoints a ON u.user_id = a.user_id
LEFT JOIN users.api_keys ak ON a.registered_via_key_id = ak.key_id
LEFT JOIN users.registration_tokens rt ON a.registered_via_token_id = rt.token_id;

CREATE VIEW users.active_api_keys AS
SELECT 
    ak.key_id,
    ak.user_id,
    u.username,
    ak.key_name,
    ak.key_prefix,
    ak.last_used,
    ak.usage_count,
    ak.expires_at,
    ak.created_at
FROM users.api_keys ak
JOIN users.users u ON ak.user_id = u.user_id
WHERE ak.is_active = TRUE
AND (ak.expires_at IS NULL OR ak.expires_at > now());