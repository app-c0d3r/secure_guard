use secureguard_api::{create_app, config::Config, database::Database};
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("secureguard_api=debug,tower_http=debug")
        .init();

    let config = Config::from_env()?;
    let database = Database::new(&config.database_url).await?;
    
    let app = create_app(database).await;
    
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("SecureGuard API server starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}