use sqlx::PgPool;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::services::{realtime_service::RealtimeService, threat_service::ThreatService};
use secureguard_shared::{
    AgentCommand, AlertStatus, CommandStatus, CreateSecurityEventRequest, Result, SecureGuardError,
    SecurityEvent, Severity, ThreatAlert,
};

// Event processing statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProcessingStats {
    pub events_processed: u64,
    pub events_per_second: f64,
    pub alerts_generated: u64,
    pub processing_latency_ms: f64,
    pub queue_depth: usize,
    pub correlation_hits: u64,
    pub auto_responses_triggered: u64,
}

// Event correlation window for pattern detection
#[derive(Debug, Clone)]
pub struct EventCorrelation {
    pub correlation_id: Uuid,
    pub agent_ids: Vec<Uuid>,
    pub event_types: Vec<String>,
    pub severity_level: Severity,
    pub first_seen: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub event_count: u32,
    pub pattern_confidence: f32, // 0.0 to 1.0
}

// Processing pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub max_concurrent_processors: usize,
    pub batch_size: usize,
    pub correlation_window_seconds: u64,
    pub high_priority_threshold: Severity,
    pub auto_response_enabled: bool,
    pub max_queue_size: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_processors: 10,
            batch_size: 100,
            correlation_window_seconds: 300, // 5 minutes
            high_priority_threshold: Severity::High,
            auto_response_enabled: true,
            max_queue_size: 10000,
        }
    }
}

// Main event processing pipeline
#[derive(Clone)]
pub struct EventProcessor {
    pool: PgPool,
    threat_service: Arc<ThreatService>,
    realtime_service: Arc<RealtimeService>,
    config: PipelineConfig,

    // Processing queues
    high_priority_queue: Arc<RwLock<VecDeque<(Uuid, CreateSecurityEventRequest, Instant)>>>,
    normal_priority_queue: Arc<RwLock<VecDeque<(Uuid, CreateSecurityEventRequest, Instant)>>>,

    // Correlation engine
    active_correlations: Arc<RwLock<HashMap<String, EventCorrelation>>>,
    correlation_patterns: Arc<RwLock<Vec<CorrelationPattern>>>,

    // Statistics and monitoring
    stats: Arc<RwLock<ProcessingStats>>,

    // Concurrency control
    processing_semaphore: Arc<Semaphore>,
}

// Pattern definitions for correlation detection
#[derive(Debug, Clone)]
pub struct CorrelationPattern {
    pub pattern_id: String,
    pub name: String,
    pub description: String,
    pub event_sequence: Vec<String>,
    pub max_time_window: Duration,
    pub min_agents: usize,
    pub confidence_threshold: f32,
    pub auto_response: Option<String>,
}

impl EventProcessor {
    pub fn new(
        pool: PgPool,
        threat_service: Arc<ThreatService>,
        realtime_service: Arc<RealtimeService>,
        config: Option<PipelineConfig>,
    ) -> Self {
        let config = config.unwrap_or_default();
        let processing_semaphore = Arc::new(Semaphore::new(config.max_concurrent_processors));

        Self {
            pool,
            threat_service,
            realtime_service,
            config: config.clone(),
            high_priority_queue: Arc::new(RwLock::new(VecDeque::new())),
            normal_priority_queue: Arc::new(RwLock::new(VecDeque::new())),
            active_correlations: Arc::new(RwLock::new(HashMap::new())),
            correlation_patterns: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(ProcessingStats::default())),
            processing_semaphore,
        }
    }

    // Initialize the event processor with default patterns
    pub async fn initialize(&self) -> Result<()> {
        // Load default correlation patterns
        self.load_default_patterns().await;

        // Start background processing tasks
        self.start_processing_workers().await;
        self.start_correlation_engine().await;
        self.start_stats_collector().await;
        self.start_queue_monitor().await;

        info!(
            "Event processor initialized with {} correlation patterns",
            self.correlation_patterns.read().await.len()
        );
        Ok(())
    }

    // Main entry point for event processing
    pub async fn queue_event(
        &self,
        agent_id: Uuid,
        event_request: CreateSecurityEventRequest,
    ) -> Result<()> {
        let enqueue_time = Instant::now();

        // Check queue capacity
        let total_queue_size = self.get_total_queue_size().await;
        if total_queue_size >= self.config.max_queue_size {
            return Err(SecureGuardError::ValidationError(
                "Event processing queue is full".to_string(),
            ));
        }

        // Determine priority based on severity
        let is_high_priority = self.is_high_priority(&event_request);

        if is_high_priority {
            let mut queue = self.high_priority_queue.write().await;
            queue.push_back((agent_id, event_request, enqueue_time));
            debug!("Queued high-priority event from agent {}", agent_id);
        } else {
            let mut queue = self.normal_priority_queue.write().await;
            queue.push_back((agent_id, event_request, enqueue_time));
            debug!("Queued normal-priority event from agent {}", agent_id);
        }

        Ok(())
    }

    // Batch processing for high-throughput scenarios
    pub async fn queue_events_batch(
        &self,
        events: Vec<(Uuid, CreateSecurityEventRequest)>,
    ) -> Result<()> {
        let enqueue_time = Instant::now();
        let mut high_priority_batch = Vec::new();
        let mut normal_priority_batch = Vec::new();

        // Sort events by priority
        for (agent_id, event_request) in events {
            if self.is_high_priority(&event_request) {
                high_priority_batch.push((agent_id, event_request, enqueue_time));
            } else {
                normal_priority_batch.push((agent_id, event_request, enqueue_time));
            }
        }

        // Count batches before moving
        let high_priority_count = high_priority_batch.len();
        let normal_priority_count = normal_priority_batch.len();

        // Batch enqueue
        if !high_priority_batch.is_empty() {
            let mut queue = self.high_priority_queue.write().await;
            queue.extend(high_priority_batch);
        }

        if !normal_priority_batch.is_empty() {
            let mut queue = self.normal_priority_queue.write().await;
            queue.extend(normal_priority_batch);
        }

        info!(
            "Batch queued {} events for processing",
            high_priority_count + normal_priority_count
        );

        Ok(())
    }

    // Get current processing statistics
    pub async fn get_stats(&self) -> ProcessingStats {
        let mut stats = self.stats.read().await.clone();
        stats.queue_depth = self.get_total_queue_size().await;
        stats
    }

    // Add custom correlation pattern
    pub async fn add_correlation_pattern(&self, pattern: CorrelationPattern) -> Result<()> {
        let mut patterns = self.correlation_patterns.write().await;
        patterns.push(pattern.clone());
        info!("Added correlation pattern: {}", pattern.name);
        Ok(())
    }

    // Private helper methods
    async fn get_total_queue_size(&self) -> usize {
        let high_priority_size = self.high_priority_queue.read().await.len();
        let normal_priority_size = self.normal_priority_queue.read().await.len();
        high_priority_size + normal_priority_size
    }

    fn is_high_priority(&self, event: &CreateSecurityEventRequest) -> bool {
        match (&event.severity, &self.config.high_priority_threshold) {
            (Severity::Critical, _) => true,
            (Severity::High, Severity::Medium | Severity::Low) => true,
            (Severity::Medium, Severity::Low) => true,
            _ => false,
        }
    }

    // Background processing workers
    async fn start_processing_workers(&self) {
        let processor_clone = self.clone();
        tokio::spawn(async move {
            processor_clone.high_priority_worker().await;
        });

        let processor_clone = self.clone();
        tokio::spawn(async move {
            processor_clone.normal_priority_worker().await;
        });

        info!("Started event processing workers");
    }

    async fn high_priority_worker(&self) {
        let mut interval = time::interval(Duration::from_millis(10)); // 100 Hz processing

        loop {
            interval.tick().await;

            // Process high-priority events first
            while let Some((agent_id, event_request, enqueue_time)) = {
                let mut queue = self.high_priority_queue.write().await;
                queue.pop_front()
            } {
                if let Ok(_permit) = self.processing_semaphore.try_acquire() {
                    let processor = self.clone();
                    tokio::spawn(async move {
                        let processing_start = Instant::now();

                        match processor
                            .process_single_event(agent_id, event_request)
                            .await
                        {
                            Ok(event) => {
                                let latency = processing_start.elapsed();
                                processor
                                    .update_stats(latency, enqueue_time.elapsed())
                                    .await;
                                debug!(
                                    "Processed high-priority event {} in {:?}",
                                    event.event_id, latency
                                );
                            }
                            Err(e) => {
                                error!(
                                    "Failed to process high-priority event from agent {}: {}",
                                    agent_id, e
                                );
                            }
                        }
                    });
                } else {
                    // Return event to queue if no workers available
                    let mut queue = self.high_priority_queue.write().await;
                    queue.push_front((agent_id, event_request, enqueue_time));
                    break;
                }
            }
        }
    }

    async fn normal_priority_worker(&self) {
        let mut interval = time::interval(Duration::from_millis(50)); // 20 Hz processing

        loop {
            interval.tick().await;

            // Process normal-priority events in batches
            let batch: Vec<_> = {
                let mut queue = self.normal_priority_queue.write().await;
                (0..self.config.batch_size.min(queue.len()))
                    .filter_map(|_| queue.pop_front())
                    .collect()
            };

            if !batch.is_empty() {
                if let Ok(_permit) = self.processing_semaphore.try_acquire() {
                    let processor = self.clone();
                    tokio::spawn(async move {
                        processor.process_event_batch(batch).await;
                    });
                } else {
                    // Return batch to queue if no workers available
                    let mut queue = self.normal_priority_queue.write().await;
                    for item in batch.into_iter().rev() {
                        queue.push_front(item);
                    }
                }
            }
        }
    }

    async fn process_single_event(
        &self,
        agent_id: Uuid,
        event_request: CreateSecurityEventRequest,
    ) -> Result<SecurityEvent> {
        // Process through realtime service (which includes threat detection)
        let event = self
            .realtime_service
            .process_security_event(agent_id, event_request)
            .await?;

        // Feed into correlation engine
        self.feed_correlation_engine(&event).await;

        // Increment processing stats
        {
            let mut stats = self.stats.write().await;
            stats.events_processed += 1;
        }

        Ok(event)
    }

    async fn process_event_batch(&self, batch: Vec<(Uuid, CreateSecurityEventRequest, Instant)>) {
        let batch_start = Instant::now();
        let batch_size = batch.len();

        for (agent_id, event_request, enqueue_time) in batch {
            match self.process_single_event(agent_id, event_request).await {
                Ok(_) => {
                    self.update_stats(batch_start.elapsed(), enqueue_time.elapsed())
                        .await;
                }
                Err(e) => {
                    error!(
                        "Failed to process batch event from agent {}: {}",
                        agent_id, e
                    );
                }
            }
        }

        debug!(
            "Processed batch of {} events in {:?}",
            batch_size,
            batch_start.elapsed()
        );
    }

    async fn update_stats(&self, processing_latency: Duration, queue_latency: Duration) {
        let mut stats = self.stats.write().await;
        stats.processing_latency_ms = processing_latency.as_millis() as f64;

        // Simple moving average for events per second
        let current_eps = 1000.0 / (processing_latency.as_millis().max(1) as f64);
        stats.events_per_second = (stats.events_per_second * 0.9) + (current_eps * 0.1);
    }

    // Correlation engine methods
    async fn start_correlation_engine(&self) {
        let processor_clone = self.clone();
        tokio::spawn(async move {
            processor_clone.correlation_worker().await;
        });
    }

    async fn correlation_worker(&self) {
        let mut interval = time::interval(Duration::from_secs(30)); // Check correlations every 30 seconds

        loop {
            interval.tick().await;
            self.process_correlations().await;
            self.cleanup_old_correlations().await;
        }
    }

    async fn feed_correlation_engine(&self, event: &SecurityEvent) {
        // This is a simplified correlation feed - in production this would be much more sophisticated
        let correlation_key = format!("{}_{}", event.event_type, event.severity.clone() as i32);

        let mut correlations = self.active_correlations.write().await;

        if let Some(correlation) = correlations.get_mut(&correlation_key) {
            // Update existing correlation
            correlation.agent_ids.push(event.agent_id);
            correlation.last_seen = event.occurred_at;
            correlation.event_count += 1;

            // Increase confidence based on multiple agents
            let unique_agents = correlation
                .agent_ids
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len();
            correlation.pattern_confidence = (unique_agents as f32 / 10.0).min(1.0);
        } else {
            // Create new correlation
            let correlation = EventCorrelation {
                correlation_id: Uuid::new_v4(),
                agent_ids: vec![event.agent_id],
                event_types: vec![event.event_type.clone()],
                severity_level: event.severity.clone(),
                first_seen: event.occurred_at,
                last_seen: event.occurred_at,
                event_count: 1,
                pattern_confidence: 0.1,
            };

            correlations.insert(correlation_key, correlation);
        }
    }

    async fn process_correlations(&self) {
        let correlations = self.active_correlations.read().await;

        for (key, correlation) in correlations.iter() {
            // Check if correlation meets alert criteria
            if correlation.pattern_confidence > 0.7 && correlation.event_count > 5 {
                // Generate correlated threat alert
                self.generate_correlation_alert(correlation).await;

                let mut stats = self.stats.write().await;
                stats.correlation_hits += 1;

                info!(
                    "Generated correlation alert for pattern: {} (confidence: {:.2})",
                    key, correlation.pattern_confidence
                );
            }
        }
    }

    async fn generate_correlation_alert(&self, correlation: &EventCorrelation) {
        // This would create a high-priority threat alert based on the correlation
        if let Err(e) = self
            .realtime_service
            .broadcast_emergency_alert(
                "Multi-Agent Threat Correlation",
                &format!(
                    "Detected coordinated threat activity across {} agents: {} events of type {:?}",
                    correlation.agent_ids.len(),
                    correlation.event_count,
                    correlation.event_types
                ),
                correlation.severity_level.clone(),
                correlation.agent_ids.clone(),
            )
            .await
        {
            error!("Failed to broadcast correlation alert: {}", e);
        }
    }

    async fn cleanup_old_correlations(&self) {
        let cutoff = chrono::Utc::now()
            - chrono::Duration::seconds(self.config.correlation_window_seconds as i64);
        let mut correlations = self.active_correlations.write().await;

        let initial_count = correlations.len();
        correlations.retain(|_, correlation| correlation.last_seen > cutoff);

        let removed = initial_count - correlations.len();
        if removed > 0 {
            debug!("Cleaned up {} old correlations", removed);
        }
    }

    // Load default correlation patterns
    async fn load_default_patterns(&self) {
        let patterns = vec![
            CorrelationPattern {
                pattern_id: "lateral_movement".to_string(),
                name: "Lateral Movement Detection".to_string(),
                description: "Detects lateral movement attempts across multiple agents".to_string(),
                event_sequence: vec![
                    "authentication".to_string(),
                    "process_creation".to_string(),
                    "network_connection".to_string(),
                ],
                max_time_window: Duration::from_secs(600), // 10 minutes
                min_agents: 2,
                confidence_threshold: 0.8,
                auto_response: Some("isolate_agents".to_string()),
            },
            CorrelationPattern {
                pattern_id: "mass_file_encryption".to_string(),
                name: "Ransomware Activity".to_string(),
                description: "Detects potential ransomware through mass file modifications"
                    .to_string(),
                event_sequence: vec!["file_access".to_string(), "file_modification".to_string()],
                max_time_window: Duration::from_secs(300), // 5 minutes
                min_agents: 1,
                confidence_threshold: 0.9,
                auto_response: Some("emergency_isolation".to_string()),
            },
            CorrelationPattern {
                pattern_id: "data_exfiltration".to_string(),
                name: "Data Exfiltration Detection".to_string(),
                description: "Detects potential data exfiltration through network patterns"
                    .to_string(),
                event_sequence: vec!["file_access".to_string(), "network_connection".to_string()],
                max_time_window: Duration::from_secs(180), // 3 minutes
                min_agents: 1,
                confidence_threshold: 0.75,
                auto_response: Some("block_network".to_string()),
            },
        ];

        let mut pattern_store = self.correlation_patterns.write().await;
        pattern_store.extend(patterns);
    }

    // Statistics and monitoring
    async fn start_stats_collector(&self) {
        let processor_clone = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10));

            loop {
                interval.tick().await;
                let stats = processor_clone.get_stats().await;

                info!("Event Processor Stats: {} events/sec, {} queue depth, {:.2}ms latency, {} correlations",
                      stats.events_per_second as u32,
                      stats.queue_depth,
                      stats.processing_latency_ms,
                      stats.correlation_hits
                );
            }
        });
    }

    async fn start_queue_monitor(&self) {
        let processor_clone = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;
                let queue_size = processor_clone.get_total_queue_size().await;

                if queue_size > processor_clone.config.max_queue_size / 2 {
                    warn!(
                        "Event processing queue is getting full: {}/{}",
                        queue_size, processor_clone.config.max_queue_size
                    );
                }
            }
        });
    }
}
