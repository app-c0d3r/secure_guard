use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Agent configuration structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub agent: AgentConfig,
    pub server: ServerConfig,
    pub monitoring: MonitoringConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub privacy: PrivacyConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentConfig {
    pub version: String,
    pub agent_id: Option<String>,
    pub installation_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub endpoint: String,
    pub api_endpoint: String,
    pub certificate_path: Option<PathBuf>,
    pub verify_ssl: bool,
    pub connection_timeout: u64,
    pub retry_attempts: u32,
    pub retry_delay: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoringConfig {
    pub heartbeat_interval: u64,
    pub data_collection_interval: u64,
    pub cpu_threshold: f64,
    pub memory_threshold: f64,
    pub disk_threshold: f64,
    pub enable_performance_monitoring: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub enable_file_monitoring: bool,
    pub enable_network_monitoring: bool,
    pub enable_process_monitoring: bool,
    pub scan_interval: u64,
    pub threat_detection_level: ThreatDetectionLevel,
    pub quarantine_directory: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ThreatDetectionLevel {
    Low,
    Medium,
    High,
    Paranoid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub local_logs: bool,
    pub remote_logs: bool,
    pub max_log_size: String,
    pub log_rotation: u32,
    pub log_directory: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivacyConfig {
    pub collect_personal_data: bool,
    pub anonymize_data: bool,
    pub data_retention_days: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            agent: AgentConfig {
                version: env!("CARGO_PKG_VERSION").to_string(),
                agent_id: None,
                installation_date: None,
            },
            server: ServerConfig {
                endpoint: "wss://secureguard.company.com/agent".to_string(),
                api_endpoint: "https://api.secureguard.company.com".to_string(),
                certificate_path: None,
                verify_ssl: true,
                connection_timeout: 30,
                retry_attempts: 3,
                retry_delay: 5,
            },
            monitoring: MonitoringConfig {
                heartbeat_interval: 30,
                data_collection_interval: 300,
                cpu_threshold: 80.0,
                memory_threshold: 85.0,
                disk_threshold: 90.0,
                enable_performance_monitoring: true,
            },
            security: SecurityConfig {
                enable_file_monitoring: true,
                enable_network_monitoring: true,
                enable_process_monitoring: true,
                scan_interval: 3600,
                threat_detection_level: ThreatDetectionLevel::Medium,
                quarantine_directory: None,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                local_logs: true,
                remote_logs: true,
                max_log_size: "100MB".to_string(),
                log_rotation: 7,
                log_directory: None,
            },
            privacy: PrivacyConfig {
                collect_personal_data: false,
                anonymize_data: true,
                data_retention_days: 30,
            },
        }
    }
}

impl Config {
    /// Load configuration from file or create default
    pub async fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if config_path.exists() {
            Self::load_from_file(&config_path).await
        } else {
            let config = Self::default();
            config.save_to_file(&config_path).await?;
            Ok(config)
        }
    }

    /// Load configuration from specific file
    pub async fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file
    pub async fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = toml::to_string_pretty(self)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Get the configuration file path based on platform
    pub fn get_config_path() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let mut path = PathBuf::from(
                std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string()),
            );
            path.push("SecureGuard");
            path.push("config.toml");
            Ok(path)
        }

        #[cfg(target_os = "macos")]
        {
            let mut path = PathBuf::from("/Library/Application Support");
            path.push("SecureGuard");
            path.push("config.toml");
            Ok(path)
        }

        #[cfg(target_os = "linux")]
        {
            let mut path = PathBuf::from("/etc");
            path.push("secureguard");
            path.push("config.toml");
            Ok(path)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            let mut path = std::env::current_dir()?;
            path.push("config.toml");
            Ok(path)
        }
    }

    /// Get the data directory path based on platform
    pub fn get_data_directory() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let mut path = PathBuf::from(
                std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string()),
            );
            path.push("SecureGuard");
            Ok(path)
        }

        #[cfg(target_os = "macos")]
        {
            let mut path = PathBuf::from("/Library/Application Support");
            path.push("SecureGuard");
            Ok(path)
        }

        #[cfg(target_os = "linux")]
        {
            let mut path = PathBuf::from("/var/lib");
            path.push("secureguard");
            Ok(path)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            std::env::current_dir().map_err(Into::into)
        }
    }

    /// Update configuration with new values
    pub async fn update(&mut self, updates: serde_json::Value) -> Result<()> {
        // Merge updates into current configuration
        // This is a simplified implementation - in production you'd want more sophisticated merging
        let current_json = serde_json::to_value(&*self)?;
        let merged = merge_json_values(current_json, updates)?;
        *self = serde_json::from_value(merged)?;

        // Save updated configuration
        let config_path = Self::get_config_path()?;
        self.save_to_file(&config_path).await?;

        Ok(())
    }
}

/// Simple JSON value merging function
fn merge_json_values(
    base: serde_json::Value,
    updates: serde_json::Value,
) -> Result<serde_json::Value> {
    use serde_json::{Map, Value};

    match (base, updates) {
        (Value::Object(mut base_map), Value::Object(update_map)) => {
            for (key, value) in update_map {
                match base_map.get(&key) {
                    Some(base_value) => {
                        base_map.insert(key, merge_json_values(base_value.clone(), value)?);
                    }
                    None => {
                        base_map.insert(key, value);
                    }
                }
            }
            Ok(Value::Object(base_map))
        }
        (_, update_value) => Ok(update_value),
    }
}
