use crate::communication::messages::{AgentMessage, ServerMessage};
use crate::utils::config::Config;
use anyhow::Result;

/// Communication client for connecting to SecureGuard backend
pub struct Client {
    _config: Config,
}

impl Client {
    pub async fn new(config: &Config) -> Result<Self> {
        tracing::info!("Client initialized (mock)");
        Ok(Self {
            _config: config.clone(),
        })
    }

    pub async fn send_message(&mut self, _message: &AgentMessage) -> Result<()> {
        // TODO: Implement actual message sending
        Ok(())
    }

    pub async fn receive_message(&mut self) -> Result<ServerMessage> {
        // TODO: Implement actual message receiving
        // For now, this will never return (blocking)
        tokio::time::sleep(std::time::Duration::from_secs(u64::MAX)).await;
        unreachable!()
    }

    pub async fn close(&mut self) -> Result<()> {
        tracing::info!("Client connection closed");
        Ok(())
    }
}
