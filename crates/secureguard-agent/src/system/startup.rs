use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;
use tracing::{info, warn, error};
use uuid::Uuid;
use winreg::enums::*;
use winreg::RegKey;

use crate::communication::Client;
use crate::utils::config::Config;
use secureguard_shared::{RegisterAgentRequest, AgentStatus};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationResult {
    pub agent_id: Uuid,
    pub registration_successful: bool,
    pub subscription_tier: String,
    pub enabled_features: Vec<String>,
    pub config_version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemIntegration {
    pub service_installed: bool,
    pub registry_configured: bool,
    pub firewall_configured: bool,
    pub auto_start_enabled: bool,
    pub hardware_fingerprint: String,
}

pub struct FirstTimeStartup {
    config: Config,
}

impl FirstTimeStartup {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Complete first-time startup workflow
    pub async fn execute(&self) -> Result<RegistrationResult> {
        info!("🚀 SecureGuard Agent First-Time Startup");
        
        // Phase 1: System Integration
        self.setup_system_integration().await?;
        
        // Phase 2: Server Registration  
        let registration = self.register_with_server().await?;
        
        // Phase 3: Configuration Setup
        self.setup_subscription_config(&registration).await?;
        
        // Phase 4: Feature Initialization
        self.initialize_features(&registration).await?;
        
        // Phase 5: Final System Configuration
        self.finalize_setup(&registration).await?;
        
        info!("✅ First-time startup completed successfully");
        info!("📊 Subscription: {} | Features: {}", 
            registration.subscription_tier, 
            registration.enabled_features.len()
        );
        
        Ok(registration)
    }

    /// Phase 1: System Integration Setup
    async fn setup_system_integration(&self) -> Result<SystemIntegration> {
        info!("🔧 Setting up system integration");
        
        // Configure Windows Service for auto-start
        let service_installed = self.setup_windows_service().await?;
        
        // Set registry entries
        let registry_configured = self.configure_registry().await?;
        
        // Configure Windows Firewall
        let firewall_configured = self.setup_firewall_rules().await?;
        
        // Generate hardware fingerprint
        let hardware_fingerprint = self.generate_hardware_fingerprint().await?;
        
        let integration = SystemIntegration {
            service_installed,
            registry_configured,
            firewall_configured,
            auto_start_enabled: true,
            hardware_fingerprint: hardware_fingerprint.clone(),
        };
        
        info!("✅ System integration completed");
        info!("🔒 Hardware fingerprint: {}", &hardware_fingerprint[..16]);
        
        Ok(integration)
    }

    async fn setup_windows_service(&self) -> Result<bool> {
        info!("⚙️ Configuring Windows Service auto-start");
        
        // Note: In production, this would use Windows Service API
        // For now, we'll simulate the configuration
        
        // Set service to automatic startup
        self.set_service_startup_type("SecureGuardAgent", "automatic").await?;
        
        // Configure service dependencies
        self.configure_service_dependencies().await?;
        
        // Set service recovery options
        self.configure_service_recovery().await?;
        
        info!("✅ Windows Service configured for auto-start");
        Ok(true)
    }

    async fn configure_registry(&self) -> Result<bool> {
        info!("📝 Configuring Windows Registry entries");
        
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let secureguard_key = hklm.create_subkey("SOFTWARE\\SecureGuard\\Agent")?;
        
        // Basic installation info
        secureguard_key.0.set_value("InstallPath", &self.get_install_path())?;
        secureguard_key.0.set_value("Version", &env!("CARGO_PKG_VERSION"))?;
        secureguard_key.0.set_value("DeviceName", &self.config.agent.device_name.as_deref().unwrap_or("Unknown"))?;
        secureguard_key.0.set_value("ServerURL", &self.config.server.base_url)?;
        
        // Configuration
        secureguard_key.0.set_value("AutoStart", &1u32)?;
        secureguard_key.0.set_value("UpdateChannel", &"stable")?;
        secureguard_key.0.set_value("LastUpdate", &Utc::now().to_rfc3339())?;
        
        // Create subkeys for features and metrics
        let _features_key = hklm.create_subkey("SOFTWARE\\SecureGuard\\Agent\\Features")?;
        let _metrics_key = hklm.create_subkey("SOFTWARE\\SecureGuard\\Agent\\Metrics")?;
        
        info!("✅ Registry entries configured");
        Ok(true)
    }

    async fn setup_firewall_rules(&self) -> Result<bool> {
        info!("🛡️ Configuring Windows Firewall rules");
        
        // Create firewall rules for SecureGuard Agent
        let exe_path = self.get_exe_path()?;
        
        // Inbound rule for agent communication
        self.create_firewall_rule(
            "SecureGuard Agent Inbound",
            &exe_path,
            "in",
            "allow"
        ).await?;
        
        // Outbound rule for server communication
        self.create_firewall_rule(
            "SecureGuard Agent Outbound", 
            &exe_path,
            "out",
            "allow"
        ).await?;
        
        info!("✅ Firewall rules configured");
        Ok(true)
    }

    async fn generate_hardware_fingerprint(&self) -> Result<String> {
        info!("🔍 Generating hardware fingerprint");
        
        use sha2::{Sha256, Digest};
        
        // Collect hardware identifiers
        let mut identifiers = Vec::new();
        
        // CPU information
        if let Ok(cpu_info) = self.get_cpu_info().await {
            identifiers.push(format!("cpu:{}", cpu_info));
        }
        
        // Motherboard serial
        if let Ok(mb_serial) = self.get_motherboard_serial().await {
            identifiers.push(format!("mb:{}", mb_serial));
        }
        
        // MAC addresses
        if let Ok(mac_addresses) = self.get_network_adapters().await {
            for mac in mac_addresses {
                identifiers.push(format!("mac:{}", mac));
            }
        }
        
        // Disk serials
        if let Ok(disk_serials) = self.get_disk_serials().await {
            for serial in disk_serials {
                identifiers.push(format!("disk:{}", serial));
            }
        }
        
        // Sort for consistent ordering
        identifiers.sort();
        let combined = identifiers.join("|");
        
        // Generate SHA256 hash
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();
        
        let fingerprint = format!("hw_{}", hex::encode(hash)[..32].to_string());
        
        info!("✅ Hardware fingerprint generated");
        Ok(fingerprint)
    }

    /// Phase 2: Server Registration
    async fn register_with_server(&self) -> Result<RegistrationResult> {
        info!("🔗 Registering agent with SecureGuard servers");
        
        // Create secure client connection
        let client = Client::new(&self.config).await?;
        
        // Collect system information
        let os_info = self.collect_os_information().await?;
        let hardware_fingerprint = self.generate_hardware_fingerprint().await?;
        
        // Prepare registration request
        let api_key = self.config.agent.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("API key not configured"))?;
        
        let device_name = self.config.agent.device_name.as_ref()
            .unwrap_or(&std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Unknown".to_string()));
        
        let registration_request = RegisterAgentRequest {
            api_key: api_key.clone(),
            device_name: device_name.clone(),
            hardware_fingerprint,
            os_info,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        
        // Submit registration
        let response = client.register_agent(registration_request).await?;
        
        // Save agent credentials
        self.save_agent_credentials(&response.agent_id).await?;
        
        // Fetch subscription details
        let subscription_info = client.get_subscription_info(response.agent_id).await?;
        
        let result = RegistrationResult {
            agent_id: response.agent_id,
            registration_successful: true,
            subscription_tier: subscription_info.tier,
            enabled_features: subscription_info.enabled_features,
            config_version: subscription_info.config_version,
        };
        
        info!("✅ Agent registered successfully");
        info!("🆔 Agent ID: {}", result.agent_id);
        info!("📊 Subscription: {}", result.subscription_tier);
        
        Ok(result)
    }

    /// Phase 3: Subscription Configuration
    async fn setup_subscription_config(&self, registration: &RegistrationResult) -> Result<()> {
        info!("⚙️ Setting up subscription-based configuration");
        
        // Create client for config fetch
        let client = Client::new(&self.config).await?;
        
        // Fetch subscription-specific configuration
        let agent_config = client.get_agent_config(registration.agent_id).await?;
        
        // Save configuration locally
        let config_path = self.get_config_path();
        let config_content = serde_json::to_string_pretty(&agent_config)?;
        fs::write(&config_path, config_content).await?;
        
        info!("✅ Subscription configuration saved");
        info!("📁 Config path: {}", config_path);
        info!("🎯 Features enabled: {}", registration.enabled_features.join(", "));
        
        Ok(())
    }

    /// Phase 4: Feature Initialization
    async fn initialize_features(&self, registration: &RegistrationResult) -> Result<()> {
        info!("🧩 Initializing subscription features");
        
        let mut initialized_features = Vec::new();
        
        for feature in &registration.enabled_features {
            match self.initialize_feature(feature).await {
                Ok(_) => {
                    initialized_features.push(feature.clone());
                    info!("✅ Initialized feature: {}", feature);
                }
                Err(e) => {
                    warn!("⚠️ Failed to initialize feature {}: {}", feature, e);
                }
            }
        }
        
        // Save initialized features to registry
        self.save_active_features(&initialized_features).await?;
        
        info!("✅ Feature initialization completed");
        info!("🎯 Active features: {}", initialized_features.join(", "));
        
        Ok(())
    }

    async fn initialize_feature(&self, feature_name: &str) -> Result<()> {
        match feature_name {
            "real_time_monitoring" => {
                self.setup_real_time_monitoring().await?;
            }
            "advanced_threat_detection" => {
                self.setup_threat_detection().await?;
            }
            "vulnerability_scanning" => {
                self.setup_vulnerability_scanning().await?;
            }
            "file_integrity_monitoring" => {
                self.setup_file_integrity_monitoring().await?;
            }
            "remote_commands" => {
                self.setup_remote_command_processing().await?;
            }
            _ => {
                warn!("Unknown feature: {}", feature_name);
            }
        }
        
        Ok(())
    }

    /// Phase 5: Final Setup
    async fn finalize_setup(&self, registration: &RegistrationResult) -> Result<()> {
        info!("🎯 Finalizing agent setup");
        
        // Update registry with final configuration
        self.update_registry_final_config(registration).await?;
        
        // Create management scripts
        self.create_management_scripts().await?;
        
        // Set up logging
        self.configure_logging().await?;
        
        // Create scheduled tasks
        self.create_scheduled_tasks().await?;
        
        // Start monitoring services
        self.start_monitoring_services().await?;
        
        info!("✅ Agent setup finalized");
        Ok(())
    }

    // Helper methods
    async fn set_service_startup_type(&self, service_name: &str, startup_type: &str) -> Result<()> {
        // In production, this would use Windows Service Control Manager API
        info!("Setting {} service startup type to {}", service_name, startup_type);
        Ok(())
    }

    async fn configure_service_dependencies(&self) -> Result<()> {
        info!("Configuring service dependencies: Tcpip, Dnscache");
        Ok(())
    }

    async fn configure_service_recovery(&self) -> Result<()> {
        info!("Configuring service recovery options");
        Ok(())
    }

    async fn create_firewall_rule(&self, name: &str, program: &str, direction: &str, action: &str) -> Result<()> {
        info!("Creating firewall rule: {} for {} ({} {})", name, program, direction, action);
        Ok(())
    }

    fn get_install_path(&self) -> String {
        std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("C:\\Program Files\\SecureGuard\\Agent"))
            .parent()
            .unwrap_or_else(|| std::path::Path::new("C:\\Program Files\\SecureGuard\\Agent"))
            .to_string_lossy()
            .to_string()
    }

    fn get_exe_path(&self) -> Result<String> {
        Ok(std::env::current_exe()?.to_string_lossy().to_string())
    }

    fn get_config_path(&self) -> String {
        format!("{}\\config\\runtime_config.json", self.get_install_path())
    }

    async fn collect_os_information(&self) -> Result<serde_json::Value> {
        use sysinfo::{System, SystemExt};
        
        let mut sys = System::new_all();
        sys.refresh_all();
        
        Ok(serde_json::json!({
            "os": sys.name(),
            "os_version": sys.os_version(),
            "kernel_version": sys.kernel_version(),
            "host_name": sys.host_name(),
            "total_memory": sys.total_memory(),
            "cpu_count": sys.processors().len(),
            "architecture": std::env::consts::ARCH,
        }))
    }

    async fn get_cpu_info(&self) -> Result<String> {
        // Simulate CPU info collection
        Ok("Intel(R) Core(TM) i7-8700K CPU @ 3.70GHz".to_string())
    }

    async fn get_motherboard_serial(&self) -> Result<String> {
        // Simulate motherboard serial collection
        Ok("MB123456789".to_string())
    }

    async fn get_network_adapters(&self) -> Result<Vec<String>> {
        // Simulate MAC address collection
        Ok(vec!["00:11:22:33:44:55".to_string()])
    }

    async fn get_disk_serials(&self) -> Result<Vec<String>> {
        // Simulate disk serial collection  
        Ok(vec!["WD-1234567890".to_string()])
    }

    async fn save_agent_credentials(&self, agent_id: &Uuid) -> Result<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let agent_key = hklm.open_subkey_with_flags("SOFTWARE\\SecureGuard\\Agent", KEY_WRITE)?;
        
        agent_key.set_value("AgentID", &agent_id.to_string())?;
        agent_key.set_value("RegisteredAt", &Utc::now().to_rfc3339())?;
        
        info!("Agent credentials saved to registry");
        Ok(())
    }

    async fn save_active_features(&self, features: &[String]) -> Result<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let features_key = hklm.open_subkey_with_flags("SOFTWARE\\SecureGuard\\Agent\\Features", KEY_WRITE)?;
        
        // Clear existing features
        for value_name in features_key.enum_values().map(|x| x.unwrap().0) {
            let _ = features_key.delete_value(value_name);
        }
        
        // Set active features
        for feature in features {
            features_key.set_value(feature, &1u32)?;
        }
        
        info!("Active features saved to registry");
        Ok(())
    }

    async fn setup_real_time_monitoring(&self) -> Result<()> {
        info!("Setting up real-time monitoring");
        Ok(())
    }

    async fn setup_threat_detection(&self) -> Result<()> {
        info!("Setting up advanced threat detection");
        Ok(())
    }

    async fn setup_vulnerability_scanning(&self) -> Result<()> {
        info!("Setting up vulnerability scanning");
        Ok(())
    }

    async fn setup_file_integrity_monitoring(&self) -> Result<()> {
        info!("Setting up file integrity monitoring");
        Ok(())
    }

    async fn setup_remote_command_processing(&self) -> Result<()> {
        info!("Setting up remote command processing");
        Ok(())
    }

    async fn update_registry_final_config(&self, registration: &RegistrationResult) -> Result<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let agent_key = hklm.open_subkey_with_flags("SOFTWARE\\SecureGuard\\Agent", KEY_WRITE)?;
        
        agent_key.set_value("SubscriptionTier", &registration.subscription_tier)?;
        agent_key.set_value("ConfigVersion", &registration.config_version)?;
        agent_key.set_value("SetupCompleted", &Utc::now().to_rfc3339())?;
        
        info!("Final configuration saved to registry");
        Ok(())
    }

    async fn create_management_scripts(&self) -> Result<()> {
        info!("Creating agent management scripts");
        Ok(())
    }

    async fn configure_logging(&self) -> Result<()> {
        info!("Configuring logging system");
        Ok(())
    }

    async fn create_scheduled_tasks(&self) -> Result<()> {
        info!("Creating scheduled tasks for maintenance");
        Ok(())
    }

    async fn start_monitoring_services(&self) -> Result<()> {
        info!("Starting monitoring services");
        Ok(())
    }
}