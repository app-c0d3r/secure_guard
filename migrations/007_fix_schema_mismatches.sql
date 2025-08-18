-- migrations/007_fix_schema_mismatches.sql
-- Fix schema mismatches between code expectations and database

-- Add missing columns to users table
ALTER TABLE users.users ADD COLUMN IF NOT EXISTS tenant_id UUID;
ALTER TABLE users.users ADD COLUMN IF NOT EXISTS primary_role_id UUID;
ALTER TABLE users.users ADD COLUMN IF NOT EXISTS phone VARCHAR(20);
ALTER TABLE users.users ADD COLUMN IF NOT EXISTS push_notifications_enabled BOOLEAN DEFAULT TRUE;
ALTER TABLE users.users ADD COLUMN IF NOT EXISTS email_notifications_enabled BOOLEAN DEFAULT TRUE;
ALTER TABLE users.users ADD COLUMN IF NOT EXISTS sms_notifications_enabled BOOLEAN DEFAULT FALSE;

-- Add missing columns to agents.endpoints table
ALTER TABLE agents.endpoints ADD COLUMN IF NOT EXISTS config_version INTEGER DEFAULT 0;
ALTER TABLE agents.endpoints ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT now();

-- Create agent_commands table if it doesn't exist
CREATE TABLE IF NOT EXISTS agent_commands (
    command_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents.endpoints(agent_id) ON DELETE CASCADE,
    command_type VARCHAR(50) NOT NULL,
    command_data JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    executed_at TIMESTAMPTZ,
    response JSONB
);

-- Create indexes for agent_commands
CREATE INDEX IF NOT EXISTS idx_agent_commands_agent_id ON agent_commands(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_commands_status ON agent_commands(status);

-- Update tenants table to have a reference tenant for existing users
INSERT INTO tenants.tenants (tenant_id, name, plan_tier)
VALUES (gen_random_uuid(), 'Default Tenant', 'free')
ON CONFLICT DO NOTHING;

-- Update users to have a tenant_id if they don't have one
UPDATE users.users 
SET tenant_id = (SELECT tenant_id FROM tenants.tenants LIMIT 1)
WHERE tenant_id IS NULL;

-- Update agents to have a tenant_id if they don't have one
UPDATE agents.endpoints 
SET tenant_id = (SELECT tenant_id FROM tenants.tenants LIMIT 1)
WHERE tenant_id IS NULL;