use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use secureguard_shared::{SecureGuardError, Result};
use super::subscription_service::SubscriptionService;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentConfigResponse {
    pub agent_id: Uuid,
    pub config_version: u32,
    pub subscription: SubscriptionConfigSection,
    pub features: FeatureConfigSection,
    pub limits: LimitsConfigSection,
    pub monitoring: MonitoringConfigSection,
    pub security: SecurityConfigSection,
    pub logging: LoggingConfigSection,
    pub updates: UpdateConfigSection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionConfigSection {
    pub tier: String,
    pub plan_name: String,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub features_enabled: Vec<String>,
    pub auto_updates_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureConfigSection {
    pub enabled_features: Vec<String>,
    pub feature_configs: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LimitsConfigSection {
    pub max_file_scan_size: u64,
    pub max_concurrent_scans: u32,
    pub scan_frequency: u32,
    pub log_retention_days: u32,
    pub alert_history_days: u32,
    pub max_alerts_per_hour: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoringConfigSection {
    pub heartbeat_interval: u32,
    pub data_collection_interval: u32,
    pub metrics_retention_hours: u32,
    pub real_time_enabled: bool,
    pub performance_monitoring: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityConfigSection {
    pub encryption_enabled: bool,
    pub tls_version: String,
    pub certificate_validation: bool,
    pub command_validation_required: bool,
    pub audit_logging: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfigSection {
    pub level: String,
    pub file_path: String,
    pub max_size: String,
    pub max_files: u32,
    pub console_output: bool,
    pub remote_logging: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfigSection {
    pub auto_update: bool,
    pub update_channel: String,
    pub check_interval_hours: u32,
    pub maintenance_window_start: Option<String>,
    pub maintenance_window_end: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureConfiguration {
    pub feature_id: String,
    pub config: serde_json::Value,
    pub is_enabled: bool,
    pub auto_enable: bool,
}

pub struct ConfigService {
    pool: PgPool,
    subscription_service: SubscriptionService,
}

impl ConfigService {
    pub fn new(pool: PgPool) -> Self {
        let subscription_service = SubscriptionService::new(pool.clone());
        Self {
            pool,
            subscription_service,
        }
    }

    /// Get complete agent configuration based on subscription
    pub async fn get_agent_config(&self, agent_id: Uuid) -> Result<AgentConfigResponse> {
        // Get agent details
        let agent = sqlx::query!(
            "SELECT agent_id, user_id, device_name FROM agents.endpoints WHERE agent_id = $1",
            agent_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .ok_or_else(|| SecureGuardError::AgentNotFound)?;

        // Get user subscription
        let subscription = self.subscription_service.get_user_subscription(agent.user_id.unwrap())
            .await?
            .ok_or_else(|| SecureGuardError::NotFound("User subscription not found".to_string()))?;

        // Build configuration based on subscription tier
        let config = self.build_subscription_config(&subscription, agent_id).await?;

        Ok(config)
    }

    async fn build_subscription_config(
        &self, 
        subscription: &super::subscription_service::UserSubscription, 
        agent_id: Uuid
    ) -> Result<AgentConfigResponse> {
        
        // Get current config version
        let config_version = self.get_current_config_version(agent_id).await?;

        // Build subscription section
        let subscription_config = SubscriptionConfigSection {
            tier: subscription.plan.plan_slug.clone(),
            plan_name: subscription.plan.plan_name.clone(),
            expires_at: Some(subscription.current_period_end),
            features_enabled: self.get_enabled_features_for_plan(&subscription.plan).await?,
            auto_updates_enabled: self.get_auto_update_setting(&subscription.plan).await?,
        };

        // Build feature configuration
        let features_config = self.build_features_config(&subscription.plan).await?;

        // Build limits based on subscription
        let limits_config = self.build_limits_config(&subscription.plan).await?;

        // Build monitoring configuration
        let monitoring_config = self.build_monitoring_config(&subscription.plan).await?;

        // Build security configuration
        let security_config = self.build_security_config(&subscription.plan).await?;

        // Build logging configuration
        let logging_config = self.build_logging_config(&subscription.plan).await?;

        // Build update configuration
        let updates_config = self.build_updates_config(&subscription.plan).await?;

        Ok(AgentConfigResponse {
            agent_id,
            config_version,
            subscription: subscription_config,
            features: features_config,
            limits: limits_config,
            monitoring: monitoring_config,
            security: security_config,
            logging: logging_config,
            updates: updates_config,
        })
    }

    async fn get_enabled_features_for_plan(&self, plan: &super::subscription_service::SubscriptionPlan) -> Result<Vec<String>> {
        let mut features = Vec::new();

        // Always enabled features
        features.push("basic_monitoring".to_string());
        features.push("agent_management".to_string());

        // Starter+ features
        if plan.real_time_monitoring {
            features.push("real_time_monitoring".to_string());
        }
        if plan.api_access {
            features.push("api_access".to_string());
        }
        if plan.audit_logs {
            features.push("audit_logs".to_string());
        }

        // Professional+ features
        if plan.advanced_threat_detection {
            features.push("advanced_threat_detection".to_string());
        }
        if plan.custom_rules {
            features.push("custom_rules".to_string());
        }
        if plan.vulnerability_scanning {
            features.push("vulnerability_scanning".to_string());
        }
        if plan.remote_response {
            features.push("remote_commands".to_string());
        }
        if plan.integrations_enabled {
            features.push("integrations".to_string());
        }

        // Enterprise features
        if plan.compliance_reporting {
            features.push("compliance_reporting".to_string());
        }
        if plan.bulk_operations {
            features.push("bulk_operations".to_string());
        }
        if plan.custom_dashboards {
            features.push("custom_dashboards".to_string());
        }

        Ok(features)
    }

    async fn build_features_config(&self, plan: &super::subscription_service::SubscriptionPlan) -> Result<FeatureConfigSection> {
        let enabled_features = self.get_enabled_features_for_plan(plan).await?;
        let mut feature_configs = HashMap::new();

        // Real-time monitoring config
        if plan.real_time_monitoring {
            feature_configs.insert("real_time_monitoring".to_string(), serde_json::json!({
                "scan_interval": 30,
                "alert_threshold": "medium",
                "file_monitoring": true,
                "process_monitoring": true,
                "network_monitoring": plan.plan_slug != "starter"
            }));
        }

        // Advanced threat detection config
        if plan.advanced_threat_detection {
            feature_configs.insert("advanced_threat_detection".to_string(), serde_json::json!({
                "ai_model": match plan.plan_slug.as_str() {
                    "professional" => "standard",
                    "enterprise" => "advanced",
                    _ => "basic"
                },
                "sensitivity": "balanced",
                "behavioral_analysis": plan.plan_slug == "enterprise",
                "cloud_analysis": plan.plan_slug != "starter"
            }));
        }

        // Custom rules config
        if plan.custom_rules {
            feature_configs.insert("custom_rules".to_string(), serde_json::json!({
                "max_rules": match plan.plan_slug.as_str() {
                    "professional" => 50,
                    "enterprise" => -1, // unlimited
                    _ => 10
                },
                "rule_complexity": match plan.plan_slug.as_str() {
                    "enterprise" => "advanced",
                    _ => "standard"
                }
            }));
        }

        // Vulnerability scanning config
        if plan.vulnerability_scanning {
            feature_configs.insert("vulnerability_scanning".to_string(), serde_json::json!({
                "scan_frequency": match plan.plan_slug.as_str() {
                    "professional" => "daily",
                    "enterprise" => "real_time",
                    _ => "weekly"
                },
                "cve_database_updates": plan.plan_slug == "enterprise",
                "custom_vulnerability_rules": plan.plan_slug == "enterprise"
            }));
        }

        // Remote commands config
        if plan.remote_response {
            feature_configs.insert("remote_commands".to_string(), serde_json::json!({
                "file_operations": true,
                "system_commands": plan.plan_slug == "enterprise",
                "forensic_collection": plan.plan_slug == "enterprise",
                "command_timeout": 300,
                "audit_all_commands": true
            }));
        }

        Ok(FeatureConfigSection {
            enabled_features,
            feature_configs,
        })
    }

    async fn build_limits_config(&self, plan: &super::subscription_service::SubscriptionPlan) -> Result<LimitsConfigSection> {
        Ok(LimitsConfigSection {
            max_file_scan_size: match plan.plan_slug.as_str() {
                "free" => 10 * 1024 * 1024,      // 10MB
                "starter" => 50 * 1024 * 1024,   // 50MB
                "professional" => 200 * 1024 * 1024, // 200MB
                "enterprise" => 1024 * 1024 * 1024,  // 1GB
                _ => 10 * 1024 * 1024,
            },
            max_concurrent_scans: match plan.plan_slug.as_str() {
                "free" => 1,
                "starter" => 3,
                "professional" => 10,
                "enterprise" => 50,
                _ => 1,
            },
            scan_frequency: match plan.plan_slug.as_str() {
                "free" => 3600,      // 1 hour
                "starter" => 1800,   // 30 minutes
                "professional" => 300, // 5 minutes
                "enterprise" => 60,   // 1 minute
                _ => 3600,
            },
            log_retention_days: plan.log_retention_days as u32,
            alert_history_days: plan.alert_history_days as u32,
            max_alerts_per_hour: match plan.plan_slug.as_str() {
                "free" => 10,
                "starter" => 50,
                "professional" => 200,
                "enterprise" => 1000,
                _ => 10,
            },
        })
    }

    async fn build_monitoring_config(&self, plan: &super::subscription_service::SubscriptionPlan) -> Result<MonitoringConfigSection> {
        Ok(MonitoringConfigSection {
            heartbeat_interval: match plan.plan_slug.as_str() {
                "free" => 300,       // 5 minutes
                "starter" => 60,     // 1 minute
                "professional" => 30, // 30 seconds
                "enterprise" => 15,   // 15 seconds
                _ => 300,
            },
            data_collection_interval: match plan.plan_slug.as_str() {
                "free" => 1800,      // 30 minutes
                "starter" => 600,    // 10 minutes
                "professional" => 300, // 5 minutes
                "enterprise" => 60,   // 1 minute
                _ => 1800,
            },
            metrics_retention_hours: match plan.plan_slug.as_str() {
                "free" => 24,        // 1 day
                "starter" => 168,    // 1 week
                "professional" => 720, // 1 month
                "enterprise" => 8760,  // 1 year
                _ => 24,
            },
            real_time_enabled: plan.real_time_monitoring,
            performance_monitoring: plan.plan_slug != "free",
        })
    }

    async fn build_security_config(&self, plan: &super::subscription_service::SubscriptionPlan) -> Result<SecurityConfigSection> {
        Ok(SecurityConfigSection {
            encryption_enabled: true,
            tls_version: match plan.plan_slug.as_str() {
                "enterprise" => "1.3".to_string(),
                _ => "1.2".to_string(),
            },
            certificate_validation: true,
            command_validation_required: plan.remote_response,
            audit_logging: plan.audit_logs,
        })
    }

    async fn build_logging_config(&self, plan: &super::subscription_service::SubscriptionPlan) -> Result<LoggingConfigSection> {
        Ok(LoggingConfigSection {
            level: match plan.plan_slug.as_str() {
                "free" => "warn".to_string(),
                "starter" => "info".to_string(),
                "professional" => "debug".to_string(),
                "enterprise" => "trace".to_string(),
                _ => "warn".to_string(),
            },
            file_path: "C:\\Program Files\\SecureGuard\\Agent\\logs\\agent.log".to_string(),
            max_size: match plan.plan_slug.as_str() {
                "free" => "10MB".to_string(),
                "starter" => "50MB".to_string(),
                "professional" => "100MB".to_string(),
                "enterprise" => "500MB".to_string(),
                _ => "10MB".to_string(),
            },
            max_files: match plan.plan_slug.as_str() {
                "free" => 3,
                "starter" => 5,
                "professional" => 10,
                "enterprise" => 20,
                _ => 3,
            },
            console_output: false,
            remote_logging: plan.audit_logs,
        })
    }

    async fn build_updates_config(&self, plan: &super::subscription_service::SubscriptionPlan) -> Result<UpdateConfigSection> {
        Ok(UpdateConfigSection {
            auto_update: self.get_auto_update_setting(plan).await?,
            update_channel: match plan.plan_slug.as_str() {
                "enterprise" => "stable".to_string(),
                "professional" => "stable".to_string(),
                "starter" => "stable".to_string(),
                "free" => "stable".to_string(),
                _ => "stable".to_string(),
            },
            check_interval_hours: match plan.plan_slug.as_str() {
                "free" => 24,        // Daily
                "starter" => 12,     // Twice daily
                "professional" => 6, // 4 times daily
                "enterprise" => 1,   // Hourly
                _ => 24,
            },
            maintenance_window_start: match plan.plan_slug.as_str() {
                "enterprise" => None, // No maintenance window restrictions
                _ => Some("02:00".to_string()), // 2 AM local time
            },
            maintenance_window_end: match plan.plan_slug.as_str() {
                "enterprise" => None,
                _ => Some("04:00".to_string()), // 4 AM local time
            },
        })
    }

    async fn get_auto_update_setting(&self, plan: &super::subscription_service::SubscriptionPlan) -> Result<bool> {
        // Auto-updates enabled for paid plans
        Ok(match plan.plan_slug.as_str() {
            "free" => false,      // Manual updates for free tier
            "starter" => true,    // Auto-updates for paid tiers
            "professional" => true,
            "enterprise" => true,
            _ => false,
        })
    }

    async fn get_current_config_version(&self, agent_id: Uuid) -> Result<u32> {
        let version = sqlx::query!(
            "SELECT COALESCE(config_version, 1) as version FROM agents.endpoints WHERE agent_id = $1",
            agent_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .map(|row| row.version.unwrap_or(1) as u32)
        .unwrap_or(1);

        Ok(version)
    }

    /// Update agent config version when configuration changes
    pub async fn increment_config_version(&self, agent_id: Uuid) -> Result<u32> {
        let new_version = sqlx::query!(
            r#"
            UPDATE agents.endpoints 
            SET config_version = COALESCE(config_version, 0) + 1,
                updated_at = now()
            WHERE agent_id = $1
            RETURNING config_version
            "#,
            agent_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .config_version
        .unwrap_or(1) as u32;

        Ok(new_version)
    }

    /// Check if agent config needs updating
    pub async fn config_needs_update(&self, agent_id: Uuid, client_version: u32) -> Result<bool> {
        let server_version = self.get_current_config_version(agent_id).await?;
        Ok(server_version > client_version)
    }

    /// Get subscription info for registration
    pub async fn get_subscription_info(&self, agent_id: Uuid) -> Result<SubscriptionInfo> {
        let agent = sqlx::query!(
            "SELECT user_id FROM agents.endpoints WHERE agent_id = $1",
            agent_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .ok_or_else(|| SecureGuardError::AgentNotFound)?;

        let subscription = self.subscription_service.get_user_subscription(agent.user_id.unwrap())
            .await?
            .ok_or_else(|| SecureGuardError::NotFound("User subscription not found".to_string()))?;

        let enabled_features = self.get_enabled_features_for_plan(&subscription.plan).await?;
        let config_version = self.get_current_config_version(agent_id).await?;

        Ok(SubscriptionInfo {
            tier: subscription.plan.plan_slug,
            plan_name: subscription.plan.plan_name,
            enabled_features,
            config_version,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub tier: String,
    pub plan_name: String,
    pub enabled_features: Vec<String>,
    pub config_version: u32,
}