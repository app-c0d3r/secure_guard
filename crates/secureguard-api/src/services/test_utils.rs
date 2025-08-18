use sqlx::{PgPool, Row};
use crate::database::Database;
use std::env;

pub struct TestDatabase {
    pub database: Database,
    pub pool: PgPool,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let database_url = env::var("DATABASE_URL_TEST")
            .unwrap_or_else(|_| "postgresql://secureguard:password@localhost:5432/secureguard_test".to_string());
        
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        // Clean up any existing data
        Self::cleanup_database(&pool).await;

        // Run migrations
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        let database = Database::new(&database_url)
            .await
            .expect("Failed to create test database");

        Self { database, pool }
    }

    pub async fn cleanup_database(pool: &PgPool) {
        // Clean tables in reverse dependency order
        let _ = sqlx::query("TRUNCATE agents.endpoints RESTART IDENTITY CASCADE")
            .execute(pool)
            .await;
        
        let _ = sqlx::query("TRUNCATE users.users RESTART IDENTITY CASCADE")
            .execute(pool)
            .await;
        
        let _ = sqlx::query("TRUNCATE tenants.tenants RESTART IDENTITY CASCADE")
            .execute(pool)
            .await;
    }

    pub async fn count_users(&self) -> i64 {
        sqlx::query("SELECT COUNT(*) FROM users.users")
            .fetch_one(&self.pool)
            .await
            .unwrap()
            .get(0)
    }

    pub async fn count_agents(&self) -> i64 {
        sqlx::query("SELECT COUNT(*) FROM agents.endpoints")
            .fetch_one(&self.pool)
            .await
            .unwrap()
            .get(0)
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Cleanup is handled by TRUNCATE in new()
    }
}