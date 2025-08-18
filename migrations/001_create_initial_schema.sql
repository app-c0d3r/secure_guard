-- migrations/V001_create_initial_schema.sql

-- Enable UUID extension for unique IDs
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create schemas for clear organization
CREATE SCHEMA IF NOT EXISTS users;
CREATE SCHEMA IF NOT EXISTS agents;
CREATE SCHEMA IF NOT EXISTS tenants;

-- User management schema
CREATE TABLE users.users (
    user_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- Add a trigger to update 'updated_at' column on every row update
    -- This is a very helpful pattern for change tracking
    -- The function is a bit complex for a beginner but extremely useful and stable
    updated_at_trigger TEXT NOT NULL DEFAULT 'on update now()'
);

-- Agents management schema
CREATE TABLE agents.endpoints (
    agent_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    hardware_fingerprint TEXT UNIQUE NOT NULL,
    os_info JSONB NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'unknown',
    last_heartbeat TIMESTAMPTZ,
    version VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tenant management schema (for multi-tenancy)
CREATE TABLE tenants.tenants (
    tenant_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    plan_tier VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);