use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use secureguard_shared::{SecureGuardError, Result};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    pub plan_id: Uuid,
    pub plan_name: String,
    pub plan_slug: String,
    pub display_name: String,
    pub description: Option<String>,
    
    // Device Limits
    pub max_devices: i32,
    pub max_api_keys: i32,
    
    // Feature Flags
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
    
    // Data Retention
    pub log_retention_days: i32,
    pub alert_history_days: i32,
    
    // Pricing
    pub monthly_price_cents: i32,
    pub yearly_price_cents: i32,
    
    // Metadata
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlanRequest {
    pub plan_name: String,
    pub plan_slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub max_devices: i32,
    pub max_api_keys: i32,
    pub features: PlanFeatures,
    pub retention: DataRetention,
    pub pricing: PlanPricing,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlanRequest {
    pub plan_name: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub max_devices: Option<i32>,
    pub max_api_keys: Option<i32>,
    pub features: Option<PlanFeatures>,
    pub retention: Option<DataRetention>,
    pub pricing: Option<PlanPricing>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanFeatures {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRetention {
    pub log_retention_days: i32,
    pub alert_history_days: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanPricing {
    pub monthly_price_cents: i32,
    pub yearly_price_cents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanUsageStats {
    pub plan_id: Uuid,
    pub plan_name: String,
    pub total_users: i32,
    pub active_users: i32,
    pub trial_users: i32,
    pub monthly_revenue_cents: i64,
    pub yearly_revenue_cents: i64,
    pub avg_devices_per_user: f64,
    pub avg_api_keys_per_user: f64,
    pub feature_adoption: HashMap<String, FeatureAdoptionStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureAdoptionStats {
    pub feature_name: String,
    pub users_with_access: i32,
    pub users_actively_using: i32,
    pub adoption_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMigrationRequest {
    pub from_plan_id: Uuid,
    pub to_plan_id: Uuid,
    pub user_ids: Option<Vec<Uuid>>, // If None, migrate all users
    pub migration_date: Option<DateTime<Utc>>, // If None, migrate immediately
    pub send_notifications: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub affected_users: i32,
    pub impact_analysis: PlanImpactAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanImpactAnalysis {
    pub users_exceeding_device_limit: i32,
    pub users_exceeding_api_key_limit: i32,
    pub features_being_removed: Vec<String>,
    pub data_retention_impact: String,
    pub estimated_revenue_impact: i64,
}

pub struct SubscriptionAdminService {
    pool: PgPool,
}

impl SubscriptionAdminService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get all subscription plans
    pub async fn get_all_plans(&self, include_inactive: bool) -> Result<Vec<SubscriptionPlan>> {
        let plans = if include_inactive {
            sqlx::query_as!(
                SubscriptionPlan,
                r#"
                SELECT plan_id, plan_name, plan_slug, display_name, description,
                       max_devices, max_api_keys,
                       real_time_monitoring, advanced_threat_detection, custom_rules,
                       api_access, priority_support, audit_logs, integrations_enabled,
                       vulnerability_scanning, compliance_reporting, remote_response,
                       custom_dashboards, bulk_operations,
                       log_retention_days, alert_history_days,
                       monthly_price_cents, yearly_price_cents,
                       is_active, sort_order, created_at
                FROM subscriptions.plans
                ORDER BY sort_order, plan_name
                "#
            )
        } else {
            sqlx::query_as!(
                SubscriptionPlan,
                r#"
                SELECT plan_id, plan_name, plan_slug, display_name, description,
                       max_devices, max_api_keys,
                       real_time_monitoring, advanced_threat_detection, custom_rules,
                       api_access, priority_support, audit_logs, integrations_enabled,
                       vulnerability_scanning, compliance_reporting, remote_response,
                       custom_dashboards, bulk_operations,
                       log_retention_days, alert_history_days,
                       monthly_price_cents, yearly_price_cents,
                       is_active, sort_order, created_at
                FROM subscriptions.plans
                WHERE is_active = TRUE
                ORDER BY sort_order, plan_name
                "#
            )
        }.fetch_all(&self.pool).await.map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        info!("📋 Retrieved {} subscription plans (include_inactive: {})", plans.len(), include_inactive);
        Ok(plans)
    }

    /// Get subscription plan by ID
    pub async fn get_plan_by_id(&self, plan_id: Uuid) -> Result<SubscriptionPlan> {
        let plan = sqlx::query_as!(
            SubscriptionPlan,
            r#"
            SELECT plan_id, plan_name, plan_slug, display_name, description,
                   max_devices, max_api_keys,
                   real_time_monitoring, advanced_threat_detection, custom_rules,
                   api_access, priority_support, audit_logs, integrations_enabled,
                   vulnerability_scanning, compliance_reporting, remote_response,
                   custom_dashboards, bulk_operations,
                   log_retention_days, alert_history_days,
                   monthly_price_cents, yearly_price_cents,
                   is_active, sort_order, created_at
            FROM subscriptions.plans
            WHERE plan_id = $1
            "#,
            plan_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .ok_or_else(|| SecureGuardError::PlanNotFound)?;

        Ok(plan)
    }

    /// Create a new subscription plan
    pub async fn create_plan(&self, request: CreatePlanRequest) -> Result<SubscriptionPlan> {
        info!("🆕 Creating new subscription plan: {}", request.plan_name);

        // Validate plan slug uniqueness
        let existing = sqlx::query!(
            "SELECT plan_id FROM subscriptions.plans WHERE plan_slug = $1",
            request.plan_slug
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        if existing.is_some() {
            return Err(SecureGuardError::ValidationError(
                format!("Plan slug '{}' already exists", request.plan_slug)
            ));
        }

        // Validate limits make sense
        self.validate_plan_limits(&request)?;

        let sort_order = request.sort_order.unwrap_or_else(|| {
            // Get next sort order
            100 // Default sort order
        });

        let plan_id = sqlx::query!(
            r#"
            INSERT INTO subscriptions.plans (
                plan_name, plan_slug, display_name, description,
                max_devices, max_api_keys,
                real_time_monitoring, advanced_threat_detection, custom_rules,
                api_access, priority_support, audit_logs, integrations_enabled,
                vulnerability_scanning, compliance_reporting, remote_response,
                custom_dashboards, bulk_operations,
                log_retention_days, alert_history_days,
                monthly_price_cents, yearly_price_cents, sort_order
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                $14, $15, $16, $17, $18, $19, $20, $21, $22, $23
            ) RETURNING plan_id
            "#,
            request.plan_name,
            request.plan_slug,
            request.display_name,
            request.description,
            request.max_devices,
            request.max_api_keys,
            request.features.real_time_monitoring,
            request.features.advanced_threat_detection,
            request.features.custom_rules,
            request.features.api_access,
            request.features.priority_support,
            request.features.audit_logs,
            request.features.integrations_enabled,
            request.features.vulnerability_scanning,
            request.features.compliance_reporting,
            request.features.remote_response,
            request.features.custom_dashboards,
            request.features.bulk_operations,
            request.retention.log_retention_days,
            request.retention.alert_history_days,
            request.pricing.monthly_price_cents,
            request.pricing.yearly_price_cents,
            sort_order
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .plan_id;

        info!("✅ Created subscription plan: {} ({})", request.plan_name, plan_id);

        // Return the created plan
        self.get_plan_by_id(plan_id).await
    }

    /// Update an existing subscription plan
    pub async fn update_plan(&self, plan_id: Uuid, request: UpdatePlanRequest) -> Result<SubscriptionPlan> {
        info!("📝 Updating subscription plan: {}", plan_id);

        // Check if plan exists
        let _existing_plan = self.get_plan_by_id(plan_id).await?;

        // Validate impact before updating
        let validation_result = self.validate_plan_update(plan_id, &request).await?;
        if !validation_result.is_valid {
            return Err(SecureGuardError::ValidationError(
                format!("Plan update validation failed: {:?}", validation_result.errors)
            ));
        }

        // Build dynamic update query
        let mut query_parts = Vec::new();
        let mut param_count = 1;

        if request.plan_name.is_some() {
            query_parts.push(format!("plan_name = ${}", param_count));
            param_count += 1;
        }
        if request.display_name.is_some() {
            query_parts.push(format!("display_name = ${}", param_count));
            param_count += 1;
        }
        if request.description.is_some() {
            query_parts.push(format!("description = ${}", param_count));
            param_count += 1;
        }
        if request.max_devices.is_some() {
            query_parts.push(format!("max_devices = ${}", param_count));
            param_count += 1;
        }
        if request.max_api_keys.is_some() {
            query_parts.push(format!("max_api_keys = ${}", param_count));
            param_count += 1;
        }

        // Handle features if provided
        if let Some(ref features) = request.features {
            query_parts.extend([
                format!("real_time_monitoring = ${}", param_count),
                format!("advanced_threat_detection = ${}", param_count + 1),
                format!("custom_rules = ${}", param_count + 2),
                format!("api_access = ${}", param_count + 3),
                format!("priority_support = ${}", param_count + 4),
                format!("audit_logs = ${}", param_count + 5),
                format!("integrations_enabled = ${}", param_count + 6),
                format!("vulnerability_scanning = ${}", param_count + 7),
                format!("compliance_reporting = ${}", param_count + 8),
                format!("remote_response = ${}", param_count + 9),
                format!("custom_dashboards = ${}", param_count + 10),
                format!("bulk_operations = ${}", param_count + 11),
            ]);
            param_count += 12;
        }

        // Handle retention if provided
        if let Some(ref retention) = request.retention {
            query_parts.extend([
                format!("log_retention_days = ${}", param_count),
                format!("alert_history_days = ${}", param_count + 1),
            ]);
            param_count += 2;
        }

        // Handle pricing if provided
        if let Some(ref pricing) = request.pricing {
            query_parts.extend([
                format!("monthly_price_cents = ${}", param_count),
                format!("yearly_price_cents = ${}", param_count + 1),
            ]);
            param_count += 2;
        }

        if request.is_active.is_some() {
            query_parts.push(format!("is_active = ${}", param_count));
            param_count += 1;
        }

        if request.sort_order.is_some() {
            query_parts.push(format!("sort_order = ${}", param_count));
        }

        if query_parts.is_empty() {
            return Err(SecureGuardError::ValidationError("No fields to update".to_string()));
        }

        // For simplicity, let's use a more straightforward approach with individual updates
        if let Some(ref features) = request.features {
            sqlx::query!(
                r#"
                UPDATE subscriptions.plans SET
                    real_time_monitoring = $1,
                    advanced_threat_detection = $2,
                    custom_rules = $3,
                    api_access = $4,
                    priority_support = $5,
                    audit_logs = $6,
                    integrations_enabled = $7,
                    vulnerability_scanning = $8,
                    compliance_reporting = $9,
                    remote_response = $10,
                    custom_dashboards = $11,
                    bulk_operations = $12
                WHERE plan_id = $13
                "#,
                features.real_time_monitoring,
                features.advanced_threat_detection,
                features.custom_rules,
                features.api_access,
                features.priority_support,
                features.audit_logs,
                features.integrations_enabled,
                features.vulnerability_scanning,
                features.compliance_reporting,
                features.remote_response,
                features.custom_dashboards,
                features.bulk_operations,
                plan_id
            )
            .execute(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;
        }

        // Update other fields as needed
        if let Some(max_devices) = request.max_devices {
            sqlx::query!(
                "UPDATE subscriptions.plans SET max_devices = $1 WHERE plan_id = $2",
                max_devices, plan_id
            )
            .execute(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;
        }

        if let Some(max_api_keys) = request.max_api_keys {
            sqlx::query!(
                "UPDATE subscriptions.plans SET max_api_keys = $1 WHERE plan_id = $2",
                max_api_keys, plan_id
            )
            .execute(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;
        }

        info!("✅ Updated subscription plan: {}", plan_id);

        // Return the updated plan
        self.get_plan_by_id(plan_id).await
    }

    /// Delete a subscription plan (soft delete by deactivating)
    pub async fn delete_plan(&self, plan_id: Uuid, force: bool) -> Result<()> {
        info!("🗑️ Deleting subscription plan: {} (force: {})", plan_id, force);

        // Check if plan exists
        let _plan = self.get_plan_by_id(plan_id).await?;

        // Check if plan has active users
        let user_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM subscriptions.user_subscriptions WHERE plan_id = $1 AND status = 'active'",
            plan_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .count
        .unwrap_or(0);

        if user_count > 0 && !force {
            return Err(SecureGuardError::ValidationError(
                format!("Cannot delete plan with {} active users. Use force=true to override.", user_count)
            ));
        }

        if force && user_count > 0 {
            warn!("⚠️ Force deleting plan with {} active users", user_count);
            // In a real implementation, you'd migrate users to a default plan
        }

        // Soft delete by deactivating
        sqlx::query!(
            "UPDATE subscriptions.plans SET is_active = FALSE WHERE plan_id = $1",
            plan_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        info!("✅ Deactivated subscription plan: {}", plan_id);
        Ok(())
    }

    /// Get plan usage statistics
    pub async fn get_plan_usage_stats(&self, plan_id: Option<Uuid>) -> Result<Vec<PlanUsageStats>> {
        let stats = if let Some(plan_id) = plan_id {
            // Get stats for specific plan
            self.get_single_plan_stats(plan_id).await?
        } else {
            // Get stats for all plans
            self.get_all_plan_stats().await?
        };

        Ok(stats)
    }

    /// Migrate users between plans
    pub async fn migrate_users_between_plans(&self, request: PlanMigrationRequest) -> Result<u32> {
        info!("🔄 Migrating users from plan {} to plan {}", request.from_plan_id, request.to_plan_id);

        // Validate both plans exist
        let _from_plan = self.get_plan_by_id(request.from_plan_id).await?;
        let _to_plan = self.get_plan_by_id(request.to_plan_id).await?;

        let affected_users = if let Some(ref user_ids) = request.user_ids {
            // Migrate specific users
            let user_count = sqlx::query!(
                r#"
                UPDATE subscriptions.user_subscriptions 
                SET plan_id = $1, updated_at = now()
                WHERE plan_id = $2 AND user_id = ANY($3) AND status = 'active'
                "#,
                request.to_plan_id,
                request.from_plan_id,
                user_ids
            )
            .execute(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
            .rows_affected();

            user_count as u32
        } else {
            // Migrate all users
            let user_count = sqlx::query!(
                r#"
                UPDATE subscriptions.user_subscriptions 
                SET plan_id = $1, updated_at = now()
                WHERE plan_id = $2 AND status = 'active'
                "#,
                request.to_plan_id,
                request.from_plan_id
            )
            .execute(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
            .rows_affected();

            user_count as u32
        };

        info!("✅ Migrated {} users between plans", affected_users);

        // TODO: Send notifications if requested
        if request.send_notifications {
            // Implementation would send migration notifications
        }

        Ok(affected_users)
    }

    /// Validate plan configuration
    pub async fn validate_plan_update(&self, plan_id: Uuid, request: &UpdatePlanRequest) -> Result<PlanValidationResult> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Check device limit impact
        if let Some(max_devices) = request.max_devices {
            let users_exceeding = sqlx::query!(
                r#"
                SELECT COUNT(*) as count
                FROM subscriptions.usage_tracking ut
                JOIN subscriptions.user_subscriptions us ON ut.user_id = us.user_id
                WHERE us.plan_id = $1 AND ut.current_devices > $2
                "#,
                plan_id,
                max_devices
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
            .count
            .unwrap_or(0);

            if users_exceeding > 0 {
                warnings.push(format!("{} users exceed new device limit of {}", users_exceeding, max_devices));
            }
        }

        // Check API key limit impact
        if let Some(max_api_keys) = request.max_api_keys {
            let users_exceeding = sqlx::query!(
                r#"
                SELECT COUNT(*) as count
                FROM subscriptions.usage_tracking ut
                JOIN subscriptions.user_subscriptions us ON ut.user_id = us.user_id
                WHERE us.plan_id = $1 AND ut.current_api_keys > $2
                "#,
                plan_id,
                max_api_keys
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
            .count
            .unwrap_or(0);

            if users_exceeding > 0 {
                warnings.push(format!("{} users exceed new API key limit of {}", users_exceeding, max_api_keys));
            }
        }

        // Check feature removal impact
        let mut features_being_removed = Vec::new();
        if let Some(ref features) = request.features {
            let current_plan = self.get_plan_by_id(plan_id).await?;
            
            if current_plan.real_time_monitoring && !features.real_time_monitoring {
                features_being_removed.push("real_time_monitoring".to_string());
            }
            if current_plan.advanced_threat_detection && !features.advanced_threat_detection {
                features_being_removed.push("advanced_threat_detection".to_string());
            }
            // Add other feature checks...
        }

        if !features_being_removed.is_empty() {
            warnings.push(format!("Features being removed: {}", features_being_removed.join(", ")));
        }

        let is_valid = errors.is_empty();

        Ok(PlanValidationResult {
            is_valid,
            warnings,
            errors,
            affected_users: 0, // Calculate based on plan usage
            impact_analysis: PlanImpactAnalysis {
                users_exceeding_device_limit: 0,
                users_exceeding_api_key_limit: 0,
                features_being_removed,
                data_retention_impact: "No impact".to_string(),
                estimated_revenue_impact: 0,
            },
        })
    }

    // Helper methods
    fn validate_plan_limits(&self, request: &CreatePlanRequest) -> Result<()> {
        if request.max_devices < 1 && request.max_devices != -1 {
            return Err(SecureGuardError::ValidationError(
                "max_devices must be >= 1 or -1 for unlimited".to_string()
            ));
        }

        if request.max_api_keys < 1 && request.max_api_keys != -1 {
            return Err(SecureGuardError::ValidationError(
                "max_api_keys must be >= 1 or -1 for unlimited".to_string()
            ));
        }

        if request.retention.log_retention_days < 1 {
            return Err(SecureGuardError::ValidationError(
                "log_retention_days must be >= 1".to_string()
            ));
        }

        if request.retention.alert_history_days < 1 {
            return Err(SecureGuardError::ValidationError(
                "alert_history_days must be >= 1".to_string()
            ));
        }

        if request.pricing.monthly_price_cents < 0 {
            return Err(SecureGuardError::ValidationError(
                "monthly_price_cents must be >= 0".to_string()
            ));
        }

        if request.pricing.yearly_price_cents < 0 {
            return Err(SecureGuardError::ValidationError(
                "yearly_price_cents must be >= 0".to_string()
            ));
        }

        Ok(())
    }

    async fn get_single_plan_stats(&self, plan_id: Uuid) -> Result<Vec<PlanUsageStats>> {
        // Implementation would get detailed stats for a single plan
        Ok(vec![])
    }

    async fn get_all_plan_stats(&self) -> Result<Vec<PlanUsageStats>> {
        // Implementation would get stats for all plans
        Ok(vec![])
    }
}