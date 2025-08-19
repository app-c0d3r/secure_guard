use secureguard_api::{config::Config, create_app, database::Database, telemetry};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    // Initialize OpenTelemetry
    telemetry::init_telemetry()?;
    
    // Initialize metrics
    telemetry::metrics::init();

    let config = Config::from_env()?;
    let database = Database::new(&config.database_url).await?;

    let app = create_app(database).await;

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("SecureGuard API server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    // Run server with graceful shutdown
    let result = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await;
    
    // Shutdown telemetry on exit
    telemetry::shutdown_telemetry();
    
    result?;
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    tracing::info!("Received shutdown signal");
}
