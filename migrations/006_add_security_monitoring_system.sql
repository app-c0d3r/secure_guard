-- migrations/006_add_security_monitoring_system.sql
-- Comprehensive security monitoring and user isolation system

-- Create role for application with limited permissions (before it's used)
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'app_user') THEN
        CREATE ROLE app_user;
    END IF;
END
$$;

-- Security Incidents Table - Track all security events
CREATE TABLE security_incidents (
    incident_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Incident Classification
    incident_type VARCHAR(50) NOT NULL, -- agent_tampering, unauthorized_access, suspicious_activity, data_breach, etc.
    severity VARCHAR(20) NOT NULL, -- low, medium, high, critical
    status VARCHAR(20) NOT NULL DEFAULT 'open', -- open, investigating, resolved, false_positive
    
    -- Affected Resources
    user_id UUID REFERENCES users.users(user_id),
    agent_id UUID REFERENCES agents.endpoints(agent_id),
    affected_resource VARCHAR(200), -- IP, endpoint, API key, etc.
    
    -- Incident Details
    title VARCHAR(200) NOT NULL,
    description TEXT NOT NULL,
    evidence JSONB, -- Structured evidence data
    indicators JSONB, -- IoCs (Indicators of Compromise)
    
    -- Detection Information
    detection_source VARCHAR(50) NOT NULL, -- agent, api, database, network, manual
    detection_rule VARCHAR(100),
    detection_confidence DECIMAL(3,2) DEFAULT 0.50, -- 0.0 to 1.0
    
    -- Geolocation & Network
    source_ip VARCHAR(45),
    source_country VARCHAR(2),
    user_agent TEXT,
    
    -- Timing
    first_detected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    
    -- Response
    assigned_to UUID REFERENCES users.users(user_id), -- Analyst assigned
    response_actions TEXT[],
    resolution_notes TEXT,
    
    -- Notifications
    user_notified_at TIMESTAMPTZ,
    admin_notified_at TIMESTAMPTZ,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Agent Tampering Detection - Specific to agent protection
CREATE TABLE agent_tampering_events (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents.endpoints(agent_id) ON DELETE CASCADE,
    incident_id UUID REFERENCES security_incidents(incident_id),
    
    -- Tampering Type
    tampering_type VARCHAR(50) NOT NULL, 
    -- agent_shutdown, service_stop, file_deletion, registry_modification, 
    -- process_kill, firewall_block, uninstall_attempt, config_tampering
    
    -- Detection Details
    detected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    detection_method VARCHAR(50), -- self_monitor, heartbeat_timeout, file_monitor, registry_monitor
    
    -- Event Context
    process_name VARCHAR(255),
    process_id INTEGER,
    process_user VARCHAR(100),
    command_line TEXT,
    parent_process VARCHAR(255),
    
    -- System State
    system_metrics JSONB,
    running_processes JSONB,
    network_connections JSONB,
    
    -- Evidence Collection
    memory_dump_path VARCHAR(500),
    forensic_snapshot JSONB,
    
    -- Response Actions
    protection_action VARCHAR(50), -- alert_only, restart_service, block_process, quarantine, full_response
    action_successful BOOLEAN,
    action_details TEXT,
    
    -- Recovery
    recovery_attempted BOOLEAN DEFAULT FALSE,
    recovery_successful BOOLEAN,
    recovery_time_seconds INTEGER,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- API Security Events - Track API abuse and attacks
CREATE TABLE api_security_events (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id UUID REFERENCES security_incidents(incident_id),
    
    -- Request Information
    user_id UUID REFERENCES users.users(user_id),
    api_key_id UUID REFERENCES users.api_keys(key_id),
    endpoint VARCHAR(200) NOT NULL,
    method VARCHAR(10) NOT NULL,
    
    -- Attack Classification
    attack_type VARCHAR(50) NOT NULL,
    -- rate_limit_exceeded, brute_force, sql_injection, unauthorized_access,
    -- data_exfiltration, privilege_escalation, suspicious_pattern
    
    -- Request Details
    request_headers JSONB,
    request_body JSONB,
    query_parameters JSONB,
    response_status INTEGER,
    response_size INTEGER,
    
    -- Network Context
    source_ip VARCHAR(45) NOT NULL,
    source_country VARCHAR(2),
    user_agent TEXT,
    x_forwarded_for VARCHAR(100),
    
    -- Detection Metrics
    requests_per_minute INTEGER,
    unique_endpoints_accessed INTEGER,
    suspicious_patterns TEXT[],
    geolocation_anomaly BOOLEAN DEFAULT FALSE,
    
    -- Response
    action_taken VARCHAR(50), -- blocked, rate_limited, flagged, allowed
    blocked_until TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- User Access Violations - Track unauthorized access attempts
CREATE TABLE user_access_violations (
    violation_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id UUID REFERENCES security_incidents(incident_id),
    
    -- Violation Details
    user_id UUID REFERENCES users.users(user_id),
    target_user_id UUID REFERENCES users.users(user_id), -- User they tried to access
    target_resource_type VARCHAR(50), -- agent, api_key, command, dashboard, etc.
    target_resource_id UUID,
    
    -- Access Details
    access_type VARCHAR(50) NOT NULL, -- read, write, delete, execute, admin
    violation_type VARCHAR(50) NOT NULL, 
    -- cross_user_access, privilege_escalation, resource_enumeration, 
    -- unauthorized_command, subscription_bypass
    
    -- Request Context
    endpoint VARCHAR(200),
    method VARCHAR(10),
    source_ip VARCHAR(45),
    user_agent TEXT,
    session_id VARCHAR(100),
    
    -- Detection
    detection_rule VARCHAR(100),
    risk_score INTEGER, -- 0-100
    
    -- Response
    access_denied BOOLEAN DEFAULT TRUE,
    user_suspended BOOLEAN DEFAULT FALSE,
    admin_notified BOOLEAN DEFAULT FALSE,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Backend Intrusion Monitoring - Monitor our own systems
CREATE TABLE backend_monitoring (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- System Component
    component VARCHAR(50) NOT NULL, -- database, api, frontend, load_balancer, etc.
    server_hostname VARCHAR(100),
    
    -- Event Type
    event_type VARCHAR(50) NOT NULL,
    -- unusual_query, high_cpu, memory_spike, disk_full, network_anomaly,
    -- failed_login, privilege_escalation, file_modification, process_anomaly
    
    -- Metrics
    cpu_usage DECIMAL(5,2),
    memory_usage DECIMAL(5,2),
    disk_usage DECIMAL(5,2),
    network_in_mbps DECIMAL(10,2),
    network_out_mbps DECIMAL(10,2),
    
    -- Database Specific
    active_connections INTEGER,
    slow_queries INTEGER,
    failed_queries INTEGER,
    suspicious_queries TEXT[],
    
    -- Application Metrics  
    response_time_ms INTEGER,
    error_rate DECIMAL(5,4),
    active_sessions INTEGER,
    
    -- Security Indicators
    failed_auth_attempts INTEGER,
    privilege_escalation_attempts INTEGER,
    unusual_access_patterns BOOLEAN DEFAULT FALSE,
    
    -- Alerting
    alert_threshold_exceeded BOOLEAN DEFAULT FALSE,
    alert_sent BOOLEAN DEFAULT FALSE,
    alert_level VARCHAR(20), -- info, warning, critical
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Security Notifications - Track notifications sent
CREATE TABLE security_notifications (
    notification_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id UUID REFERENCES security_incidents(incident_id),
    
    -- Recipient Information
    user_id UUID REFERENCES users.users(user_id),
    notification_type VARCHAR(50) NOT NULL, -- email, sms, push, dashboard, webhook
    recipient VARCHAR(200) NOT NULL, -- email address, phone number, etc.
    
    -- Message Details
    subject VARCHAR(200),
    message TEXT NOT NULL,
    priority VARCHAR(20) DEFAULT 'medium', -- low, medium, high, urgent
    
    -- Delivery Status
    status VARCHAR(20) DEFAULT 'pending', -- pending, sent, delivered, failed, bounced
    sent_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    read_at TIMESTAMPTZ,
    
    -- Response Tracking
    user_acknowledged BOOLEAN DEFAULT FALSE,
    user_response TEXT,
    response_at TIMESTAMPTZ,
    
    -- Retry Logic
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    next_retry_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Fix API Key Limits - Update subscription plans for proper API key logic
UPDATE subscriptions.plans SET
    max_api_keys = CASE 
        WHEN plan_slug = 'free' THEN 2  -- 1 active + 1 backup for device replacement
        WHEN plan_slug = 'starter' THEN 5  -- Multiple devices need multiple keys  
        WHEN plan_slug = 'professional' THEN 15 -- More devices + integration keys
        WHEN plan_slug = 'enterprise' THEN -1 -- Unlimited
        ELSE max_api_keys
    END;

-- Add user isolation constraints to prevent cross-user data access
-- Update agents table to enforce user isolation
CREATE OR REPLACE FUNCTION check_user_agent_access()
RETURNS TRIGGER AS $$
BEGIN
    -- Ensure user can only access their own agents
    IF NOT EXISTS (
        SELECT 1 FROM agents.endpoints 
        WHERE agent_id = NEW.agent_id 
        AND user_id = (
            SELECT user_id FROM users.users 
            WHERE user_id = current_setting('app.current_user_id', true)::UUID
        )
    ) THEN
        RAISE EXCEPTION 'Access denied: User can only access their own agents';
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Row Level Security (RLS) Policies
ALTER TABLE agents.endpoints ENABLE ROW LEVEL SECURITY;
ALTER TABLE users.api_keys ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent_commands ENABLE ROW LEVEL SECURITY;
ALTER TABLE security_incidents ENABLE ROW LEVEL SECURITY;

-- Policy: Users can only see their own agents
CREATE POLICY user_agents_isolation ON agents.endpoints
    FOR ALL TO app_user
    USING (user_id = current_setting('app.current_user_id', true)::UUID);

-- Policy: Users can only see their own API keys
CREATE POLICY user_api_keys_isolation ON users.api_keys
    FOR ALL TO app_user  
    USING (user_id = current_setting('app.current_user_id', true)::UUID);

-- Policy: Users can only see commands for their agents
CREATE POLICY user_agent_commands_isolation ON agent_commands
    FOR ALL TO app_user
    USING (agent_id IN (
        SELECT agent_id FROM agents.endpoints 
        WHERE user_id = current_setting('app.current_user_id', true)::UUID
    ));

-- Policy: Users can only see their own security incidents  
CREATE POLICY user_security_incidents_isolation ON security_incidents
    FOR ALL TO app_user
    USING (user_id = current_setting('app.current_user_id', true)::UUID);

-- Grant permissions to existing app_user role
GRANT USAGE ON SCHEMA users, agents, subscriptions TO app_user;
GRANT SELECT, INSERT, UPDATE ON users.users, users.api_keys TO app_user;
GRANT SELECT, INSERT, UPDATE ON agents.endpoints, agent_commands TO app_user;
GRANT SELECT ON subscriptions.plans, subscriptions.user_subscriptions TO app_user;
GRANT INSERT ON security_incidents, security_notifications TO app_user;

-- Create indexes for security monitoring
CREATE INDEX idx_security_incidents_user_id ON security_incidents(user_id);
CREATE INDEX idx_security_incidents_type_severity ON security_incidents(incident_type, severity);
CREATE INDEX idx_security_incidents_detection_time ON security_incidents(first_detected_at);
CREATE INDEX idx_security_incidents_status ON security_incidents(status);

CREATE INDEX idx_agent_tampering_agent_id ON agent_tampering_events(agent_id);
CREATE INDEX idx_agent_tampering_type ON agent_tampering_events(tampering_type);
CREATE INDEX idx_agent_tampering_detected ON agent_tampering_events(detected_at);

CREATE INDEX idx_api_security_user_id ON api_security_events(user_id);
CREATE INDEX idx_api_security_ip ON api_security_events(source_ip);
CREATE INDEX idx_api_security_attack_type ON api_security_events(attack_type);
CREATE INDEX idx_api_security_created ON api_security_events(created_at);

CREATE INDEX idx_user_violations_user_id ON user_access_violations(user_id);
CREATE INDEX idx_user_violations_target ON user_access_violations(target_user_id);
CREATE INDEX idx_user_violations_type ON user_access_violations(violation_type);

CREATE INDEX idx_backend_monitoring_component ON backend_monitoring(component);
CREATE INDEX idx_backend_monitoring_event_type ON backend_monitoring(event_type);  
CREATE INDEX idx_backend_monitoring_created ON backend_monitoring(created_at);
CREATE INDEX idx_backend_monitoring_alerts ON backend_monitoring(alert_threshold_exceeded) WHERE alert_threshold_exceeded = TRUE;

-- Create views for security dashboards
CREATE VIEW security_incidents_summary AS
SELECT 
    incident_type,
    severity,
    status,
    COUNT(*) as incident_count,
    COUNT(CASE WHEN first_detected_at >= now() - INTERVAL '24 hours' THEN 1 END) as incidents_24h,
    COUNT(CASE WHEN first_detected_at >= now() - INTERVAL '7 days' THEN 1 END) as incidents_7d,
    AVG(EXTRACT(EPOCH FROM (COALESCE(resolved_at, now()) - first_detected_at))/3600) as avg_resolution_hours
FROM security_incidents
GROUP BY incident_type, severity, status;

CREATE VIEW user_security_dashboard AS
SELECT 
    u.user_id,
    u.username,
    COUNT(DISTINCT ae.agent_id) as total_agents,
    COUNT(DISTINCT si.incident_id) as total_incidents,
    COUNT(DISTINCT CASE WHEN si.first_detected_at >= now() - INTERVAL '24 hours' THEN si.incident_id END) as incidents_24h,
    COUNT(DISTINCT CASE WHEN si.severity = 'critical' THEN si.incident_id END) as critical_incidents,
    COUNT(DISTINCT ate.event_id) as tampering_events,
    MAX(si.first_detected_at) as last_incident_time
FROM users.users u
LEFT JOIN agents.endpoints ae ON u.user_id = ae.user_id
LEFT JOIN security_incidents si ON u.user_id = si.user_id
LEFT JOIN agent_tampering_events ate ON ae.agent_id = ate.agent_id
GROUP BY u.user_id, u.username;

-- Function to automatically create security incident for agent tampering
CREATE OR REPLACE FUNCTION create_tampering_incident()
RETURNS TRIGGER AS $$
DECLARE
    incident_uuid UUID;
    agent_info RECORD;
    user_info RECORD;
BEGIN
    -- Get agent and user information
    SELECT ae.agent_id, ae.user_id, ae.device_name, u.username, u.email
    INTO agent_info
    FROM agents.endpoints ae
    JOIN users.users u ON ae.user_id = u.user_id
    WHERE ae.agent_id = NEW.agent_id;
    
    -- Create security incident
    INSERT INTO security_incidents (
        incident_type, severity, title, description, evidence,
        user_id, agent_id, affected_resource, detection_source,
        detection_rule, source_ip
    ) VALUES (
        'agent_tampering',
        CASE 
            WHEN NEW.tampering_type IN ('agent_shutdown', 'service_stop', 'uninstall_attempt') THEN 'high'
            WHEN NEW.tampering_type IN ('file_deletion', 'config_tampering') THEN 'medium'
            ELSE 'medium'
        END,
        format('Agent Tampering Detected: %s', NEW.tampering_type),
        format('Tampering attempt detected on agent %s (%s). Type: %s. Process: %s (PID: %s) User: %s',
            agent_info.device_name, agent_info.agent_id, NEW.tampering_type, 
            NEW.process_name, NEW.process_id, NEW.process_user),
        jsonb_build_object(
            'tampering_type', NEW.tampering_type,
            'process_name', NEW.process_name,
            'process_id', NEW.process_id,
            'command_line', NEW.command_line,
            'detection_method', NEW.detection_method,
            'system_metrics', NEW.system_metrics
        ),
        agent_info.user_id,
        agent_info.agent_id,
        format('Agent-%s', agent_info.device_name),
        'agent',
        format('tampering_%s', NEW.tampering_type),
        NULL
    ) RETURNING incident_id INTO incident_uuid;
    
    -- Update the tampering event with incident reference
    NEW.incident_id := incident_uuid;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic incident creation
CREATE TRIGGER trigger_create_tampering_incident
    BEFORE INSERT ON agent_tampering_events
    FOR EACH ROW
    EXECUTE FUNCTION create_tampering_incident();

-- Function to send security notifications
CREATE OR REPLACE FUNCTION send_security_notification(
    p_incident_id UUID,
    p_user_id UUID,
    p_notification_type VARCHAR(50),
    p_priority VARCHAR(20) DEFAULT 'medium'
)
RETURNS UUID AS $$
DECLARE
    notification_uuid UUID;
    incident_info RECORD;
    user_info RECORD;
    message_text TEXT;
    subject_text VARCHAR(200);
BEGIN
    -- Get incident details
    SELECT * INTO incident_info FROM security_incidents WHERE incident_id = p_incident_id;
    SELECT * INTO user_info FROM users.users WHERE user_id = p_user_id;
    
    -- Build notification message
    subject_text := format('[SecureGuard Alert] %s', incident_info.title);
    message_text := format(
        'Security Alert for %s\n\n' ||
        'Incident Type: %s\n' ||
        'Severity: %s\n' ||
        'Description: %s\n' ||
        'Detected: %s\n\n' ||
        'Please log into your SecureGuard dashboard to review this incident and take appropriate action.\n\n' ||
        'If this was not authorized by you, please contact support immediately.',
        user_info.username,
        incident_info.incident_type,
        incident_info.severity,
        incident_info.description,
        incident_info.first_detected_at
    );
    
    -- Create notification record
    INSERT INTO security_notifications (
        incident_id, user_id, notification_type, subject, message, priority,
        recipient, status
    ) VALUES (
        p_incident_id, p_user_id, p_notification_type, subject_text, message_text, p_priority,
        user_info.email, 'pending'
    ) RETURNING notification_id INTO notification_uuid;
    
    -- Update incident notification timestamps
    UPDATE security_incidents 
    SET user_notified_at = now() 
    WHERE incident_id = p_incident_id;
    
    RETURN notification_uuid;
END;
$$ LANGUAGE plpgsql;