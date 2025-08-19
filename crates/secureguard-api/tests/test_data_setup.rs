use secureguard_api::database::Database;
use uuid::Uuid;

/// Test data setup for comprehensive workflow testing
pub struct TestDataSetup {
    pub database: Database,
}

impl TestDataSetup {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL_TEST").unwrap_or_else(|_| {
            "postgresql://secureguard:password@localhost:5432/secureguard_dev".to_string()
        });
        
        let database = Database::new(&database_url)
            .await
            .expect("Failed to connect to test database");
            
        TestDataSetup { database }
    }
    
    /// Set up all roles in the database for testing
    pub async fn setup_roles(&self) -> Result<(), sqlx::Error> {
        let roles = vec![
            ("system_admin", "System Administrator", 100, "Full system access"),
            ("security_analyst", "Security Analyst", 80, "Security monitoring and incident response"),
            ("admin", "Administrator", 70, "User and system management"),
            ("manager", "Manager", 50, "Management oversight"),
            ("power_user", "Power User", 30, "Advanced user capabilities"),
            ("user", "User", 10, "Standard user access"),
            ("read_only", "Read Only", 5, "View-only access"),
            ("guest", "Guest", 1, "Minimal guest access"),
        ];
        
        for (slug, name, level, desc) in roles {
            sqlx::query!(
                r#"
                INSERT INTO rbac.roles (role_id, role_name, role_slug, hierarchy_level, description, is_active, is_system_role)
                VALUES (gen_random_uuid(), $1, $2, $3, $4, TRUE, TRUE)
                ON CONFLICT (role_slug) DO UPDATE SET
                    role_name = EXCLUDED.role_name,
                    hierarchy_level = EXCLUDED.hierarchy_level,
                    description = EXCLUDED.description
                "#,
                name, slug, level, desc
            )
            .execute(self.database.pool())
            .await?;
        }
        
        Ok(())
    }
    
    /// Set up permissions in the database
    pub async fn setup_permissions(&self) -> Result<(), sqlx::Error> {
        let permissions = vec![
            // System permissions
            ("system.admin", "system", "Full system administration", 10),
            ("system.config", "system", "System configuration", 8),
            ("system.maintenance", "system", "System maintenance", 8),
            
            // User management
            ("users.create", "user", "Create users", 7),
            ("users.read", "user", "View users", 3),
            ("users.update", "user", "Update users", 6),
            ("users.delete", "user", "Delete users", 9),
            ("users.roles", "user", "Manage user roles", 8),
            
            // Secrets and security
            ("secrets.read", "security", "Read secrets", 8),
            ("secrets.create", "security", "Create secrets", 9),
            ("secrets.update", "security", "Update secrets", 9),
            ("secrets.delete", "security", "Delete secrets", 10),
            
            // Agent management
            ("agents.read", "agent", "View agents", 3),
            ("agents.create", "agent", "Create agents", 5),
            ("agents.update", "agent", "Update agents", 5),
            ("agents.delete", "agent", "Delete agents", 7),
            ("agents.control", "agent", "Control agents", 8),
            
            // Security monitoring
            ("security.incidents", "security", "View security incidents", 6),
            ("security.monitoring", "security", "Security monitoring", 6),
            ("security.response", "security", "Security incident response", 7),
            
            // Subscription management
            ("subscriptions.read", "subscription", "View subscriptions", 4),
            ("subscriptions.create", "subscription", "Create subscriptions", 6),
            ("subscriptions.update", "subscription", "Update subscriptions", 6),
            ("subscriptions.delete", "subscription", "Delete subscriptions", 8),
            ("subscriptions.migrate", "subscription", "Migrate subscriptions", 9),
            
            // Audit and compliance
            ("audit.read", "audit", "Read audit logs", 5),
            ("audit.export", "audit", "Export audit data", 7),
            
            // API access
            ("api.read", "api", "Read API access", 2),
            ("api.write", "api", "Write API access", 5),
            ("api.admin", "api", "API administration", 8),
        ];
        
        for (slug, category, desc, sensitivity) in permissions {
            sqlx::query!(
                r#"
                INSERT INTO rbac.permissions (permission_id, permission_name, permission_slug, category, description, sensitivity_level, is_active)
                VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, TRUE)
                ON CONFLICT (permission_slug) DO UPDATE SET
                    permission_name = EXCLUDED.permission_name,
                    category = EXCLUDED.category,
                    description = EXCLUDED.description,
                    sensitivity_level = EXCLUDED.sensitivity_level
                "#,
                slug, slug, category, desc, sensitivity
            )
            .execute(self.database.pool())
            .await?;
        }
        
        Ok(())
    }
    
    /// Set up role-permission mappings
    pub async fn setup_role_permissions(&self) -> Result<(), sqlx::Error> {
        let mappings = vec![
            // System Admin - all permissions
            ("system_admin", vec![
                "system.admin", "system.config", "system.maintenance",
                "users.create", "users.read", "users.update", "users.delete", "users.roles",
                "secrets.read", "secrets.create", "secrets.update", "secrets.delete",
                "agents.read", "agents.create", "agents.update", "agents.delete", "agents.control",
                "security.incidents", "security.monitoring", "security.response",
                "subscriptions.read", "subscriptions.create", "subscriptions.update", "subscriptions.delete", "subscriptions.migrate",
                "audit.read", "audit.export",
                "api.read", "api.write", "api.admin"
            ]),
            
            // Security Analyst
            ("security_analyst", vec![
                "agents.read", "agents.update",
                "security.incidents", "security.monitoring", "security.response",
                "secrets.read",
                "audit.read",
                "api.read", "api.write"
            ]),
            
            // Admin
            ("admin", vec![
                "users.create", "users.read", "users.update", "users.delete", "users.roles",
                "agents.read", "agents.create", "agents.update", "agents.delete",
                "subscriptions.read", "subscriptions.create", "subscriptions.update",
                "audit.read",
                "api.read", "api.write"
            ]),
            
            // Manager  
            ("manager", vec![
                "users.read",
                "agents.read",
                "security.incidents", "security.monitoring",
                "subscriptions.read",
                "audit.read",
                "api.read"
            ]),
            
            // Power User
            ("power_user", vec![
                "agents.read", "agents.update",
                "security.monitoring",
                "api.read", "api.write"
            ]),
            
            // User
            ("user", vec![
                "agents.read",
                "api.read"
            ]),
            
            // Read Only
            ("read_only", vec![
                "api.read"
            ]),
            
            // Guest - minimal access
            ("guest", vec![]),
        ];
        
        for (role_slug, permissions) in mappings {
            for permission_slug in permissions {
                sqlx::query!(
                    r#"
                    INSERT INTO rbac.role_permissions (role_permission_id, role_id, permission_id, is_active)
                    SELECT gen_random_uuid(), r.role_id, p.permission_id, TRUE
                    FROM rbac.roles r, rbac.permissions p
                    WHERE r.role_slug = $1 AND p.permission_slug = $2
                    ON CONFLICT DO NOTHING
                    "#,
                    role_slug, permission_slug
                )
                .execute(self.database.pool())
                .await?;
            }
        }
        
        Ok(())
    }
    
    /// Create test tenant for multi-tenant testing
    pub async fn create_test_tenant(&self, name: &str) -> Result<Uuid, sqlx::Error> {
        let tenant_id = Uuid::new_v4();
        
        sqlx::query!(
            r#"
            INSERT INTO tenants (tenant_id, name, plan_tier, created_at)
            VALUES ($1, $2, 'test', now())
            "#,
            tenant_id, name
        )
        .execute(self.database.pool())
        .await?;
        
        Ok(tenant_id)
    }
    
    /// Create test agent for workflow testing
    pub async fn create_test_agent(&self, tenant_id: Uuid, user_id: Option<Uuid>, name: &str) -> Result<Uuid, sqlx::Error> {
        let agent_id = Uuid::new_v4();
        let hardware_fingerprint = format!("test-hw-{}", name);
        let os_info = serde_json::json!({
            "name": "Windows",
            "version": "10",
            "arch": "x64"
        });
        
        sqlx::query!(
            r#"
            INSERT INTO agents (agent_id, tenant_id, user_id, hardware_fingerprint, device_name, os_info, status, version, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, 'offline', '1.0.0', now())
            "#,
            agent_id, tenant_id, user_id, hardware_fingerprint, name, os_info, 
        )
        .execute(self.database.pool())
        .await?;
        
        Ok(agent_id)
    }
    
    /// Clean up test data
    pub async fn cleanup_test_data(&self) -> Result<(), sqlx::Error> {
        // Clean up in reverse dependency order
        sqlx::query!("DELETE FROM rbac.user_roles WHERE user_id IN (SELECT user_id FROM users WHERE email LIKE '%@test.com')")
            .execute(self.database.pool()).await?;
            
        sqlx::query!("DELETE FROM agents WHERE device_name LIKE 'test_%'")
            .execute(self.database.pool()).await?;
            
        sqlx::query!("DELETE FROM users WHERE email LIKE '%@test.com'")
            .execute(self.database.pool()).await?;
            
        sqlx::query!("DELETE FROM tenants WHERE name LIKE 'test_%'")
            .execute(self.database.pool()).await?;
        
        Ok(())
    }
    
    /// Setup complete test environment
    pub async fn setup_complete_test_environment(&self) -> Result<(), sqlx::Error> {
        self.setup_roles().await?;
        self.setup_permissions().await?;
        self.setup_role_permissions().await?;
        
        // Create test tenant
        let _tenant_id = self.create_test_tenant("test_company").await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_setup_roles_and_permissions() {
        let setup = TestDataSetup::new().await;
        
        // Test roles setup
        setup.setup_roles().await.expect("Failed to setup roles");
        
        // Verify roles exist
        let role_count = sqlx::query!("SELECT COUNT(*) as count FROM rbac.roles WHERE is_active = TRUE")
            .fetch_one(setup.database.pool())
            .await
            .expect("Failed to count roles");
            
        assert!(role_count.count.unwrap_or(0) >= 8);
        
        // Test permissions setup
        setup.setup_permissions().await.expect("Failed to setup permissions");
        
        // Verify permissions exist
        let perm_count = sqlx::query!("SELECT COUNT(*) as count FROM rbac.permissions WHERE is_active = TRUE")
            .fetch_one(setup.database.pool())
            .await
            .expect("Failed to count permissions");
            
        assert!(perm_count.count.unwrap_or(0) >= 20);
        
        // Test role-permission mappings
        setup.setup_role_permissions().await.expect("Failed to setup role-permission mappings");
        
        // Verify system admin has all permissions
        let sys_admin_perms = sqlx::query!(
            r#"
            SELECT COUNT(*) as count 
            FROM rbac.role_permissions rp
            JOIN rbac.roles r ON rp.role_id = r.role_id
            WHERE r.role_slug = 'system_admin' AND rp.is_active = TRUE
            "#
        )
        .fetch_one(setup.database.pool())
        .await
        .expect("Failed to count system admin permissions");
        
        assert!(sys_admin_perms.count.unwrap_or(0) > 15);
    }
    
    #[tokio::test] 
    async fn test_create_test_resources() {
        let setup = TestDataSetup::new().await;
        
        // Create tenant
        let tenant_id = setup.create_test_tenant("test_workflow_tenant").await
            .expect("Failed to create test tenant");
        
        // Create agent  
        let agent_id = setup.create_test_agent(tenant_id, None, "test_workflow_agent").await
            .expect("Failed to create test agent");
        
        // Verify resources exist
        let tenant_exists = sqlx::query!("SELECT COUNT(*) as count FROM tenants WHERE tenant_id = $1", tenant_id)
            .fetch_one(setup.database.pool())
            .await
            .expect("Failed to check tenant");
            
        assert_eq!(tenant_exists.count.unwrap_or(0), 1);
        
        let agent_exists = sqlx::query!("SELECT COUNT(*) as count FROM agents WHERE agent_id = $1", agent_id)
            .fetch_one(setup.database.pool())
            .await
            .expect("Failed to check agent");
            
        assert_eq!(agent_exists.count.unwrap_or(0), 1);
        
        // Cleanup
        setup.cleanup_test_data().await.expect("Failed to cleanup test data");
    }
}