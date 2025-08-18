use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::communication::Client;
use crate::utils::config::Config;

/// Trait that all feature modules must implement
#[async_trait]
pub trait FeatureModule: Send + Sync {
    /// Get the feature name/ID
    fn name(&self) -> &str;
    
    /// Start the feature module
    async fn start(&self) -> Result<()>;
    
    /// Stop the feature module
    async fn stop(&self) -> Result<()>;
    
    /// Get current feature status
    async fn status(&self) -> Result<FeatureStatus>;
    
    /// Update feature configuration
    async fn update_config(&self, config: serde_json::Value) -> Result<()>;
    
    /// Get feature metrics
    async fn metrics(&self) -> Result<FeatureMetrics>;
    
    /// Check if feature is healthy
    async fn health_check(&self) -> Result<HealthStatus>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDefinition {
    pub feature_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: String,
    pub required_subscription: String,
    pub required_role: String,
    pub dependencies: Vec<String>,
    pub conflicts_with: Vec<String>,
    pub default_config: serde_json::Value,
    pub config_schema: Option<serde_json::Value>,
    pub is_enabled: bool,
    pub auto_enable: bool,
    pub rollout_percentage: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStatus {
    pub feature_id: String,
    pub is_active: bool,
    pub is_healthy: bool,
    pub last_activity: Option<DateTime<Utc>>,
    pub start_time: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub config_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureMetrics {
    pub feature_id: String,
    pub uptime_seconds: u64,
    pub operations_count: u64,
    pub errors_count: u64,
    pub last_error: Option<String>,
    pub resource_usage: ResourceUsage,
    pub custom_metrics: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    pub disk_bytes: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning { message: String },
    Unhealthy { message: String },
    Unknown,
}

#[derive(Debug, Clone)]
pub enum FeatureCommand {
    Enable { feature_id: String },
    Disable { feature_id: String },
    UpdateConfig { feature_id: String, config: serde_json::Value },
    GetStatus { feature_id: String },
    GetMetrics { feature_id: String },
    HealthCheck { feature_id: String },
    Restart { feature_id: String },
}

pub struct FeatureManager {
    config: Arc<Config>,
    client: Arc<Client>,
    available_features: RwLock<HashMap<String, FeatureDefinition>>,
    active_modules: RwLock<HashMap<String, Box<dyn FeatureModule>>>,
    command_tx: mpsc::UnboundedSender<FeatureCommand>,
    command_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<FeatureCommand>>>,
}

impl FeatureManager {
    pub fn new(config: Config, client: Client) -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        
        Self {
            config: Arc::new(config),
            client: Arc::new(client),
            available_features: RwLock::new(HashMap::new()),
            active_modules: RwLock::new(HashMap::new()),
            command_tx,
            command_rx: Arc::new(tokio::sync::Mutex::new(command_rx)),
        }
    }

    /// Initialize the feature manager
    pub async fn initialize(&self) -> Result<()> {
        info!("🧩 Initializing Feature Manager");

        // Register built-in features
        self.register_builtin_features().await?;

        // Sync features from server
        self.sync_features_from_server().await?;

        // Auto-enable features based on subscription
        self.auto_enable_subscription_features().await?;

        // Start command processing loop
        self.start_command_processor().await?;

        info!("✅ Feature Manager initialized successfully");
        Ok(())
    }

    /// Register built-in feature modules
    async fn register_builtin_features(&self) -> Result<()> {
        info!("📋 Registering built-in features");

        let builtin_features = vec![
            FeatureDefinition {
                feature_id: "basic_monitoring".to_string(),
                name: "Basic System Monitoring".to_string(),
                description: "Essential system health and status monitoring".to_string(),
                version: "1.0.0".to_string(),
                category: "monitoring".to_string(),
                required_subscription: "free".to_string(),
                required_role: "user".to_string(),
                dependencies: vec![],
                conflicts_with: vec![],
                default_config: serde_json::json!({
                    "scan_interval": 300,
                    "alerts_enabled": true,
                    "metrics_collection": true
                }),
                config_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "scan_interval": {"type": "integer", "minimum": 60},
                        "alerts_enabled": {"type": "boolean"},
                        "metrics_collection": {"type": "boolean"}
                    }
                })),
                is_enabled: true,
                auto_enable: true,
                rollout_percentage: 100,
            },
            FeatureDefinition {
                feature_id: "real_time_monitoring".to_string(),
                name: "Real-time Security Monitoring".to_string(),
                description: "Continuous real-time security monitoring and alerting".to_string(),
                version: "1.0.0".to_string(),
                category: "monitoring".to_string(),
                required_subscription: "starter".to_string(),
                required_role: "user".to_string(),
                dependencies: vec!["basic_monitoring".to_string()],
                conflicts_with: vec![],
                default_config: serde_json::json!({
                    "scan_interval": 30,
                    "real_time_alerts": true,
                    "file_monitoring": true,
                    "process_monitoring": true,
                    "network_monitoring": false
                }),
                config_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "scan_interval": {"type": "integer", "minimum": 10},
                        "real_time_alerts": {"type": "boolean"},
                        "file_monitoring": {"type": "boolean"},
                        "process_monitoring": {"type": "boolean"},
                        "network_monitoring": {"type": "boolean"}
                    }
                })),
                is_enabled: false,
                auto_enable: true,
                rollout_percentage: 100,
            },
            FeatureDefinition {
                feature_id: "advanced_threat_detection".to_string(),
                name: "Advanced Threat Detection".to_string(),
                description: "AI-powered behavioral analysis and advanced threat detection".to_string(),
                version: "1.0.0".to_string(),
                category: "security".to_string(),
                required_subscription: "professional".to_string(),
                required_role: "user".to_string(),
                dependencies: vec!["real_time_monitoring".to_string()],
                conflicts_with: vec![],
                default_config: serde_json::json!({
                    "ai_model": "standard",
                    "behavioral_analysis": true,
                    "cloud_analysis": true,
                    "sensitivity": "balanced",
                    "machine_learning": true
                }),
                config_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "ai_model": {"type": "string", "enum": ["basic", "standard", "advanced"]},
                        "behavioral_analysis": {"type": "boolean"},
                        "cloud_analysis": {"type": "boolean"},
                        "sensitivity": {"type": "string", "enum": ["low", "balanced", "high"]},
                        "machine_learning": {"type": "boolean"}
                    }
                })),
                is_enabled: false,
                auto_enable: true,
                rollout_percentage: 100,
            },
            FeatureDefinition {
                feature_id: "file_integrity_monitoring".to_string(),
                name: "File Integrity Monitoring".to_string(),
                description: "Monitor critical files for unauthorized changes".to_string(),
                version: "1.0.0".to_string(),
                category: "security".to_string(),
                required_subscription: "professional".to_string(),
                required_role: "admin".to_string(),
                dependencies: vec![],
                conflicts_with: vec![],
                default_config: serde_json::json!({
                    "watch_paths": ["C:\\Windows\\System32", "C:\\Program Files"],
                    "check_interval": 300,
                    "hash_algorithm": "sha256",
                    "ignore_patterns": ["*.tmp", "*.log"],
                    "recursive": true
                }),
                config_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "watch_paths": {"type": "array", "items": {"type": "string"}},
                        "check_interval": {"type": "integer", "minimum": 60},
                        "hash_algorithm": {"type": "string", "enum": ["md5", "sha1", "sha256"]},
                        "ignore_patterns": {"type": "array", "items": {"type": "string"}},
                        "recursive": {"type": "boolean"}
                    }
                })),
                is_enabled: false,
                auto_enable: false, // User must explicitly enable
                rollout_percentage: 100,
            },
            FeatureDefinition {
                feature_id: "vulnerability_scanning".to_string(),
                name: "Vulnerability Scanning".to_string(),
                description: "Automated vulnerability detection and assessment".to_string(),
                version: "1.0.0".to_string(),
                category: "security".to_string(),
                required_subscription: "professional".to_string(),
                required_role: "admin".to_string(),
                dependencies: vec!["basic_monitoring".to_string()],
                conflicts_with: vec![],
                default_config: serde_json::json!({
                    "scan_frequency": "weekly",
                    "cve_updates": true,
                    "custom_rules": false,
                    "scan_network": false,
                    "scan_applications": true
                }),
                config_schema: None,
                is_enabled: false,
                auto_enable: true,
                rollout_percentage: 80, // Gradual rollout
            },
            FeatureDefinition {
                feature_id: "forensic_collection".to_string(),
                name: "Forensic Data Collection".to_string(),
                description: "Advanced forensic evidence collection and analysis".to_string(),
                version: "1.0.0".to_string(),
                category: "security".to_string(),
                required_subscription: "enterprise".to_string(),
                required_role: "analyst".to_string(),
                dependencies: vec!["advanced_threat_detection".to_string()],
                conflicts_with: vec![],
                default_config: serde_json::json!({
                    "memory_dumps": true,
                    "network_capture": true,
                    "registry_snapshots": true,
                    "process_analysis": true,
                    "disk_forensics": false
                }),
                config_schema: None,
                is_enabled: false,
                auto_enable: false, // Requires explicit enablement
                rollout_percentage: 100,
            },
        ];

        let mut features = self.available_features.write().unwrap();
        for feature in builtin_features {
            features.insert(feature.feature_id.clone(), feature);
        }

        info!("✅ Registered {} built-in features", features.len());
        Ok(())
    }

    /// Sync available features from server
    async fn sync_features_from_server(&self) -> Result<()> {
        info!("🔄 Syncing features from server");

        match self.client.get_available_features().await {
            Ok(server_features) => {
                let mut features = self.available_features.write().unwrap();
                
                for feature in server_features {
                    features.insert(feature.feature_id.clone(), feature);
                }
                
                info!("✅ Synced {} features from server", features.len());
            }
            Err(e) => {
                warn!("⚠️ Failed to sync features from server: {}", e);
                // Continue with built-in features only
            }
        }

        Ok(())
    }

    /// Auto-enable features based on subscription
    async fn auto_enable_subscription_features(&self) -> Result<()> {
        info!("🎯 Auto-enabling subscription features");

        // Get current subscription info
        let subscription_info = match self.client.get_current_subscription().await {
            Ok(info) => info,
            Err(e) => {
                warn!("⚠️ Could not get subscription info: {}", e);
                return Ok(());
            }
        };

        let features = self.available_features.read().unwrap();
        let mut features_to_enable = Vec::new();

        for feature in features.values() {
            if feature.auto_enable 
                && self.subscription_allows_feature(&subscription_info.tier, &feature.required_subscription)
                && self.is_in_rollout(&feature.rollout_percentage).await? {
                features_to_enable.push(feature.feature_id.clone());
            }
        }
        drop(features);

        // Enable qualifying features
        for feature_id in features_to_enable {
            if let Err(e) = self.enable_feature(&feature_id).await {
                warn!("⚠️ Failed to auto-enable feature {}: {}", feature_id, e);
            } else {
                info!("✅ Auto-enabled feature: {}", feature_id);
            }
        }

        Ok(())
    }

    /// Enable a specific feature
    pub async fn enable_feature(&self, feature_id: &str) -> Result<()> {
        info!("🔧 Enabling feature: {}", feature_id);

        // Get feature definition
        let feature_def = {
            let features = self.available_features.read().unwrap();
            features.get(feature_id).cloned()
                .ok_or_else(|| anyhow::anyhow!("Feature not found: {}", feature_id))?
        };

        // Check subscription requirements
        let subscription_info = self.client.get_current_subscription().await?;
        if !self.subscription_allows_feature(&subscription_info.tier, &feature_def.required_subscription) {
            return Err(anyhow::anyhow!(
                "Feature '{}' requires {} subscription or higher", 
                feature_id, feature_def.required_subscription
            ));
        }

        // Check and enable dependencies
        for dep in &feature_def.dependencies {
            if !self.is_feature_active(dep).await? {
                info!("📦 Enabling dependency: {}", dep);
                self.enable_feature(dep).await?;
            }
        }

        // Check for conflicts
        for conflict in &feature_def.conflicts_with {
            if self.is_feature_active(conflict).await? {
                warn!("⚠️ Disabling conflicting feature: {}", conflict);
                self.disable_feature(conflict).await?;
            }
        }

        // Load and start feature module
        let feature_module = self.load_feature_module(&feature_def).await?;
        feature_module.start().await?;

        // Store active module
        {
            let mut active_modules = self.active_modules.write().unwrap();
            active_modules.insert(feature_id.to_string(), feature_module);
        }

        // Report to server
        self.client.report_feature_status(feature_id, true).await?;

        info!("✅ Feature enabled successfully: {}", feature_id);
        Ok(())
    }

    /// Disable a specific feature
    pub async fn disable_feature(&self, feature_id: &str) -> Result<()> {
        info!("🔌 Disabling feature: {}", feature_id);

        // Stop and remove module
        {
            let mut active_modules = self.active_modules.write().unwrap();
            if let Some(module) = active_modules.remove(feature_id) {
                if let Err(e) = module.stop().await {
                    warn!("⚠️ Error stopping feature {}: {}", feature_id, e);
                }
            }
        }

        // Check if other features depend on this one
        let dependents = self.find_dependent_features(feature_id).await?;
        for dependent in dependents {
            warn!("🔗 Also disabling dependent feature: {}", dependent);
            self.disable_feature(&dependent).await?;
        }

        // Report to server
        self.client.report_feature_status(feature_id, false).await?;

        info!("✅ Feature disabled successfully: {}", feature_id);
        Ok(())
    }

    /// Update feature configuration
    pub async fn update_feature_config(&self, feature_id: &str, config: serde_json::Value) -> Result<()> {
        info!("⚙️ Updating configuration for feature: {}", feature_id);

        // Validate configuration against schema
        {
            let features = self.available_features.read().unwrap();
            if let Some(feature_def) = features.get(feature_id) {
                if let Some(schema) = &feature_def.config_schema {
                    self.validate_config(&config, schema)?;
                }
            }
        }

        // Update active module configuration
        {
            let active_modules = self.active_modules.read().unwrap();
            if let Some(module) = active_modules.get(feature_id) {
                module.update_config(config.clone()).await?;
            }
        }

        // Report configuration change to server
        self.client.report_feature_config_update(feature_id, config).await?;

        info!("✅ Configuration updated for feature: {}", feature_id);
        Ok(())
    }

    /// Get status of all features
    pub async fn get_all_feature_status(&self) -> Result<HashMap<String, FeatureStatus>> {
        let mut status_map = HashMap::new();
        
        let active_modules = self.active_modules.read().unwrap();
        for (feature_id, module) in active_modules.iter() {
            match module.status().await {
                Ok(status) => {
                    status_map.insert(feature_id.clone(), status);
                }
                Err(e) => {
                    warn!("⚠️ Failed to get status for feature {}: {}", feature_id, e);
                    status_map.insert(feature_id.clone(), FeatureStatus {
                        feature_id: feature_id.clone(),
                        is_active: false,
                        is_healthy: false,
                        last_activity: None,
                        start_time: None,
                        error_message: Some(e.to_string()),
                        config_version: 0,
                    });
                }
            }
        }
        
        Ok(status_map)
    }

    /// Start the command processing loop
    async fn start_command_processor(&self) -> Result<()> {
        let command_rx = self.command_rx.clone();
        
        tokio::spawn(async move {
            let mut rx = command_rx.lock().await;
            while let Some(command) = rx.recv().await {
                // Process feature management commands
                debug!("📨 Processing feature command: {:?}", command);
                
                // Implementation would handle different command types
                match command {
                    FeatureCommand::Enable { feature_id } => {
                        // Handle enable command
                    }
                    FeatureCommand::Disable { feature_id } => {
                        // Handle disable command
                    }
                    // ... other commands
                    _ => {}
                }
            }
        });

        Ok(())
    }

    // Helper methods
    async fn load_feature_module(&self, feature_def: &FeatureDefinition) -> Result<Box<dyn FeatureModule>> {
        // This would dynamically load feature modules based on the feature definition
        // For now, return a mock implementation
        Ok(Box::new(MockFeatureModule::new(
            feature_def.feature_id.clone(),
            feature_def.default_config.clone()
        )))
    }

    async fn is_feature_active(&self, feature_id: &str) -> Result<bool> {
        let active_modules = self.active_modules.read().unwrap();
        Ok(active_modules.contains_key(feature_id))
    }

    async fn find_dependent_features(&self, feature_id: &str) -> Result<Vec<String>> {
        let features = self.available_features.read().unwrap();
        let mut dependents = Vec::new();
        
        for (id, feature) in features.iter() {
            if feature.dependencies.contains(&feature_id.to_string()) {
                dependents.push(id.clone());
            }
        }
        
        Ok(dependents)
    }

    fn subscription_allows_feature(&self, user_tier: &str, required_tier: &str) -> bool {
        let tier_hierarchy = ["free", "starter", "professional", "enterprise"];
        
        let user_index = tier_hierarchy.iter().position(|&t| t == user_tier).unwrap_or(0);
        let required_index = tier_hierarchy.iter().position(|&t| t == required_tier).unwrap_or(0);
        
        user_index >= required_index
    }

    async fn is_in_rollout(&self, rollout_percentage: &u32) -> Result<bool> {
        // Simple rollout logic based on agent ID hash
        let agent_id = self.client.get_agent_id().await?;
        let hash = format!("{:x}", md5::compute(agent_id.to_string().as_bytes()));
        let hash_value = u32::from_str_radix(&hash[..8], 16).unwrap_or(0);
        let percentage = hash_value % 100;
        
        Ok(percentage < *rollout_percentage)
    }

    fn validate_config(&self, config: &serde_json::Value, _schema: &serde_json::Value) -> Result<()> {
        // Implementation would validate config against JSON schema
        // For now, just check that it's a valid JSON object
        if !config.is_object() {
            return Err(anyhow::anyhow!("Configuration must be a JSON object"));
        }
        Ok(())
    }
}

/// Mock feature module for demonstration
struct MockFeatureModule {
    feature_id: String,
    config: serde_json::Value,
    is_running: Arc<std::sync::atomic::AtomicBool>,
    start_time: Option<DateTime<Utc>>,
}

impl MockFeatureModule {
    fn new(feature_id: String, config: serde_json::Value) -> Self {
        Self {
            feature_id,
            config,
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            start_time: None,
        }
    }
}

#[async_trait]
impl FeatureModule for MockFeatureModule {
    fn name(&self) -> &str {
        &self.feature_id
    }

    async fn start(&self) -> Result<()> {
        info!("🚀 Starting feature module: {}", self.feature_id);
        self.is_running.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("⏹️ Stopping feature module: {}", self.feature_id);
        self.is_running.store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    async fn status(&self) -> Result<FeatureStatus> {
        Ok(FeatureStatus {
            feature_id: self.feature_id.clone(),
            is_active: self.is_running.load(std::sync::atomic::Ordering::SeqCst),
            is_healthy: true,
            last_activity: Some(Utc::now()),
            start_time: self.start_time,
            error_message: None,
            config_version: 1,
        })
    }

    async fn update_config(&self, _config: serde_json::Value) -> Result<()> {
        info!("⚙️ Updating config for feature: {}", self.feature_id);
        Ok(())
    }

    async fn metrics(&self) -> Result<FeatureMetrics> {
        Ok(FeatureMetrics {
            feature_id: self.feature_id.clone(),
            uptime_seconds: 3600,
            operations_count: 1000,
            errors_count: 0,
            last_error: None,
            resource_usage: ResourceUsage {
                cpu_percent: 1.5,
                memory_bytes: 1024 * 1024 * 10, // 10MB
                disk_bytes: 1024 * 1024 * 100,  // 100MB
                network_bytes_sent: 1024 * 100,
                network_bytes_received: 1024 * 50,
            },
            custom_metrics: HashMap::new(),
        })
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        if self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Unhealthy { 
                message: "Feature module is not running".to_string() 
            })
        }
    }
}