use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use axum::extract::ws::WebSocket;
use secureguard_shared::{AgentMessage, DashboardMessage};
use tracing::{info, warn, error};

#[derive(Debug, Clone)]
pub enum ConnectionType {
    Agent { agent_id: Uuid },
    Dashboard { user_id: Uuid },
}

#[derive(Debug)]
pub struct Connection {
    pub connection_type: ConnectionType,
    pub sender: mpsc::UnboundedSender<String>,
}

#[derive(Clone)]
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<Uuid, Connection>>>,
    agent_connections: Arc<RwLock<HashMap<Uuid, Uuid>>>, // agent_id -> connection_id
    dashboard_connections: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>, // user_id -> connection_ids
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            agent_connections: Arc::new(RwLock::new(HashMap::new())),
            dashboard_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_agent_connection(&self, agent_id: Uuid, sender: mpsc::UnboundedSender<String>) -> Uuid {
        let connection_id = Uuid::new_v4();
        let connection = Connection {
            connection_type: ConnectionType::Agent { agent_id },
            sender,
        };

        let mut connections = self.connections.write().await;
        let mut agent_connections = self.agent_connections.write().await;
        
        connections.insert(connection_id, connection);
        agent_connections.insert(agent_id, connection_id);

        info!("Agent {} connected with connection {}", agent_id, connection_id);
        connection_id
    }

    pub async fn add_dashboard_connection(&self, user_id: Uuid, sender: mpsc::UnboundedSender<String>) -> Uuid {
        let connection_id = Uuid::new_v4();
        let connection = Connection {
            connection_type: ConnectionType::Dashboard { user_id },
            sender,
        };

        let mut connections = self.connections.write().await;
        let mut dashboard_connections = self.dashboard_connections.write().await;
        
        connections.insert(connection_id, connection);
        dashboard_connections
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(connection_id);

        info!("Dashboard user {} connected with connection {}", user_id, connection_id);
        connection_id
    }

    pub async fn remove_connection(&self, connection_id: Uuid) {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.remove(&connection_id) {
            match connection.connection_type {
                ConnectionType::Agent { agent_id } => {
                    let mut agent_connections = self.agent_connections.write().await;
                    agent_connections.remove(&agent_id);
                    info!("Agent {} disconnected", agent_id);
                }
                ConnectionType::Dashboard { user_id } => {
                    let mut dashboard_connections = self.dashboard_connections.write().await;
                    if let Some(user_connections) = dashboard_connections.get_mut(&user_id) {
                        user_connections.retain(|&id| id != connection_id);
                        if user_connections.is_empty() {
                            dashboard_connections.remove(&user_id);
                        }
                    }
                    info!("Dashboard user {} disconnected", user_id);
                }
            }
        }
    }

    pub async fn send_to_agent(&self, agent_id: Uuid, message: &AgentMessage) -> Result<(), String> {
        let agent_connections = self.agent_connections.read().await;
        let connections = self.connections.read().await;
        
        if let Some(connection_id) = agent_connections.get(&agent_id) {
            if let Some(connection) = connections.get(connection_id) {
                let message_json = serde_json::to_string(message)
                    .map_err(|e| format!("Failed to serialize message: {}", e))?;
                
                connection.sender.send(message_json)
                    .map_err(|e| format!("Failed to send message to agent {}: {}", agent_id, e))?;
                
                return Ok(());
            }
        }
        
        Err(format!("Agent {} not connected", agent_id))
    }

    pub async fn send_to_all_dashboards(&self, message: &DashboardMessage) -> Result<(), String> {
        let dashboard_connections = self.dashboard_connections.read().await;
        let connections = self.connections.read().await;
        
        let message_json = serde_json::to_string(message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;
        
        let mut sent_count = 0;
        let mut error_count = 0;
        
        for connection_ids in dashboard_connections.values() {
            for connection_id in connection_ids {
                if let Some(connection) = connections.get(connection_id) {
                    match connection.sender.send(message_json.clone()) {
                        Ok(_) => sent_count += 1,
                        Err(e) => {
                            error_count += 1;
                            warn!("Failed to send message to dashboard connection {}: {}", connection_id, e);
                        }
                    }
                }
            }
        }
        
        info!("Sent dashboard message to {} connections ({} errors)", sent_count, error_count);
        Ok(())
    }

    pub async fn send_to_user_dashboards(&self, user_id: Uuid, message: &DashboardMessage) -> Result<(), String> {
        let dashboard_connections = self.dashboard_connections.read().await;
        let connections = self.connections.read().await;
        
        if let Some(connection_ids) = dashboard_connections.get(&user_id) {
            let message_json = serde_json::to_string(message)
                .map_err(|e| format!("Failed to serialize message: {}", e))?;
            
            for connection_id in connection_ids {
                if let Some(connection) = connections.get(connection_id) {
                    if let Err(e) = connection.sender.send(message_json.clone()) {
                        warn!("Failed to send message to user {} dashboard {}: {}", user_id, connection_id, e);
                    }
                }
            }
        }
        
        Ok(())
    }

    pub async fn get_connected_agents(&self) -> Vec<Uuid> {
        let agent_connections = self.agent_connections.read().await;
        agent_connections.keys().cloned().collect()
    }

    pub async fn get_connection_count(&self) -> (usize, usize) {
        let agent_connections = self.agent_connections.read().await;
        let dashboard_connections = self.dashboard_connections.read().await;
        
        let agent_count = agent_connections.len();
        let dashboard_count: usize = dashboard_connections.values().map(|v| v.len()).sum();
        
        (agent_count, dashboard_count)
    }

    pub async fn is_agent_connected(&self, agent_id: Uuid) -> bool {
        let agent_connections = self.agent_connections.read().await;
        agent_connections.contains_key(&agent_id)
    }
}