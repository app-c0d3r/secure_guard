use anyhow::Result;
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod agent;
mod communication;
mod security;
mod utils;

use agent::AgentCore;
use utils::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "secureguard_agent=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting SecureGuard Agent v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = Config::load().await?;
    info!("Configuration loaded successfully");

    // Initialize agent core
    let mut agent = AgentCore::new(config).await?;
    info!("Agent core initialized");

    // Start agent services
    agent.start().await?;
    info!("Agent services started successfully");

    // Wait for shutdown signal
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received CTRL+C, shutting down gracefully...");
        }
        result = agent.run() => {
            match result {
                Ok(_) => info!("Agent completed successfully"),
                Err(e) => error!("Agent error: {}", e),
            }
        }
    }

    // Graceful shutdown
    agent.shutdown().await?;
    info!("SecureGuard Agent shutdown complete");

    Ok(())
}
