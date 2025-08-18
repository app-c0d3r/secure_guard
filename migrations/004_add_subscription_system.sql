-- migrations/004_add_subscription_system.sql
-- Subscription-based device and feature limitations system

-- Create subscriptions schema
CREATE SCHEMA IF NOT EXISTS subscriptions;

-- Subscription Plans Definition
CREATE TABLE subscriptions.plans (
    plan_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_name VARCHAR(50) UNIQUE NOT NULL,
    plan_slug VARCHAR(20) UNIQUE NOT NULL, -- free, starter, pro, enterprise
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- Device Limits
    max_devices INTEGER NOT NULL DEFAULT 1,
    max_api_keys INTEGER NOT NULL DEFAULT 2,
    
    -- Feature Limits
    real_time_monitoring BOOLEAN NOT NULL DEFAULT FALSE,
    advanced_threat_detection BOOLEAN NOT NULL DEFAULT FALSE,
    custom_rules BOOLEAN NOT NULL DEFAULT FALSE,
    api_access BOOLEAN NOT NULL DEFAULT FALSE,
    priority_support BOOLEAN NOT NULL DEFAULT FALSE,
    audit_logs BOOLEAN NOT NULL DEFAULT FALSE,
    integrations_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Advanced Features
    vulnerability_scanning BOOLEAN NOT NULL DEFAULT FALSE,
    compliance_reporting BOOLEAN NOT NULL DEFAULT FALSE,
    remote_response BOOLEAN NOT NULL DEFAULT FALSE,
    custom_dashboards BOOLEAN NOT NULL DEFAULT FALSE,
    bulk_operations BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Data Retention
    log_retention_days INTEGER NOT NULL DEFAULT 7,
    alert_history_days INTEGER NOT NULL DEFAULT 30,
    
    -- Pricing
    monthly_price_cents INTEGER NOT NULL DEFAULT 0,
    yearly_price_cents INTEGER NOT NULL DEFAULT 0,
    
    -- Metadata
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- User Subscriptions
CREATE TABLE subscriptions.user_subscriptions (
    subscription_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users.users(user_id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES subscriptions.plans(plan_id),
    
    -- Subscription Status
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, cancelled, expired, trial
    
    -- Billing
    billing_cycle VARCHAR(10) NOT NULL DEFAULT 'monthly', -- monthly, yearly
    current_period_start TIMESTAMPTZ NOT NULL DEFAULT now(),
    current_period_end TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '30 days'),
    
    -- Trial Information
    trial_start TIMESTAMPTZ,
    trial_end TIMESTAMPTZ,
    is_trial BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Payment
    stripe_subscription_id VARCHAR(100), -- For Stripe integration
    last_payment_at TIMESTAMPTZ,
    next_billing_date TIMESTAMPTZ,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    cancelled_at TIMESTAMPTZ,
    
    UNIQUE(user_id) -- One active subscription per user
);

-- Subscription Usage Tracking
CREATE TABLE subscriptions.usage_tracking (
    usage_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users.users(user_id),
    subscription_id UUID NOT NULL REFERENCES subscriptions.user_subscriptions(subscription_id),
    
    -- Current Usage
    current_devices INTEGER NOT NULL DEFAULT 0,
    current_api_keys INTEGER NOT NULL DEFAULT 0,
    
    -- Monthly Metrics
    api_calls_this_month INTEGER NOT NULL DEFAULT 0,
    alerts_this_month INTEGER NOT NULL DEFAULT 0,
    data_processed_mb INTEGER NOT NULL DEFAULT 0,
    
    -- Limits Hit
    device_limit_hit_count INTEGER NOT NULL DEFAULT 0,
    api_key_limit_hit_count INTEGER NOT NULL DEFAULT 0,
    feature_access_denied_count INTEGER NOT NULL DEFAULT 0,
    
    -- Reset Tracking
    last_reset TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Insert Default Plans
INSERT INTO subscriptions.plans (
    plan_slug, plan_name, display_name, description, 
    max_devices, max_api_keys,
    real_time_monitoring, advanced_threat_detection, custom_rules, 
    api_access, priority_support, audit_logs, integrations_enabled,
    vulnerability_scanning, compliance_reporting, remote_response,
    custom_dashboards, bulk_operations,
    log_retention_days, alert_history_days,
    monthly_price_cents, yearly_price_cents, sort_order
) VALUES 
(
    'free', 'Free', 'SecureGuard Free', 
    'Perfect for personal use. Monitor one device with basic security features.',
    1, 2, -- 1 device, 2 API keys
    FALSE, FALSE, FALSE, FALSE, FALSE, FALSE, FALSE, -- Basic features only
    FALSE, FALSE, FALSE, FALSE, FALSE, -- No advanced features
    7, 30, -- 7 days logs, 30 days alerts
    0, 0, 1 -- Free
),
(
    'starter', 'Starter', 'SecureGuard Starter',
    'Great for small teams. Monitor up to 5 devices with real-time alerts.',
    5, 10, -- 5 devices, 10 API keys
    TRUE, FALSE, FALSE, TRUE, FALSE, TRUE, FALSE, -- Some features
    FALSE, FALSE, FALSE, FALSE, FALSE, -- No advanced features
    30, 90, -- 30 days logs, 90 days alerts
    999, 9999, 2 -- $9.99/month, $99.99/year
),
(
    'professional', 'Professional', 'SecureGuard Professional',
    'Perfect for growing businesses. Advanced threat detection and custom rules.',
    25, 50, -- 25 devices, 50 API keys
    TRUE, TRUE, TRUE, TRUE, TRUE, TRUE, TRUE, -- Most features
    TRUE, FALSE, TRUE, TRUE, FALSE, -- Some advanced features
    90, 180, -- 90 days logs, 180 days alerts
    2999, 29999, 3 -- $29.99/month, $299.99/year
),
(
    'enterprise', 'Enterprise', 'SecureGuard Enterprise',
    'For large organizations. Unlimited devices and all advanced features.',
    -1, -1, -- Unlimited devices and API keys (-1 = unlimited)
    TRUE, TRUE, TRUE, TRUE, TRUE, TRUE, TRUE, -- All features
    TRUE, TRUE, TRUE, TRUE, TRUE, -- All advanced features
    365, 730, -- 1 year logs, 2 years alerts
    9999, 99999, 4 -- $99.99/month, $999.99/year
);

-- Create indexes
CREATE INDEX idx_user_subscriptions_user_id ON subscriptions.user_subscriptions(user_id);
CREATE INDEX idx_user_subscriptions_status ON subscriptions.user_subscriptions(status);
CREATE INDEX idx_user_subscriptions_billing_date ON subscriptions.user_subscriptions(next_billing_date);

CREATE INDEX idx_usage_tracking_user_id ON subscriptions.usage_tracking(user_id);
CREATE INDEX idx_usage_tracking_subscription_id ON subscriptions.usage_tracking(subscription_id);

CREATE INDEX idx_plans_active ON subscriptions.plans(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_plans_sort ON subscriptions.plans(sort_order);

-- Initialize all existing users with Free plan
INSERT INTO subscriptions.user_subscriptions (user_id, plan_id, status)
SELECT 
    u.user_id, 
    p.plan_id, 
    'active'
FROM users.users u
CROSS JOIN subscriptions.plans p 
WHERE p.plan_slug = 'free'
AND NOT EXISTS (
    SELECT 1 FROM subscriptions.user_subscriptions s 
    WHERE s.user_id = u.user_id
);

-- Initialize usage tracking for all users
INSERT INTO subscriptions.usage_tracking (user_id, subscription_id, current_devices, current_api_keys)
SELECT 
    us.user_id,
    us.subscription_id,
    COALESCE((SELECT COUNT(*) FROM agents.endpoints WHERE user_id = us.user_id), 0),
    COALESCE((SELECT COUNT(*) FROM users.api_keys WHERE user_id = us.user_id AND is_active = TRUE), 0)
FROM subscriptions.user_subscriptions us
WHERE NOT EXISTS (
    SELECT 1 FROM subscriptions.usage_tracking ut 
    WHERE ut.user_id = us.user_id
);

-- Helper Views
CREATE VIEW subscriptions.user_plan_details AS
SELECT 
    u.user_id,
    u.username,
    u.email,
    us.subscription_id,
    us.status as subscription_status,
    us.is_trial,
    us.current_period_end,
    p.plan_slug,
    p.plan_name,
    p.display_name,
    p.max_devices,
    p.max_api_keys,
    p.real_time_monitoring,
    p.advanced_threat_detection,
    p.custom_rules,
    p.api_access,
    p.priority_support,
    p.audit_logs,
    p.integrations_enabled,
    p.vulnerability_scanning,
    p.compliance_reporting,
    p.remote_response,
    p.custom_dashboards,
    p.bulk_operations,
    p.log_retention_days,
    p.alert_history_days,
    ut.current_devices,
    ut.current_api_keys
FROM users.users u
JOIN subscriptions.user_subscriptions us ON u.user_id = us.user_id
JOIN subscriptions.plans p ON us.plan_id = p.plan_id
LEFT JOIN subscriptions.usage_tracking ut ON u.user_id = ut.user_id
WHERE us.status = 'active';

CREATE VIEW subscriptions.plan_comparison AS
SELECT 
    plan_slug,
    display_name,
    monthly_price_cents,
    yearly_price_cents,
    max_devices,
    max_api_keys,
    real_time_monitoring,
    advanced_threat_detection,
    custom_rules,
    api_access,
    priority_support,
    audit_logs,
    integrations_enabled,
    vulnerability_scanning,
    compliance_reporting,
    remote_response,
    custom_dashboards,
    bulk_operations,
    log_retention_days,
    alert_history_days,
    sort_order
FROM subscriptions.plans 
WHERE is_active = TRUE
ORDER BY sort_order;