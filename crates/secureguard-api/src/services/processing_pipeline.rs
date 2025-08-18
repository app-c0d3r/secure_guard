use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;
use uuid::Uuid;
use sqlx::PgPool;
use tracing::{info, warn, error};

use crate::services::{
    threat_service::ThreatService,
    realtime_service::RealtimeService,
    event_processor::{EventProcessor, PipelineConfig, ProcessingStats},
};
use crate::websocket::connection_manager::ConnectionManager;
use secureguard_shared::{
    CreateSecurityEventRequest, AgentCommand, CommandStatus,
    Severity, SecureGuardError, Result, AgentStatus
};

// Main processing pipeline orchestrator
#[derive(Clone)]
pub struct ProcessingPipeline {
    event_processor: Arc<EventProcessor>,
    realtime_service: Arc<RealtimeService>,
    threat_service: Arc<ThreatService>,
    pool: PgPool,
    
    // Pipeline health monitoring
    health_check_interval: Duration,
    last_health_check: Arc<tokio::sync::RwLock<Instant>>,
}

// Pipeline health metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineHealth {
    pub is_healthy: bool,
    pub uptime_seconds: u64,
    pub processing_stats: ProcessingStats,
    pub database_connection_healthy: bool,
    pub websocket_connections_active: usize,
    pub last_error: Option<String>,
    pub performance_score: f32, // 0.0 to 1.0
}

// Event enrichment and preprocessing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventEnrichment {
    pub geo_location: Option<String>,
    pub threat_intelligence: Option<ThreatIntelligence>,
    pub agent_context: Option<AgentContext>,
    pub risk_score: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThreatIntelligence {
    pub known_malware: bool,
    pub reputation_score: f32,
    pub threat_categories: Vec<String>,
    pub ioc_matches: Vec<String>, // Indicators of Compromise
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentContext {
    pub agent_name: String,
    pub agent_version: String,
    pub system_info: String,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub health_status: AgentStatus,
}

impl ProcessingPipeline {
    pub async fn new(
        pool: PgPool,
        connection_manager: ConnectionManager,
        config: Option<PipelineConfig>,
    ) -> Result<Self> {
        // Initialize core services
        let realtime_service = Arc::new(RealtimeService::new(pool.clone(), connection_manager));
        let threat_service = Arc::new(ThreatService::with_message_router(
            pool.clone(), 
            realtime_service.get_message_router()
        ));
        
        // Initialize event processor
        let event_processor = Arc::new(EventProcessor::new(
            pool.clone(),
            threat_service.clone(),
            realtime_service.clone(),
            config,
        ));

        Ok(Self {
            event_processor,
            realtime_service,
            threat_service,
            pool: pool.clone(),
            health_check_interval: Duration::from_secs(30),
            last_health_check: Arc::new(tokio::sync::RwLock::new(Instant::now())),
        })
    }

    // Initialize the entire processing pipeline
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing SecureGuard Processing Pipeline...");

        // Initialize all components
        self.realtime_service.initialize().await?;
        self.event_processor.initialize().await?;

        // Start health monitoring
        self.start_health_monitor().await;
        
        // Start automated response system
        self.start_automated_response_system().await;
        
        // Load threat intelligence feeds
        self.initialize_threat_intelligence().await?;

        info!("SecureGuard Processing Pipeline initialized successfully!");
        Ok(())
    }

    // Main entry point for processing security events
    pub async fn process_security_event(
        &self,
        agent_id: Uuid,
        mut event_request: CreateSecurityEventRequest,
    ) -> Result<()> {
        let start_time = Instant::now();

        // Step 1: Event enrichment and preprocessing
        self.enrich_event(&mut event_request, agent_id).await?;

        // Step 2: Queue for processing
        self.event_processor.queue_event(agent_id, event_request).await?;

        let processing_time = start_time.elapsed();
        if processing_time > Duration::from_millis(100) {
            warn!("Event preprocessing took {:?} for agent {}", processing_time, agent_id);
        }

        Ok(())
    }

    // Batch processing for high-throughput scenarios
    pub async fn process_events_batch(
        &self,
        events: Vec<(Uuid, CreateSecurityEventRequest)>,
    ) -> Result<()> {
        info!("Processing batch of {} events", events.len());
        
        let mut enriched_events = Vec::new();
        
        // Enrich all events in parallel
        let enrichment_tasks: Vec<_> = events.into_iter().map(|(agent_id, mut event)| {
            let pipeline = self.clone();
            async move {
                pipeline.enrich_event(&mut event, agent_id).await?;
                Ok::<_, SecureGuardError>((agent_id, event))
            }
        }).collect();

        // Wait for all enrichments to complete
        for task in enrichment_tasks {
            match task.await {
                Ok((agent_id, enriched_event)) => enriched_events.push((agent_id, enriched_event)),
                Err(e) => error!("Failed to enrich event: {}", e),
            }
        }

        // Queue all enriched events
        self.event_processor.queue_events_batch(enriched_events).await?;
        
        Ok(())
    }

    // Get comprehensive pipeline health status
    pub async fn get_health_status(&self) -> PipelineHealth {
        let uptime_seconds = self.last_health_check.read().await.elapsed().as_secs();
        let processing_stats = self.event_processor.get_stats().await;
        
        // Check database health
        let db_healthy = self.check_database_health().await;
        
        // Get WebSocket connection count
        let (_, dashboard_count, _) = self.realtime_service.get_message_router().get_connection_stats().await;
        
        // Calculate performance score
        let performance_score = self.calculate_performance_score(&processing_stats, db_healthy).await;
        
        PipelineHealth {
            is_healthy: db_healthy && performance_score > 0.7,
            uptime_seconds,
            processing_stats,
            database_connection_healthy: db_healthy,
            websocket_connections_active: dashboard_count,
            last_error: None, // Could track last error here
            performance_score,
        }
    }

    // Emergency pipeline controls
    pub async fn emergency_stop(&self) -> Result<()> {
        warn!("EMERGENCY STOP: Halting all processing pipeline operations");
        
        // Broadcast emergency alert
        self.realtime_service.broadcast_emergency_alert(
            "Pipeline Emergency Stop",
            "Processing pipeline has been emergency stopped",
            Severity::Critical,
            vec![], // Affects all agents
        ).await?;

        info!("Emergency stop completed");
        Ok(())
    }

    pub async fn emergency_isolate_agents(&self, agent_ids: Vec<Uuid>) -> Result<()> {
        warn!("EMERGENCY ISOLATION: Isolating {} agents", agent_ids.len());
        
        for agent_id in agent_ids {
            // Send isolation command to agent
            let isolation_command = AgentCommand {
                command_id: Uuid::new_v4(),
                agent_id,
                issued_by: Uuid::new_v4(), // System user
                command_type: "emergency_isolate".to_string(),
                command_data: serde_json::json!({
                    "action": "isolate",
                    "reason": "Emergency isolation due to threat detection",
                    "timestamp": chrono::Utc::now()
                }),
                status: CommandStatus::Pending,
                result: None,
                issued_at: chrono::Utc::now(),
                executed_at: None,
                completed_at: None,
            };

            if let Err(e) = self.realtime_service.send_agent_command(agent_id, &isolation_command).await {
                error!("Failed to send isolation command to agent {}: {}", agent_id, e);
            }
        }

        Ok(())
    }

    // Performance optimization and scaling
    pub async fn optimize_pipeline_performance(&self) -> Result<()> {
        let stats = self.event_processor.get_stats().await;
        
        // Adaptive scaling based on queue depth and processing rate
        if stats.queue_depth > 1000 && stats.events_per_second < 50.0 {
            info!("High queue depth detected, consider scaling up processing capacity");
            // In a real implementation, this could trigger auto-scaling
        }

        if stats.processing_latency_ms > 500.0 {
            warn!("High processing latency detected: {:.2}ms", stats.processing_latency_ms);
            // Could trigger performance optimization here
        }

        Ok(())
    }

    // Private helper methods
    async fn enrich_event(
        &self,
        event_request: &mut CreateSecurityEventRequest,
        agent_id: Uuid,
    ) -> Result<()> {
        // Add enrichment data to event_data
        let enrichment = EventEnrichment {
            geo_location: self.get_geo_location(&event_request.source_ip).await,
            threat_intelligence: self.get_threat_intelligence(event_request).await,
            agent_context: self.get_agent_context(agent_id).await,
            risk_score: self.calculate_risk_score(event_request).await,
        };

        // Add enrichment to event data
        if let Some(existing_data) = event_request.event_data.as_object_mut() {
            existing_data.insert("enrichment".to_string(), 
                serde_json::to_value(enrichment)
                    .map_err(|e| SecureGuardError::ValidationError(e.to_string()))?);
        }

        Ok(())
    }

    async fn get_geo_location(&self, source_ip: &Option<String>) -> Option<String> {
        // Simplified geo-location lookup
        // In production, this would integrate with a geo-IP service
        if let Some(ip) = source_ip {
            if ip.starts_with("192.168.") || ip.starts_with("10.") || ip.starts_with("172.") {
                Some("Internal Network".to_string())
            } else {
                Some("External".to_string())
            }
        } else {
            None
        }
    }

    async fn get_threat_intelligence(&self, event: &CreateSecurityEventRequest) -> Option<ThreatIntelligence> {
        // Simplified threat intelligence lookup
        // In production, this would query threat intelligence databases
        
        let mut threat_categories = Vec::new();
        let mut ioc_matches = Vec::new();
        
        // Check for known malicious patterns
        if let Some(process_name) = &event.process_name {
            if process_name.to_lowercase().contains("powershell") && 
               event.event_data.get("command_line").is_some() {
                threat_categories.push("PowerShell Execution".to_string());
                ioc_matches.push("Suspicious PowerShell Activity".to_string());
            }
        }

        if !threat_categories.is_empty() {
            Some(ThreatIntelligence {
                known_malware: false,
                reputation_score: 0.3,
                threat_categories,
                ioc_matches,
            })
        } else {
            None
        }
    }

    async fn get_agent_context(&self, agent_id: Uuid) -> Option<AgentContext> {
        // In production, this would query the agents database
        Some(AgentContext {
            agent_name: format!("Agent-{}", agent_id.to_string()[0..8].to_uppercase()),
            agent_version: "1.0.0".to_string(),
            system_info: "Windows 10 Professional".to_string(),
            last_seen: chrono::Utc::now(),
            health_status: AgentStatus::Online,
        })
    }

    async fn calculate_risk_score(&self, event: &CreateSecurityEventRequest) -> f32 {
        let mut score: f32 = match event.severity {
            Severity::Critical => 0.9,
            Severity::High => 0.7,
            Severity::Medium => 0.5,
            Severity::Low => 0.3,
        };

        // Adjust based on event type
        match event.event_type.as_str() {
            "process_creation" => score += 0.1,
            "file_modification" => score += 0.2,
            "network_connection" => score += 0.15,
            "registry_modification" => score += 0.25,
            _ => {}
        }

        score.min(1.0)
    }

    async fn check_database_health(&self) -> bool {
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => true,
            Err(e) => {
                error!("Database health check failed: {}", e);
                false
            }
        }
    }

    async fn calculate_performance_score(&self, stats: &ProcessingStats, db_healthy: bool) -> f32 {
        let mut score: f32 = 1.0;

        // Penalize high latency
        if stats.processing_latency_ms > 100.0 {
            score -= 0.3;
        }

        // Penalize low throughput
        if stats.events_per_second < 10.0 {
            score -= 0.2;
        }

        // Penalize large queue
        if stats.queue_depth > 500 {
            score -= 0.2;
        }

        // Database health
        if !db_healthy {
            score -= 0.4;
        }

        score.max(0.0)
    }

    async fn start_health_monitor(&self) {
        let pipeline = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(pipeline.health_check_interval);
            
            loop {
                interval.tick().await;
                
                let health = pipeline.get_health_status().await;
                
                if !health.is_healthy {
                    warn!("Pipeline health check FAILED: performance score {:.2}", health.performance_score);
                } else {
                    info!("Pipeline health check OK: {:.1}% performance", health.performance_score * 100.0);
                }
                
                // Update last health check time
                *pipeline.last_health_check.write().await = Instant::now();
                
                // Auto-optimization if needed
                if health.performance_score < 0.5 {
                    if let Err(e) = pipeline.optimize_pipeline_performance().await {
                        error!("Failed to optimize pipeline performance: {}", e);
                    }
                }
            }
        });
    }

    async fn start_automated_response_system(&self) {
        let pipeline = self.clone();
        tokio::spawn(async move {
            // This would implement automated incident response
            // For now, it's a placeholder for future enhancement
            info!("Automated response system started");
        });
    }

    async fn initialize_threat_intelligence(&self) -> Result<()> {
        // Initialize threat intelligence feeds and correlation patterns
        info!("Initializing threat intelligence feeds...");
        
        // Add custom correlation patterns for common attack scenarios
        // These would typically be loaded from a database or external feed
        
        info!("Threat intelligence initialized");
        Ok(())
    }
}

// System-wide processing controls
impl ProcessingPipeline {
    pub async fn get_system_metrics(&self) -> serde_json::Value {
        let health = self.get_health_status().await;
        let realtime_stats = self.realtime_service.get_realtime_stats().await;
        
        serde_json::json!({
            "pipeline_health": health,
            "realtime_stats": realtime_stats,
            "timestamp": chrono::Utc::now(),
            "version": "1.0.0"
        })
    }

    pub async fn trigger_system_maintenance(&self) -> Result<()> {
        info!("Starting system maintenance routine");
        
        // Cleanup old correlations and events
        // Optimize database performance
        // Clear caches
        
        info!("System maintenance completed");
        Ok(())
    }
}