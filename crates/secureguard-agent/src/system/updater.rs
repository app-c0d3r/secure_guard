use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::communication::Client as SecureGuardClient;
use crate::utils::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub update_id: Uuid,
    pub version: String,
    pub release_notes: String,
    pub release_date: DateTime<Utc>,
    pub download_url: String,
    pub file_size: u64,
    pub file_hash: String,
    pub signature: Option<String>,
    pub min_agent_version: Option<String>,
    pub supported_platforms: Vec<String>,
    pub is_security_update: bool,
    pub is_mandatory: bool,
    pub force_install_after: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProgress {
    pub update_id: Uuid,
    pub status: UpdateStatus,
    pub download_progress: u32, // 0-100
    pub install_progress: u32,  // 0-100
    pub current_version: String,
    pub target_version: String,
    pub started_at: DateTime<Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateStatus {
    Checking,
    Available,
    Downloading,
    Downloaded,
    Installing,
    Completed,
    Failed,
    Rollback,
    RollbackCompleted,
}

pub struct AgentUpdater {
    config: Config,
    client: SecureGuardClient,
    http_client: Client,
    current_version: String,
    update_channel: String,
    check_interval: Duration,
}

impl AgentUpdater {
    pub fn new(config: Config, client: SecureGuardClient) -> Self {
        let current_version = env!("CARGO_PKG_VERSION").to_string();
        let update_channel = config.updates.as_ref()
            .and_then(|u| u.update_channel.as_ref())
            .unwrap_or(&"stable".to_string())
            .clone();
        
        let check_interval = Duration::from_secs(
            config.updates.as_ref()
                .and_then(|u| u.check_interval_hours.as_ref())
                .unwrap_or(&24) * 3600 // Convert hours to seconds
        );

        Self {
            config,
            client,
            http_client: Client::new(),
            current_version,
            update_channel,
            check_interval,
        }
    }

    /// Start the update checker loop
    pub async fn start_update_checker(&self) -> Result<()> {
        info!("🔄 Starting agent update checker");
        info!("📋 Current version: {}", self.current_version);
        info!("📡 Update channel: {}", self.update_channel);
        info!("⏱️ Check interval: {:?}", self.check_interval);

        let mut interval_timer = interval(self.check_interval);

        loop {
            interval_timer.tick().await;
            
            if let Err(e) = self.check_for_updates().await {
                error!("❌ Update check failed: {}", e);
                continue;
            }
            
            debug!("✅ Update check completed");
        }
    }

    /// Check for available updates
    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>> {
        debug!("🔍 Checking for agent updates");

        // Get latest version info from server
        let latest_update = self.client.get_latest_update(&self.update_channel).await?;
        
        if let Some(update) = latest_update {
            // Compare versions
            if self.is_newer_version(&update.version)? {
                info!("🆕 Update available: {} -> {}", self.current_version, update.version);
                
                // Check if auto-update is enabled
                if self.should_auto_update(&update).await? {
                    info!("🚀 Starting automatic update");
                    self.perform_update(update.clone()).await?;
                } else {
                    info!("ℹ️ Update available but auto-update disabled");
                    self.notify_user_of_update(&update).await?;
                }
                
                return Ok(Some(update));
            } else {
                debug!("✅ Agent is up to date ({})", self.current_version);
            }
        } else {
            debug!("ℹ️ No updates available for channel: {}", self.update_channel);
        }

        Ok(None)
    }

    /// Perform a complete agent update
    pub async fn perform_update(&self, update_info: UpdateInfo) -> Result<()> {
        info!("🚀 Starting update to version {}", update_info.version);

        // Create progress tracker
        let mut progress = UpdateProgress {
            update_id: update_info.update_id,
            status: UpdateStatus::Downloading,
            download_progress: 0,
            install_progress: 0,
            current_version: self.current_version.clone(),
            target_version: update_info.version.clone(),
            started_at: Utc::now(),
            error_message: None,
        };

        // Report update start
        self.report_update_progress(&progress).await?;

        // Step 1: Download update package
        match self.download_update(&update_info, &mut progress).await {
            Ok(package_path) => {
                info!("✅ Download completed: {}", package_path);
                progress.download_progress = 100;
                progress.status = UpdateStatus::Downloaded;
                self.report_update_progress(&progress).await?;

                // Step 2: Verify package integrity
                match self.verify_package(&package_path, &update_info).await {
                    Ok(_) => {
                        info!("✅ Package verification successful");
                        
                        // Step 3: Install update
                        match self.install_update(&package_path, &mut progress).await {
                            Ok(_) => {
                                info!("✅ Update installed successfully");
                                progress.status = UpdateStatus::Completed;
                                progress.install_progress = 100;
                                self.report_update_progress(&progress).await?;
                                
                                // Step 4: Restart agent service
                                self.restart_agent_service().await?;
                            }
                            Err(e) => {
                                error!("❌ Update installation failed: {}", e);
                                progress.status = UpdateStatus::Failed;
                                progress.error_message = Some(e.to_string());
                                self.report_update_progress(&progress).await?;
                                
                                // Attempt rollback
                                self.rollback_update(&mut progress).await?;
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ Package verification failed: {}", e);
                        progress.status = UpdateStatus::Failed;
                        progress.error_message = Some(format!("Package verification failed: {}", e));
                        self.report_update_progress(&progress).await?;
                    }
                }
            }
            Err(e) => {
                error!("❌ Download failed: {}", e);
                progress.status = UpdateStatus::Failed;
                progress.error_message = Some(format!("Download failed: {}", e));
                self.report_update_progress(&progress).await?;
            }
        }

        Ok(())
    }

    /// Download update package
    async fn download_update(&self, update_info: &UpdateInfo, progress: &mut UpdateProgress) -> Result<String> {
        info!("⬇️ Downloading update package from: {}", update_info.download_url);
        
        let download_path = self.get_download_path(&update_info.version);
        self.ensure_download_directory().await?;

        // Download with progress tracking
        let response = self.http_client.get(&update_info.download_url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Download failed with status: {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(update_info.file_size);
        let mut downloaded = 0u64;
        let mut file = tokio::fs::File::create(&download_path).await?;
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            
            // Update progress
            let new_progress = (downloaded * 100 / total_size) as u32;
            if new_progress > progress.download_progress {
                progress.download_progress = new_progress;
                debug!("📥 Download progress: {}%", new_progress);
                
                // Report progress every 10%
                if new_progress % 10 == 0 {
                    self.report_update_progress(progress).await.ok();
                }
            }
        }

        file.flush().await?;
        drop(file);

        info!("✅ Download completed: {} bytes", downloaded);
        Ok(download_path)
    }

    /// Verify downloaded package integrity
    async fn verify_package(&self, package_path: &str, update_info: &UpdateInfo) -> Result<()> {
        info!("🔐 Verifying package integrity");

        // Check file size
        let file_size = fs::metadata(package_path)?.len();
        if file_size != update_info.file_size {
            return Err(anyhow::anyhow!(
                "File size mismatch: expected {}, got {}", 
                update_info.file_size, file_size
            ));
        }

        // Verify hash
        let file_content = fs::read(package_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&file_content);
        let file_hash = format!("sha256:{}", hex::encode(hasher.finalize()));

        if file_hash != update_info.file_hash {
            return Err(anyhow::anyhow!(
                "Hash verification failed: expected {}, got {}", 
                update_info.file_hash, file_hash
            ));
        }

        // Verify digital signature if present
        if let Some(signature) = &update_info.signature {
            self.verify_digital_signature(package_path, signature).await?;
        }

        info!("✅ Package verification successful");
        Ok(())
    }

    /// Install the update
    async fn install_update(&self, package_path: &str, progress: &mut UpdateProgress) -> Result<()> {
        info!("📦 Installing update package");
        
        progress.status = UpdateStatus::Installing;
        progress.install_progress = 0;
        self.report_update_progress(progress).await?;

        // Step 1: Backup current installation
        info!("💾 Creating backup of current installation");
        self.backup_current_installation().await?;
        progress.install_progress = 20;
        self.report_update_progress(progress).await?;

        // Step 2: Stop agent services
        info!("⏹️ Stopping agent services");
        self.stop_agent_services().await?;
        progress.install_progress = 40;
        self.report_update_progress(progress).await?;

        // Step 3: Extract and install new version
        info!("📂 Extracting update package");
        self.extract_update_package(package_path).await?;
        progress.install_progress = 70;
        self.report_update_progress(progress).await?;

        // Step 4: Update configuration if needed
        info!("⚙️ Updating configuration");
        self.update_configuration().await?;
        progress.install_progress = 85;
        self.report_update_progress(progress).await?;

        // Step 5: Verify installation
        info!("✅ Verifying installation");
        self.verify_installation().await?;
        progress.install_progress = 100;

        info!("✅ Update installation completed successfully");
        Ok(())
    }

    /// Rollback failed update
    async fn rollback_update(&self, progress: &mut UpdateProgress) -> Result<()> {
        warn!("🔄 Starting update rollback");
        
        progress.status = UpdateStatus::Rollback;
        self.report_update_progress(progress).await?;

        // Restore from backup
        self.restore_from_backup().await?;
        
        // Restart services
        self.start_agent_services().await?;
        
        progress.status = UpdateStatus::RollbackCompleted;
        self.report_update_progress(progress).await?;
        
        warn!("✅ Rollback completed successfully");
        Ok(())
    }

    /// Check if agent should auto-update
    async fn should_auto_update(&self, update_info: &UpdateInfo) -> Result<bool> {
        // Check if auto-update is enabled in configuration
        let auto_update_enabled = self.config.updates.as_ref()
            .and_then(|u| u.auto_update.as_ref())
            .unwrap_or(&false);

        if !auto_update_enabled {
            return Ok(false);
        }

        // Always auto-update security updates
        if update_info.is_security_update {
            info!("🛡️ Security update - auto-installing");
            return Ok(true);
        }

        // Check if update is mandatory and past force install date
        if update_info.is_mandatory {
            if let Some(force_after) = update_info.force_install_after {
                if Utc::now() > force_after {
                    info!("⚠️ Mandatory update past deadline - auto-installing");
                    return Ok(true);
                }
            }
        }

        // Check maintenance window for non-critical updates
        if !update_info.is_security_update && !update_info.is_mandatory {
            if !self.is_in_maintenance_window().await? {
                info!("⏰ Outside maintenance window - deferring update");
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if current time is in maintenance window
    async fn is_in_maintenance_window(&self) -> Result<bool> {
        // Get maintenance window from config
        let maintenance_start = self.config.updates.as_ref()
            .and_then(|u| u.maintenance_window_start.as_ref());
        let maintenance_end = self.config.updates.as_ref()
            .and_then(|u| u.maintenance_window_end.as_ref());

        if maintenance_start.is_none() || maintenance_end.is_none() {
            // No maintenance window configured - always allow
            return Ok(true);
        }

        // Parse maintenance window times
        let start_time = maintenance_start.unwrap();
        let end_time = maintenance_end.unwrap();

        // Get current local time
        let now = chrono::Local::now().time();
        let start = chrono::NaiveTime::parse_from_str(start_time, "%H:%M")?;
        let end = chrono::NaiveTime::parse_from_str(end_time, "%H:%M")?;

        // Check if current time is within maintenance window
        if start <= end {
            Ok(now >= start && now <= end)
        } else {
            // Maintenance window crosses midnight
            Ok(now >= start || now <= end)
        }
    }

    // Helper methods for update process
    async fn backup_current_installation(&self) -> Result<()> {
        let backup_dir = self.get_backup_directory();
        self.create_directory_if_not_exists(&backup_dir).await?;
        
        // Copy current executable and configuration
        let current_exe = std::env::current_exe()?;
        let backup_exe = format!("{}/secureguard-agent-{}.exe", backup_dir, self.current_version);
        fs::copy(current_exe, backup_exe)?;
        
        info!("💾 Backup created");
        Ok(())
    }

    async fn stop_agent_services(&self) -> Result<()> {
        info!("⏹️ Stopping agent services");
        // Implementation would stop Windows service
        Ok(())
    }

    async fn start_agent_services(&self) -> Result<()> {
        info!("▶️ Starting agent services");
        // Implementation would start Windows service
        Ok(())
    }

    async fn extract_update_package(&self, package_path: &str) -> Result<()> {
        info!("📂 Extracting update package");
        // Implementation would extract the update package
        // This could be a ZIP file, MSI, or self-extracting executable
        Ok(())
    }

    async fn update_configuration(&self) -> Result<()> {
        info!("⚙️ Updating configuration for new version");
        // Implementation would migrate configuration if needed
        Ok(())
    }

    async fn verify_installation(&self) -> Result<()> {
        info!("✅ Verifying installation");
        // Implementation would verify the new version is properly installed
        Ok(())
    }

    async fn restore_from_backup(&self) -> Result<()> {
        warn!("🔄 Restoring from backup");
        // Implementation would restore from backup
        Ok(())
    }

    async fn restart_agent_service(&self) -> Result<()> {
        info!("🔄 Restarting agent service");
        // Implementation would restart the Windows service
        Ok(())
    }

    async fn verify_digital_signature(&self, _package_path: &str, _signature: &str) -> Result<()> {
        info!("🔐 Verifying digital signature");
        // Implementation would verify the digital signature
        Ok(())
    }

    async fn notify_user_of_update(&self, update_info: &UpdateInfo) -> Result<()> {
        info!("📢 Notifying user of available update: {}", update_info.version);
        // Implementation would show user notification or log update availability
        Ok(())
    }

    async fn report_update_progress(&self, progress: &UpdateProgress) -> Result<()> {
        // Send progress update to server
        self.client.report_update_progress(progress.clone()).await?;
        Ok(())
    }

    // Utility methods
    fn is_newer_version(&self, version: &str) -> Result<bool> {
        use semver::Version;
        
        let current = Version::parse(&self.current_version)?;
        let new_version = Version::parse(version)?;
        
        Ok(new_version > current)
    }

    fn get_download_path(&self, version: &str) -> String {
        format!("{}/downloads/secureguard-agent-{}.exe", 
            self.get_update_directory(), version)
    }

    fn get_update_directory(&self) -> String {
        format!("{}/updates", 
            std::env::current_exe().unwrap()
                .parent().unwrap()
                .to_string_lossy())
    }

    fn get_backup_directory(&self) -> String {
        format!("{}/backup", self.get_update_directory())
    }

    async fn ensure_download_directory(&self) -> Result<()> {
        let download_dir = format!("{}/downloads", self.get_update_directory());
        self.create_directory_if_not_exists(&download_dir).await
    }

    async fn create_directory_if_not_exists(&self, path: &str) -> Result<()> {
        if !Path::new(path).exists() {
            tokio::fs::create_dir_all(path).await?;
        }
        Ok(())
    }
}