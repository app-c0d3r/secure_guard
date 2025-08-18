use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use secureguard_shared::{SecureGuardError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubscriptionPlan {
    pub plan_id: Uuid,
    pub plan_slug: String,
    pub plan_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub max_devices: i32,
    pub max_api_keys: i32,
    pub real_time_monitoring: bool,
    pub advanced_threat_detection: bool,
    pub custom_rules: bool,
    pub api_access: bool,
    pub priority_support: bool,
    pub audit_logs: bool,
    pub integrations_enabled: bool,
    pub vulnerability_scanning: bool,
    pub compliance_reporting: bool,
    pub remote_response: bool,
    pub custom_dashboards: bool,
    pub bulk_operations: bool,
    pub log_retention_days: i32,
    pub alert_history_days: i32,
    pub monthly_price_cents: i32,
    pub yearly_price_cents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserSubscription {
    pub user_id: Uuid,
    pub subscription_id: Uuid,
    pub plan: SubscriptionPlan,
    pub status: String,
    pub is_trial: bool,
    pub current_period_end: chrono::DateTime<Utc>,
    pub current_devices: i32,
    pub current_api_keys: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LimitCheckResult {
    pub allowed: bool,
    pub current_usage: i32,
    pub limit: i32,
    pub is_unlimited: bool,
    pub upgrade_required: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureAccessResult {
    pub allowed: bool,
    pub feature_name: String,
    pub required_plan: String,
    pub current_plan: String,
    pub upgrade_message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpgradeProposal {
    pub current_plan: String,
    pub recommended_plan: String,
    pub additional_devices: i32,
    pub additional_features: Vec<String>,
    pub monthly_cost: f64,
    pub yearly_cost: f64,
    pub savings_yearly: f64,
}

pub struct SubscriptionService {
    pool: PgPool,
}

impl SubscriptionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get user's current subscription with plan details
    pub async fn get_user_subscription(&self, user_id: Uuid) -> Result<Option<UserSubscription>> {
        let subscription = sqlx::query!(
            r#"
            SELECT 
                us.user_id, us.subscription_id, us.status, us.is_trial, us.current_period_end,
                p.plan_id, p.plan_slug, p.plan_name, p.display_name, p.description,
                p.max_devices, p.max_api_keys, p.real_time_monitoring, p.advanced_threat_detection,
                p.custom_rules, p.api_access, p.priority_support, p.audit_logs, p.integrations_enabled,
                p.vulnerability_scanning, p.compliance_reporting, p.remote_response,
                p.custom_dashboards, p.bulk_operations, p.log_retention_days, p.alert_history_days,
                p.monthly_price_cents, p.yearly_price_cents,
                COALESCE(ut.current_devices, 0) as current_devices,
                COALESCE(ut.current_api_keys, 0) as current_api_keys
            FROM subscriptions.user_subscriptions us
            JOIN subscriptions.plans p ON us.plan_id = p.plan_id
            LEFT JOIN subscriptions.usage_tracking ut ON us.user_id = ut.user_id
            WHERE us.user_id = $1 AND us.status = 'active'
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        if let Some(sub) = subscription {
            Ok(Some(UserSubscription {
                user_id: sub.user_id,
                subscription_id: sub.subscription_id,
                status: sub.status,
                is_trial: sub.is_trial,
                current_period_end: sub.current_period_end,
                current_devices: sub.current_devices.unwrap_or(0),
                current_api_keys: sub.current_api_keys.unwrap_or(0),
                plan: SubscriptionPlan {
                    plan_id: sub.plan_id,
                    plan_slug: sub.plan_slug,
                    plan_name: sub.plan_name,
                    display_name: sub.display_name,
                    description: sub.description,
                    max_devices: sub.max_devices,
                    max_api_keys: sub.max_api_keys,
                    real_time_monitoring: sub.real_time_monitoring,
                    advanced_threat_detection: sub.advanced_threat_detection,
                    custom_rules: sub.custom_rules,
                    api_access: sub.api_access,
                    priority_support: sub.priority_support,
                    audit_logs: sub.audit_logs,
                    integrations_enabled: sub.integrations_enabled,
                    vulnerability_scanning: sub.vulnerability_scanning,
                    compliance_reporting: sub.compliance_reporting,
                    remote_response: sub.remote_response,
                    custom_dashboards: sub.custom_dashboards,
                    bulk_operations: sub.bulk_operations,
                    log_retention_days: sub.log_retention_days,
                    alert_history_days: sub.alert_history_days,
                    monthly_price_cents: sub.monthly_price_cents,
                    yearly_price_cents: sub.yearly_price_cents,
                }
            }))
        } else {
            Ok(None)
        }
    }

    /// Check if user can register another device
    pub async fn can_register_device(&self, user_id: Uuid) -> Result<LimitCheckResult> {
        let subscription = self.get_user_subscription(user_id).await?
            .ok_or_else(|| SecureGuardError::NotFound("User subscription not found".to_string()))?;

        let is_unlimited = subscription.plan.max_devices == -1;
        let allowed = is_unlimited || subscription.current_devices < subscription.plan.max_devices;

        Ok(LimitCheckResult {
            allowed,
            current_usage: subscription.current_devices,
            limit: subscription.plan.max_devices,
            is_unlimited,
            upgrade_required: !allowed,
            message: if allowed {
                if is_unlimited {
                    "Unlimited devices available".to_string()
                } else {
                    format!("Device {} of {} available", 
                        subscription.current_devices + 1, 
                        subscription.plan.max_devices
                    )
                }
            } else {
                format!(
                    "Device limit reached ({}/{}). Upgrade to {} for more devices.",
                    subscription.current_devices,
                    subscription.plan.max_devices,
                    self.get_recommended_upgrade(&subscription.plan.plan_slug).await?
                )
            }
        })
    }

    /// Check if user can create another API key
    pub async fn can_create_api_key(&self, user_id: Uuid) -> Result<LimitCheckResult> {
        let subscription = self.get_user_subscription(user_id).await?
            .ok_or_else(|| SecureGuardError::NotFound("User subscription not found".to_string()))?;

        let is_unlimited = subscription.plan.max_api_keys == -1;
        let allowed = is_unlimited || subscription.current_api_keys < subscription.plan.max_api_keys;

        Ok(LimitCheckResult {
            allowed,
            current_usage: subscription.current_api_keys,
            limit: subscription.plan.max_api_keys,
            is_unlimited,
            upgrade_required: !allowed,
            message: if allowed {
                if is_unlimited {
                    "Unlimited API keys available".to_string()
                } else {
                    format!("API key {} of {} available", 
                        subscription.current_api_keys + 1, 
                        subscription.plan.max_api_keys
                    )
                }
            } else {
                format!(
                    "API key limit reached ({}/{}). Upgrade to {} for more keys.",
                    subscription.current_api_keys,
                    subscription.plan.max_api_keys,
                    self.get_recommended_upgrade(&subscription.plan.plan_slug).await?
                )
            }
        })
    }

    /// Check if user has access to a specific feature
    pub async fn check_feature_access(&self, user_id: Uuid, feature: &str) -> Result<FeatureAccessResult> {
        let subscription = self.get_user_subscription(user_id).await?
            .ok_or_else(|| SecureGuardError::NotFound("User subscription not found".to_string()))?;

        let allowed = match feature {
            "real_time_monitoring" => subscription.plan.real_time_monitoring,
            "advanced_threat_detection" => subscription.plan.advanced_threat_detection,
            "custom_rules" => subscription.plan.custom_rules,
            "api_access" => subscription.plan.api_access,
            "priority_support" => subscription.plan.priority_support,
            "audit_logs" => subscription.plan.audit_logs,
            "integrations_enabled" => subscription.plan.integrations_enabled,
            "vulnerability_scanning" => subscription.plan.vulnerability_scanning,
            "compliance_reporting" => subscription.plan.compliance_reporting,
            "remote_response" => subscription.plan.remote_response,
            "custom_dashboards" => subscription.plan.custom_dashboards,
            "bulk_operations" => subscription.plan.bulk_operations,
            _ => false, // Unknown features are denied
        };

        let required_plan = if !allowed {
            self.get_minimum_plan_for_feature(feature).await?
        } else {
            subscription.plan.plan_slug.clone()
        };

        Ok(FeatureAccessResult {
            allowed,
            feature_name: feature.to_string(),
            required_plan: required_plan.clone(),
            current_plan: subscription.plan.plan_slug.clone(),
            upgrade_message: if allowed {
                format!("Feature '{}' is available in your {} plan", feature, subscription.plan.display_name)
            } else {
                format!("Feature '{}' requires {} plan. Upgrade to access this feature.", 
                    feature, required_plan)
            }
        })
    }

    /// Increment device count when agent registers
    pub async fn increment_device_count(&self, user_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO subscriptions.usage_tracking (user_id, subscription_id, current_devices)
            VALUES (
                $1, 
                (SELECT subscription_id FROM subscriptions.user_subscriptions WHERE user_id = $1 AND status = 'active'),
                1
            )
            ON CONFLICT (user_id) 
            DO UPDATE SET 
                current_devices = subscriptions.usage_tracking.current_devices + 1,
                updated_at = now()
            "#,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Increment API key count when key is created
    pub async fn increment_api_key_count(&self, user_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO subscriptions.usage_tracking (user_id, subscription_id, current_api_keys)
            VALUES (
                $1, 
                (SELECT subscription_id FROM subscriptions.user_subscriptions WHERE user_id = $1 AND status = 'active'),
                1
            )
            ON CONFLICT (user_id) 
            DO UPDATE SET 
                current_api_keys = subscriptions.usage_tracking.current_api_keys + 1,
                updated_at = now()
            "#,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Decrement device count when agent is removed
    pub async fn decrement_device_count(&self, user_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE subscriptions.usage_tracking 
            SET 
                current_devices = GREATEST(0, current_devices - 1),
                updated_at = now()
            WHERE user_id = $1
            "#,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Decrement API key count when key is revoked
    pub async fn decrement_api_key_count(&self, user_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE subscriptions.usage_tracking 
            SET 
                current_api_keys = GREATEST(0, current_api_keys - 1),
                updated_at = now()
            WHERE user_id = $1
            "#,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get all available plans for comparison
    pub async fn get_all_plans(&self) -> Result<Vec<SubscriptionPlan>> {
        let plans = sqlx::query_as!(
            SubscriptionPlan,
            r#"
            SELECT 
                plan_id, plan_slug, plan_name, display_name, description,
                max_devices, max_api_keys, real_time_monitoring, advanced_threat_detection,
                custom_rules, api_access, priority_support, audit_logs, integrations_enabled,
                vulnerability_scanning, compliance_reporting, remote_response,
                custom_dashboards, bulk_operations, log_retention_days, alert_history_days,
                monthly_price_cents, yearly_price_cents
            FROM subscriptions.plans 
            WHERE is_active = TRUE
            ORDER BY sort_order
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(plans)
    }

    /// Get upgrade proposal for user
    pub async fn get_upgrade_proposal(&self, user_id: Uuid) -> Result<UpgradeProposal> {
        let subscription = self.get_user_subscription(user_id).await?
            .ok_or_else(|| SecureGuardError::NotFound("User subscription not found".to_string()))?;

        let recommended_plan = self.get_recommended_upgrade(&subscription.plan.plan_slug).await?;
        let plans = self.get_all_plans().await?;
        
        let current_plan = plans.iter().find(|p| p.plan_slug == subscription.plan.plan_slug).unwrap();
        let recommended = plans.iter().find(|p| p.plan_slug == recommended_plan).unwrap();

        let additional_devices = if recommended.max_devices == -1 {
            999 // Show as "unlimited"
        } else {
            recommended.max_devices - current_plan.max_devices
        };

        let additional_features = self.get_additional_features(current_plan, recommended);

        let monthly_cost = recommended.monthly_price_cents as f64 / 100.0;
        let yearly_cost = recommended.yearly_price_cents as f64 / 100.0;
        let savings_yearly = (monthly_cost * 12.0) - yearly_cost;

        Ok(UpgradeProposal {
            current_plan: current_plan.display_name.clone(),
            recommended_plan: recommended.display_name.clone(),
            additional_devices,
            additional_features,
            monthly_cost,
            yearly_cost,
            savings_yearly,
        })
    }

    // Helper methods
    async fn get_recommended_upgrade(&self, current_plan: &str) -> Result<String> {
        let next_plan = match current_plan {
            "free" => "starter",
            "starter" => "professional",
            "professional" => "enterprise",
            _ => "professional", // Default
        };
        Ok(next_plan.to_string())
    }

    async fn get_minimum_plan_for_feature(&self, feature: &str) -> Result<String> {
        let required_plan = match feature {
            "real_time_monitoring" | "api_access" | "audit_logs" => "starter",
            "advanced_threat_detection" | "custom_rules" | "vulnerability_scanning" | "remote_response" => "professional",
            "compliance_reporting" | "custom_dashboards" | "bulk_operations" => "enterprise",
            _ => "professional", // Default to professional for unknown features
        };
        Ok(required_plan.to_string())
    }

    fn get_additional_features(&self, current: &SubscriptionPlan, recommended: &SubscriptionPlan) -> Vec<String> {
        let mut features = Vec::new();
        
        if !current.real_time_monitoring && recommended.real_time_monitoring {
            features.push("Real-time Monitoring".to_string());
        }
        if !current.advanced_threat_detection && recommended.advanced_threat_detection {
            features.push("Advanced Threat Detection".to_string());
        }
        if !current.custom_rules && recommended.custom_rules {
            features.push("Custom Security Rules".to_string());
        }
        if !current.priority_support && recommended.priority_support {
            features.push("Priority Support".to_string());
        }
        if !current.vulnerability_scanning && recommended.vulnerability_scanning {
            features.push("Vulnerability Scanning".to_string());
        }
        if !current.compliance_reporting && recommended.compliance_reporting {
            features.push("Compliance Reporting".to_string());
        }
        if !current.remote_response && recommended.remote_response {
            features.push("Remote Response".to_string());
        }
        if !current.custom_dashboards && recommended.custom_dashboards {
            features.push("Custom Dashboards".to_string());
        }
        if !current.bulk_operations && recommended.bulk_operations {
            features.push("Bulk Operations".to_string());
        }

        features
    }
}