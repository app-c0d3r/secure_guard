-- migrations/005_add_remote_commands_system.sql
-- Remote command execution and agent lifecycle management

-- Agent Commands Table - Stores remote commands sent to agents
CREATE TABLE agent_commands (
    command_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents.endpoints(agent_id) ON DELETE CASCADE,
    
    -- Command Details
    command_type VARCHAR(50) NOT NULL,
    command_data JSONB NOT NULL,
    parameters JSONB,
    
    -- Sender Information
    sender_user_id UUID NOT NULL REFERENCES users.users(user_id),
    sender_username VARCHAR(255) NOT NULL,
    sender_role VARCHAR(20) NOT NULL, -- user, admin, analyst, system_admin
    sender_ip VARCHAR(45), -- IPv4/IPv6 address
    
    -- Execution Status
    status VARCHAR(20) NOT NULL DEFAULT 'queued', -- queued, sent, in_progress, completed, failed, timeout, cancelled
    priority INTEGER NOT NULL DEFAULT 0, -- 0=normal, 1=high, 2=urgent
    
    -- Timing
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    sent_at TIMESTAMPTZ, -- When command was sent to agent
    started_at TIMESTAMPTZ, -- When agent started processing
    completed_at TIMESTAMPTZ, -- When execution completed
    timeout_at TIMESTAMPTZ, -- When command times out
    
    -- Response
    response_data JSONB,
    error_message TEXT,
    execution_time_ms BIGINT,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Agent Update System - Track agent versions and updates
CREATE TABLE agent_updates (
    update_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Version Information
    version VARCHAR(50) NOT NULL,
    release_notes TEXT,
    release_date TIMESTAMPTZ NOT NULL,
    
    -- Update Details
    download_url TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    file_hash VARCHAR(128) NOT NULL,
    signature TEXT, -- Digital signature
    
    -- Requirements
    min_agent_version VARCHAR(50),
    supported_platforms TEXT[], -- ["windows", "linux", "macos"]
    
    -- Rollout Control
    rollout_percentage INTEGER NOT NULL DEFAULT 0, -- 0-100% rollout
    is_security_update BOOLEAN NOT NULL DEFAULT FALSE,
    is_mandatory BOOLEAN NOT NULL DEFAULT FALSE,
    force_install_after TIMESTAMPTZ,
    
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Agent Update Executions - Track individual agent update attempts
CREATE TABLE agent_update_executions (
    execution_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents.endpoints(agent_id) ON DELETE CASCADE,
    update_id UUID NOT NULL REFERENCES agent_updates(update_id),
    
    -- Execution Status
    status VARCHAR(20) NOT NULL DEFAULT 'queued', -- queued, downloading, installing, completed, failed, rollback
    
    -- Progress Tracking
    download_progress INTEGER DEFAULT 0, -- 0-100%
    install_progress INTEGER DEFAULT 0, -- 0-100%
    
    -- Timing
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    
    -- Version Information
    old_version VARCHAR(50),
    new_version VARCHAR(50),
    
    -- Error Handling
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    
    -- Rollback Information
    rollback_reason TEXT,
    rollback_completed_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Agent Features Management - Track dynamic feature availability
CREATE TABLE agent_features (
    feature_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Feature Information
    feature_name VARCHAR(100) UNIQUE NOT NULL,
    display_name VARCHAR(200) NOT NULL,
    description TEXT,
    category VARCHAR(50), -- monitoring, security, forensics, management
    
    -- Version Requirements
    version VARCHAR(20) NOT NULL DEFAULT '1.0.0',
    min_agent_version VARCHAR(50),
    
    -- Subscription Requirements
    required_subscription VARCHAR(20) NOT NULL DEFAULT 'free', -- free, starter, professional, enterprise
    required_role VARCHAR(20) DEFAULT 'user', -- user, admin, analyst, system_admin
    
    -- Feature Configuration
    default_config JSONB,
    config_schema JSONB, -- JSON schema for validation
    
    -- Rollout Control
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    auto_enable BOOLEAN NOT NULL DEFAULT FALSE,
    rollout_percentage INTEGER DEFAULT 100, -- 0-100%
    
    -- Dependencies
    dependencies TEXT[], -- Array of required feature names
    conflicts_with TEXT[], -- Array of conflicting features
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Agent Feature States - Track which features are enabled per agent
CREATE TABLE agent_feature_states (
    state_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents.endpoints(agent_id) ON DELETE CASCADE,
    feature_id UUID NOT NULL REFERENCES agent_features(feature_id) ON DELETE CASCADE,
    
    -- State Information
    is_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    is_available BOOLEAN NOT NULL DEFAULT TRUE, -- Based on subscription/role
    
    -- Configuration
    feature_config JSONB,
    
    -- Status Tracking
    last_status_check TIMESTAMPTZ,
    status_message TEXT,
    error_message TEXT,
    
    -- Timing
    enabled_at TIMESTAMPTZ,
    disabled_at TIMESTAMPTZ,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    UNIQUE(agent_id, feature_id)
);

-- Agent Heartbeats - Enhanced heartbeat tracking
CREATE TABLE agent_heartbeats (
    heartbeat_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents.endpoints(agent_id) ON DELETE CASCADE,
    
    -- Agent Status
    status VARCHAR(20) NOT NULL, -- online, offline, updating, error
    version VARCHAR(50),
    
    -- System Metrics
    system_metrics JSONB,
    active_features TEXT[],
    
    -- Configuration
    config_version INTEGER DEFAULT 1,
    subscription_tier VARCHAR(20),
    
    -- Timing
    heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    -- Performance
    response_time_ms INTEGER,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Command Audit Log - Comprehensive audit trail
CREATE TABLE command_audit_log (
    audit_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Command Reference
    command_id UUID REFERENCES agent_commands(command_id),
    agent_id UUID NOT NULL REFERENCES agents.endpoints(agent_id),
    
    -- Event Information
    event_type VARCHAR(50) NOT NULL, -- submitted, sent, started, completed, failed, timeout
    event_timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    -- User Context
    user_id UUID NOT NULL REFERENCES users.users(user_id),
    username VARCHAR(255) NOT NULL,
    user_role VARCHAR(20) NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    
    -- Command Context
    command_type VARCHAR(50) NOT NULL,
    subscription_tier VARCHAR(20),
    
    -- Security Context
    risk_level VARCHAR(10) DEFAULT 'low', -- low, medium, high, critical
    requires_approval BOOLEAN DEFAULT FALSE,
    approved_by UUID REFERENCES users.users(user_id),
    
    -- Event Details
    event_data JSONB,
    error_details TEXT,
    
    -- Compliance
    retention_until TIMESTAMPTZ, -- For compliance data retention
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes for Performance
CREATE INDEX idx_agent_commands_agent_id ON agent_commands(agent_id);
CREATE INDEX idx_agent_commands_status ON agent_commands(status);
CREATE INDEX idx_agent_commands_sender ON agent_commands(sender_user_id);
CREATE INDEX idx_agent_commands_submitted_at ON agent_commands(submitted_at);
CREATE INDEX idx_agent_commands_type ON agent_commands(command_type);

CREATE INDEX idx_agent_updates_version ON agent_updates(version);
CREATE INDEX idx_agent_updates_active ON agent_updates(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_agent_updates_rollout ON agent_updates(rollout_percentage) WHERE rollout_percentage < 100;

CREATE INDEX idx_agent_update_executions_agent_id ON agent_update_executions(agent_id);
CREATE INDEX idx_agent_update_executions_status ON agent_update_executions(status);
CREATE INDEX idx_agent_update_executions_update_id ON agent_update_executions(update_id);

CREATE INDEX idx_agent_features_name ON agent_features(feature_name);
CREATE INDEX idx_agent_features_subscription ON agent_features(required_subscription);
CREATE INDEX idx_agent_features_enabled ON agent_features(is_enabled) WHERE is_enabled = TRUE;

CREATE INDEX idx_agent_feature_states_agent_id ON agent_feature_states(agent_id);
CREATE INDEX idx_agent_feature_states_feature_id ON agent_feature_states(feature_id);
CREATE INDEX idx_agent_feature_states_enabled ON agent_feature_states(is_enabled) WHERE is_enabled = TRUE;

CREATE INDEX idx_agent_heartbeats_agent_id ON agent_heartbeats(agent_id);
CREATE INDEX idx_agent_heartbeats_timestamp ON agent_heartbeats(heartbeat_at);
CREATE INDEX idx_agent_heartbeats_status ON agent_heartbeats(status);

CREATE INDEX idx_command_audit_agent_id ON command_audit_log(agent_id);
CREATE INDEX idx_command_audit_user_id ON command_audit_log(user_id);
CREATE INDEX idx_command_audit_event_type ON command_audit_log(event_type);
CREATE INDEX idx_command_audit_timestamp ON command_audit_log(event_timestamp);
CREATE INDEX idx_command_audit_command_id ON command_audit_log(command_id);

-- Add new columns to existing agents table
ALTER TABLE agents.endpoints 
ADD COLUMN IF NOT EXISTS config_version INTEGER DEFAULT 1,
ADD COLUMN IF NOT EXISTS last_update_check TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS update_status VARCHAR(20) DEFAULT 'current',
ADD COLUMN IF NOT EXISTS pending_commands INTEGER DEFAULT 0,
ADD COLUMN IF NOT EXISTS active_features TEXT[] DEFAULT '{}';

-- Insert default agent features
INSERT INTO agent_features (
    feature_name, display_name, description, category, required_subscription, 
    required_role, default_config, auto_enable
) VALUES 
-- Basic Features (Free)
(
    'basic_monitoring', 'Basic System Monitoring', 
    'Essential system health and status monitoring', 'monitoring',
    'free', 'user',
    '{"scan_interval": 300, "alerts_enabled": true}',
    true
),
(
    'agent_management', 'Agent Management', 
    'Basic agent control and configuration', 'management',
    'free', 'user',
    '{"auto_update": false, "remote_config": false}',
    true
),

-- Starter Features 
(
    'real_time_monitoring', 'Real-time Monitoring',
    'Continuous real-time security monitoring and alerting', 'monitoring',
    'starter', 'user',
    '{"scan_interval": 30, "real_time_alerts": true, "file_monitoring": true}',
    true
),
(
    'api_access', 'API Access',
    'RESTful API access for integrations', 'management',
    'starter', 'user',
    '{"rate_limit": 1000, "webhook_enabled": true}',
    false
),

-- Professional Features
(
    'advanced_threat_detection', 'Advanced Threat Detection',
    'AI-powered behavioral analysis and advanced threat detection', 'security',
    'professional', 'user',
    '{"ai_model": "standard", "behavioral_analysis": true, "cloud_analysis": true}',
    true
),
(
    'custom_rules', 'Custom Security Rules',
    'Create and manage custom security detection rules', 'security',
    'professional', 'admin',
    '{"max_rules": 50, "rule_complexity": "standard"}',
    false
),
(
    'vulnerability_scanning', 'Vulnerability Scanning',
    'Automated vulnerability detection and assessment', 'security',
    'professional', 'admin',
    '{"scan_frequency": "weekly", "cve_updates": true}',
    true
),
(
    'remote_commands', 'Remote Command Execution',
    'Execute commands remotely for investigation and response', 'management',
    'professional', 'admin',
    '{"file_operations": true, "timeout": 300, "audit_all": true}',
    false
),
(
    'file_integrity_monitoring', 'File Integrity Monitoring',
    'Monitor critical files for unauthorized changes', 'security',
    'professional', 'admin',
    '{"watch_paths": ["/etc", "/bin", "/usr/bin"], "check_interval": 300}',
    false
),

-- Enterprise Features
(
    'forensic_collection', 'Forensic Data Collection',
    'Advanced forensic evidence collection and analysis', 'security',
    'enterprise', 'analyst',
    '{"memory_dumps": true, "network_capture": true, "full_disk": true}',
    false
),
(
    'compliance_reporting', 'Compliance Reporting',
    'Generate compliance reports and audit trails', 'management',
    'enterprise', 'admin',
    '{"report_types": ["pci", "hipaa", "sox"], "automated": true}',
    false
),
(
    'bulk_operations', 'Bulk Operations',
    'Perform operations across multiple agents simultaneously', 'management',
    'enterprise', 'admin',
    '{"max_agents": 1000, "parallel_execution": true}',
    false
),
(
    'custom_dashboards', 'Custom Dashboards',
    'Create personalized security monitoring dashboards', 'management',
    'enterprise', 'user',
    '{"max_dashboards": 10, "custom_widgets": true}',
    false
);

-- Insert sample update for current version
INSERT INTO agent_updates (
    version, release_notes, release_date, download_url, file_size, file_hash,
    supported_platforms, rollout_percentage, is_active
) VALUES (
    '1.0.0',
    'Initial release with core security monitoring capabilities',
    now(),
    'https://updates.secureguard.com/agent/v1.0.0/secureguard-agent-1.0.0.exe',
    52428800, -- 50MB
    'sha256:a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6',
    ARRAY['windows', 'linux', 'macos'],
    100,
    true
);

-- Create views for easier querying
CREATE VIEW agent_command_summary AS
SELECT 
    ac.command_id,
    ac.agent_id,
    ae.device_name,
    ac.command_type,
    ac.status,
    ac.sender_username,
    ac.sender_role,
    ac.submitted_at,
    ac.completed_at,
    ac.execution_time_ms,
    CASE 
        WHEN ac.completed_at IS NOT NULL THEN 
            EXTRACT(EPOCH FROM (ac.completed_at - ac.submitted_at)) * 1000
        ELSE NULL 
    END as total_time_ms
FROM agent_commands ac
JOIN agents.endpoints ae ON ac.agent_id = ae.agent_id;

CREATE VIEW agent_feature_summary AS
SELECT 
    ae.agent_id,
    ae.device_name,
    ae.user_id,
    p.plan_name as subscription_plan,
    COUNT(afs.feature_id) as total_features,
    COUNT(CASE WHEN afs.is_enabled THEN 1 END) as enabled_features,
    COUNT(CASE WHEN afs.is_available THEN 1 END) as available_features,
    array_agg(
        CASE WHEN afs.is_enabled THEN af.feature_name END
    ) FILTER (WHERE afs.is_enabled) as active_feature_names
FROM agents.endpoints ae
LEFT JOIN agent_feature_states afs ON ae.agent_id = afs.agent_id
LEFT JOIN agent_features af ON afs.feature_id = af.feature_id
LEFT JOIN subscriptions.user_subscriptions us ON ae.user_id = us.user_id
LEFT JOIN subscriptions.plans p ON us.plan_id = p.plan_id
WHERE us.status = 'active'
GROUP BY ae.agent_id, ae.device_name, ae.user_id, p.plan_name;

-- Create function to automatically enable features based on subscription
CREATE OR REPLACE FUNCTION update_agent_features_on_subscription_change() 
RETURNS TRIGGER AS $$
BEGIN
    -- When subscription changes, update feature availability
    INSERT INTO agent_feature_states (agent_id, feature_id, is_available, is_enabled)
    SELECT 
        ae.agent_id,
        af.feature_id,
        CASE 
            WHEN af.required_subscription = 'free' THEN TRUE
            WHEN af.required_subscription = 'starter' AND p.plan_slug IN ('starter', 'professional', 'enterprise') THEN TRUE
            WHEN af.required_subscription = 'professional' AND p.plan_slug IN ('professional', 'enterprise') THEN TRUE
            WHEN af.required_subscription = 'enterprise' AND p.plan_slug = 'enterprise' THEN TRUE
            ELSE FALSE
        END as is_available,
        CASE 
            WHEN af.auto_enable AND af.required_subscription = 'free' THEN TRUE
            WHEN af.auto_enable AND af.required_subscription = 'starter' AND p.plan_slug IN ('starter', 'professional', 'enterprise') THEN TRUE
            WHEN af.auto_enable AND af.required_subscription = 'professional' AND p.plan_slug IN ('professional', 'enterprise') THEN TRUE
            WHEN af.auto_enable AND af.required_subscription = 'enterprise' AND p.plan_slug = 'enterprise' THEN TRUE
            ELSE FALSE
        END as is_enabled
    FROM agents.endpoints ae
    JOIN agent_features af ON TRUE
    JOIN subscriptions.plans p ON p.plan_id = NEW.plan_id
    WHERE ae.user_id = NEW.user_id
    ON CONFLICT (agent_id, feature_id) 
    DO UPDATE SET 
        is_available = EXCLUDED.is_available,
        is_enabled = CASE 
            WHEN EXCLUDED.is_available AND agent_features.auto_enable THEN TRUE
            WHEN NOT EXCLUDED.is_available THEN FALSE
            ELSE agent_feature_states.is_enabled
        END,
        updated_at = now();
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for subscription changes
CREATE TRIGGER trigger_update_agent_features
    AFTER UPDATE OF plan_id ON subscriptions.user_subscriptions
    FOR EACH ROW
    EXECUTE FUNCTION update_agent_features_on_subscription_change();