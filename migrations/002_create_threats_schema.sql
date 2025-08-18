-- migrations/V002_create_threats_schema.sql
-- Phase 2: Threat Detection and Event Processing Schema

-- Create threats schema for security events and detection
CREATE SCHEMA IF NOT EXISTS threats;

-- Security events table with time-based partitioning
CREATE TABLE threats.security_events (
    event_id UUID DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents.endpoints(agent_id),
    event_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'low',
    title VARCHAR(500) NOT NULL,
    description TEXT,
    event_data JSONB NOT NULL,
    raw_data JSONB,
    source_ip TEXT,
    process_name VARCHAR(255),
    file_path TEXT,
    user_name VARCHAR(255),
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    -- Primary key must include partition key
    PRIMARY KEY (event_id, occurred_at),
    -- Indexes for efficient querying
    CONSTRAINT valid_severity CHECK (severity IN ('low', 'medium', 'high', 'critical'))
) PARTITION BY RANGE (occurred_at);

-- Create partitions for current and future months
CREATE TABLE threats.security_events_2025_08 PARTITION OF threats.security_events
    FOR VALUES FROM ('2025-08-01') TO ('2025-09-01');

CREATE TABLE threats.security_events_2025_09 PARTITION OF threats.security_events
    FOR VALUES FROM ('2025-09-01') TO ('2025-10-01');

CREATE TABLE threats.security_events_2025_10 PARTITION OF threats.security_events
    FOR VALUES FROM ('2025-10-01') TO ('2025-11-01');

-- Detection rules table
CREATE TABLE threats.detection_rules (
    rule_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    rule_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'medium',
    conditions JSONB NOT NULL,
    actions JSONB NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_by UUID REFERENCES users.users(user_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    CONSTRAINT valid_rule_severity CHECK (severity IN ('low', 'medium', 'high', 'critical'))
);

-- Threat alerts table for processed detections
CREATE TABLE threats.alerts (
    alert_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL,
    rule_id UUID REFERENCES threats.detection_rules(rule_id),
    agent_id UUID NOT NULL REFERENCES agents.endpoints(agent_id),
    alert_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'open',
    assigned_to UUID REFERENCES users.users(user_id),
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    CONSTRAINT valid_alert_severity CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT valid_alert_status CHECK (status IN ('open', 'investigating', 'resolved', 'false_positive'))
);

-- Agent commands table for remote command execution
CREATE TABLE threats.agent_commands (
    command_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents.endpoints(agent_id),
    issued_by UUID NOT NULL REFERENCES users.users(user_id),
    command_type VARCHAR(100) NOT NULL,
    command_data JSONB NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    result JSONB,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    executed_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    
    CONSTRAINT valid_command_status CHECK (status IN ('pending', 'sent', 'executing', 'completed', 'failed', 'timeout'))
);

-- System metrics table for agent performance monitoring
CREATE TABLE threats.system_metrics (
    metric_id UUID DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents.endpoints(agent_id),
    metric_type VARCHAR(50) NOT NULL,
    metric_data JSONB NOT NULL,
    collected_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    -- Primary key must include partition key
    PRIMARY KEY (metric_id, collected_at)
) PARTITION BY RANGE (collected_at);

-- Create metrics partitions
CREATE TABLE threats.system_metrics_2025_08 PARTITION OF threats.system_metrics
    FOR VALUES FROM ('2025-08-01') TO ('2025-09-01');

CREATE TABLE threats.system_metrics_2025_09 PARTITION OF threats.system_metrics
    FOR VALUES FROM ('2025-09-01') TO ('2025-10-01');

-- Indexes for performance
CREATE INDEX idx_security_events_agent_time ON threats.security_events (agent_id, occurred_at);
CREATE INDEX idx_security_events_severity ON threats.security_events (severity, occurred_at);
CREATE INDEX idx_security_events_type ON threats.security_events (event_type, occurred_at);
CREATE INDEX idx_security_events_search ON threats.security_events USING gin (event_data);

CREATE INDEX idx_alerts_agent_status ON threats.alerts (agent_id, status, created_at);
CREATE INDEX idx_alerts_severity ON threats.alerts (severity, created_at);
CREATE INDEX idx_alerts_assigned ON threats.alerts (assigned_to, status);

CREATE INDEX idx_commands_agent_status ON threats.agent_commands (agent_id, status, issued_at);
CREATE INDEX idx_commands_user ON threats.agent_commands (issued_by, issued_at);

CREATE INDEX idx_metrics_agent_type ON threats.system_metrics (agent_id, metric_type, collected_at);

-- Update trigger for updated_at columns
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_detection_rules_updated_at 
    BEFORE UPDATE ON threats.detection_rules 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_alerts_updated_at 
    BEFORE UPDATE ON threats.alerts 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default detection rules
INSERT INTO threats.detection_rules (name, description, rule_type, severity, conditions, actions) VALUES
('Suspicious Process Execution', 'Detects execution of processes from temporary directories', 'process', 'high', 
 '{"process_path": ["%temp%", "%tmp%", "C:\\\\Windows\\\\Temp"], "extensions": [".exe", ".scr", ".bat"]}', 
 '["alert", "log"]'),

('Multiple Failed Logins', 'Detects multiple failed login attempts within 5 minutes', 'authentication', 'medium',
 '{"event_type": "user_login", "success": false, "count": 5, "timeframe": 300}',
 '["alert", "log"]'),

('Critical System File Modification', 'Detects modifications to critical system files', 'file', 'critical',
 '{"file_paths": ["C:\\\\Windows\\\\System32\\\\", "C:\\\\Windows\\\\SysWOW64\\\\"], "operations": ["write", "delete"]}',
 '["alert", "quarantine"]'),

('Suspicious Network Connection', 'Detects connections to known malicious IP ranges', 'network', 'high',
 '{"remote_ips": ["tor_exit_nodes", "malware_c2", "suspicious_ranges"]}',
 '["alert", "block", "log"]'),

('Registry Persistence Modification', 'Detects modifications to common persistence registry keys', 'registry', 'high',
 '{"registry_keys": ["HKLM\\\\Software\\\\Microsoft\\\\Windows\\\\CurrentVersion\\\\Run", "HKCU\\\\Software\\\\Microsoft\\\\Windows\\\\CurrentVersion\\\\Run"]}',
 '["alert", "log"]');

-- Grant permissions
GRANT USAGE ON SCHEMA threats TO secureguard;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA threats TO secureguard;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA threats TO secureguard;