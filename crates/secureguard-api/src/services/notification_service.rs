use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use secureguard_shared::{SecureGuardError, Result};
use tracing::{info, warn, error};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDelivery {
    pub notification_id: Uuid,
    pub delivery_method: String,
    pub recipient: String,
    pub status: String,
    pub attempt_count: u32,
    pub delivered_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityNotificationRequest {
    pub user_id: Uuid,
    pub incident_id: Option<Uuid>,
    pub notification_type: String,
    pub subject: String,
    pub message: String,
    pub priority: String,
    pub delivery_methods: Vec<String>, // email, sms, push, webhook
}

pub struct NotificationService {
    pool: PgPool,
    email_client: EmailClient,
    sms_client: SmsClient,
    push_client: PushClient,
    webhook_client: WebhookClient,
}

impl NotificationService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            email_client: EmailClient::new(),
            sms_client: SmsClient::new(),
            push_client: PushClient::new(),
            webhook_client: WebhookClient::new(),
        }
    }

    /// Send security notification to user
    pub async fn send_security_notification(&self, request: SecurityNotificationRequest) -> Result<Vec<Uuid>> {
        info!("📨 Sending security notification to user {}", request.user_id);
        
        // Get user notification preferences
        let user_prefs = self.get_user_notification_preferences(request.user_id).await?;
        
        let mut notification_ids = Vec::new();
        
        // Send via each requested delivery method
        for method in &request.delivery_methods {
            if user_prefs.is_method_enabled(method) {
                match self.send_via_method(&request, method, &user_prefs).await {
                    Ok(notification_id) => {
                        notification_ids.push(notification_id);
                        info!("✅ Notification sent via {}: {}", method, notification_id);
                    }
                    Err(e) => {
                        error!("❌ Failed to send notification via {}: {}", method, e);
                        // Continue with other methods even if one fails
                    }
                }
            } else {
                info!("⚠️ Notification method {} disabled for user {}", method, request.user_id);
            }
        }
        
        Ok(notification_ids)
    }

    /// Send notification via specific delivery method
    async fn send_via_method(
        &self,
        request: &SecurityNotificationRequest,
        method: &str,
        user_prefs: &UserNotificationPreferences,
    ) -> Result<Uuid> {
        // Create notification record
        let notification_id = sqlx::query!(
            r#"
            INSERT INTO security_notifications (
                incident_id, user_id, notification_type, recipient,
                subject, message, priority, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending')
            RETURNING notification_id
            "#,
            request.incident_id,
            request.user_id,
            method,
            self.get_recipient_address(method, user_prefs),
            request.subject,
            request.message,
            request.priority
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .notification_id;

        // Send notification based on method
        match method {
            "email" => {
                self.send_email_notification(notification_id, request, user_prefs).await?;
            }
            "sms" => {
                self.send_sms_notification(notification_id, request, user_prefs).await?;
            }
            "push" => {
                self.send_push_notification(notification_id, request, user_prefs).await?;
            }
            "webhook" => {
                self.send_webhook_notification(notification_id, request, user_prefs).await?;
            }
            _ => {
                return Err(SecureGuardError::ValidationError(
                    format!("Unsupported notification method: {}", method)
                ));
            }
        }

        Ok(notification_id)
    }

    /// Send email notification
    async fn send_email_notification(
        &self,
        notification_id: Uuid,
        request: &SecurityNotificationRequest,
        user_prefs: &UserNotificationPreferences,
    ) -> Result<()> {
        if let Some(email) = &user_prefs.email {
            let email_content = self.build_email_content(request).await?;
            
            match self.email_client.send_email(
                email,
                &request.subject,
                &email_content.html_body,
                Some(&email_content.text_body),
            ).await {
                Ok(_) => {
                    self.mark_notification_sent(notification_id).await?;
                    info!("📧 Email sent successfully to {}", email);
                }
                Err(e) => {
                    self.mark_notification_failed(notification_id, &e.to_string()).await?;
                    error!("❌ Email sending failed: {}", e);
                    return Err(e);
                }
            }
        } else {
            return Err(SecureGuardError::ValidationError(
                "User has no email address configured".to_string()
            ));
        }

        Ok(())
    }

    /// Send SMS notification
    async fn send_sms_notification(
        &self,
        notification_id: Uuid,
        request: &SecurityNotificationRequest,
        user_prefs: &UserNotificationPreferences,
    ) -> Result<()> {
        if let Some(phone) = &user_prefs.phone {
            let sms_content = self.build_sms_content(request).await?;
            
            match self.sms_client.send_sms(phone, &sms_content).await {
                Ok(_) => {
                    self.mark_notification_sent(notification_id).await?;
                    info!("📱 SMS sent successfully to {}", phone);
                }
                Err(e) => {
                    self.mark_notification_failed(notification_id, &e.to_string()).await?;
                    error!("❌ SMS sending failed: {}", e);
                    return Err(e);
                }
            }
        } else {
            return Err(SecureGuardError::ValidationError(
                "User has no phone number configured".to_string()
            ));
        }

        Ok(())
    }

    /// Send push notification
    async fn send_push_notification(
        &self,
        notification_id: Uuid,
        request: &SecurityNotificationRequest,
        user_prefs: &UserNotificationPreferences,
    ) -> Result<()> {
        if let Some(push_token) = &user_prefs.push_token {
            let push_content = self.build_push_content(request).await?;
            
            match self.push_client.send_push(push_token, &push_content).await {
                Ok(_) => {
                    self.mark_notification_sent(notification_id).await?;
                    info!("📲 Push notification sent successfully");
                }
                Err(e) => {
                    self.mark_notification_failed(notification_id, &e.to_string()).await?;
                    error!("❌ Push notification failed: {}", e);
                    return Err(e);
                }
            }
        } else {
            return Err(SecureGuardError::ValidationError(
                "User has no push token configured".to_string()
            ));
        }

        Ok(())
    }

    /// Send webhook notification
    async fn send_webhook_notification(
        &self,
        notification_id: Uuid,
        request: &SecurityNotificationRequest,
        user_prefs: &UserNotificationPreferences,
    ) -> Result<()> {
        if let Some(webhook_url) = &user_prefs.webhook_url {
            let webhook_payload = self.build_webhook_payload(request).await?;
            
            match self.webhook_client.send_webhook(webhook_url, &webhook_payload).await {
                Ok(_) => {
                    self.mark_notification_sent(notification_id).await?;
                    info!("🔗 Webhook notification sent successfully");
                }
                Err(e) => {
                    self.mark_notification_failed(notification_id, &e.to_string()).await?;
                    error!("❌ Webhook notification failed: {}", e);
                    return Err(e);
                }
            }
        } else {
            return Err(SecureGuardError::ValidationError(
                "User has no webhook URL configured".to_string()
            ));
        }

        Ok(())
    }

    /// Process pending notifications (for retry logic)
    pub async fn process_pending_notifications(&self) -> Result<u32> {
        let pending_notifications = sqlx::query!(
            r#"
            SELECT notification_id, user_id, notification_type, recipient,
                   subject, message, priority, retry_count, max_retries
            FROM security_notifications 
            WHERE status = 'pending' 
            AND (next_retry_at IS NULL OR next_retry_at <= now())
            AND retry_count < max_retries
            ORDER BY priority DESC, created_at ASC
            LIMIT 100
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        let mut processed = 0;

        for notification in pending_notifications {
            let user_prefs = self.get_user_notification_preferences(notification.user_id).await?;
            
            let request = SecurityNotificationRequest {
                user_id: notification.user_id,
                incident_id: None,
                notification_type: notification.notification_type.clone(),
                subject: notification.subject,
                message: notification.message,
                priority: notification.priority,
                delivery_methods: vec![notification.notification_type.clone()],
            };

            match self.send_via_method(&request, &notification.notification_type, &user_prefs).await {
                Ok(_) => {
                    processed += 1;
                    info!("✅ Retry successful for notification {}", notification.notification_id);
                }
                Err(e) => {
                    // Increment retry count and schedule next retry
                    self.schedule_notification_retry(notification.notification_id).await?;
                    warn!("⚠️ Retry failed for notification {}: {}", notification.notification_id, e);
                }
            }
        }

        info!("📊 Processed {} pending notifications", processed);
        Ok(processed)
    }

    // Helper methods
    async fn get_user_notification_preferences(&self, user_id: Uuid) -> Result<UserNotificationPreferences> {
        let user_info = sqlx::query!(
            r#"
            SELECT email, phone, push_notifications_enabled, 
                   email_notifications_enabled, sms_notifications_enabled,
                   webhook_url
            FROM users.users 
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?
        .ok_or_else(|| SecureGuardError::UserNotFound)?;

        Ok(UserNotificationPreferences {
            email: user_info.email,
            phone: user_info.phone,
            push_token: None, // Would be retrieved from device registration
            webhook_url: user_info.webhook_url,
            email_enabled: user_info.email_notifications_enabled.unwrap_or(true),
            sms_enabled: user_info.sms_notifications_enabled.unwrap_or(false),
            push_enabled: user_info.push_notifications_enabled.unwrap_or(true),
            webhook_enabled: user_info.webhook_url.is_some(),
        })
    }

    fn get_recipient_address(&self, method: &str, user_prefs: &UserNotificationPreferences) -> String {
        match method {
            "email" => user_prefs.email.clone().unwrap_or_default(),
            "sms" => user_prefs.phone.clone().unwrap_or_default(),
            "push" => user_prefs.push_token.clone().unwrap_or_default(),
            "webhook" => user_prefs.webhook_url.clone().unwrap_or_default(),
            _ => String::new(),
        }
    }

    async fn build_email_content(&self, request: &SecurityNotificationRequest) -> Result<EmailContent> {
        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <title>{}</title>
            </head>
            <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
                <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                    <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 8px 8px 0 0;">
                        <h1 style="margin: 0; font-size: 24px;">🛡️ SecureGuard Security Alert</h1>
                    </div>
                    <div style="background: #f8f9fa; padding: 20px; border: 1px solid #ddd; border-top: none; border-radius: 0 0 8px 8px;">
                        <h2 style="color: #dc3545; margin-top: 0;">{}</h2>
                        <div style="background: white; padding: 15px; border-radius: 4px; border-left: 4px solid #dc3545;">
                            {}
                        </div>
                        <div style="margin-top: 20px; padding: 15px; background: #e9ecef; border-radius: 4px;">
                            <p><strong>Priority:</strong> <span style="color: {};">{}</span></p>
                            <p><strong>Time:</strong> {}</p>
                        </div>
                        <div style="margin-top: 20px; text-align: center;">
                            <a href="https://dashboard.secureguard.com/incidents" 
                               style="background: #667eea; color: white; padding: 12px 24px; text-decoration: none; border-radius: 4px; display: inline-block;">
                                View in Dashboard
                            </a>
                        </div>
                        <hr style="margin: 20px 0; border: none; border-top: 1px solid #ddd;">
                        <p style="font-size: 12px; color: #666; text-align: center;">
                            This is an automated security alert from SecureGuard. If you have questions, contact support.
                        </p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            request.subject,
            request.subject,
            request.message.replace('\n', "<br>"),
            match request.priority.as_str() {
                "high" | "urgent" => "#dc3545",
                "medium" => "#fd7e14", 
                _ => "#28a745"
            },
            request.priority.to_uppercase(),
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        let text_body = format!(
            "SecureGuard Security Alert\n\n{}\n\n{}\n\nPriority: {}\nTime: {}\n\nView in Dashboard: https://dashboard.secureguard.com/incidents",
            request.subject,
            request.message,
            request.priority.to_uppercase(),
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        Ok(EmailContent { html_body, text_body })
    }

    async fn build_sms_content(&self, request: &SecurityNotificationRequest) -> Result<String> {
        Ok(format!(
            "🛡️ SecureGuard Alert: {} - {} View details: https://sg.app/i/{}",
            request.subject,
            request.message.chars().take(100).collect::<String>(),
            request.incident_id.unwrap_or_else(|| Uuid::new_v4())
        ))
    }

    async fn build_push_content(&self, request: &SecurityNotificationRequest) -> Result<PushContent> {
        Ok(PushContent {
            title: format!("🛡️ {}", request.subject),
            body: request.message.clone(),
            data: HashMap::from([
                ("incident_id".to_string(), request.incident_id.unwrap_or_else(|| Uuid::new_v4()).to_string()),
                ("priority".to_string(), request.priority.clone()),
                ("type".to_string(), request.notification_type.clone()),
            ]),
        })
    }

    async fn build_webhook_payload(&self, request: &SecurityNotificationRequest) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "event": "security_alert",
            "timestamp": Utc::now(),
            "incident_id": request.incident_id,
            "user_id": request.user_id,
            "subject": request.subject,
            "message": request.message,
            "priority": request.priority,
            "notification_type": request.notification_type
        }))
    }

    async fn mark_notification_sent(&self, notification_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE security_notifications SET status = 'sent', sent_at = now() WHERE notification_id = $1",
            notification_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn mark_notification_failed(&self, notification_id: Uuid, error_message: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE security_notifications 
            SET status = 'failed', 
                retry_count = retry_count + 1,
                next_retry_at = now() + interval '1 hour'
            WHERE notification_id = $1
            "#,
            notification_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn schedule_notification_retry(&self, notification_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE security_notifications 
            SET retry_count = retry_count + 1,
                next_retry_at = now() + interval '1 hour' * retry_count
            WHERE notification_id = $1
            "#,
            notification_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SecureGuardError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

// Supporting structures
#[derive(Debug, Clone)]
pub struct UserNotificationPreferences {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub push_token: Option<String>,
    pub webhook_url: Option<String>,
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub push_enabled: bool,
    pub webhook_enabled: bool,
}

impl UserNotificationPreferences {
    pub fn is_method_enabled(&self, method: &str) -> bool {
        match method {
            "email" => self.email_enabled && self.email.is_some(),
            "sms" => self.sms_enabled && self.phone.is_some(),
            "push" => self.push_enabled && self.push_token.is_some(),
            "webhook" => self.webhook_enabled && self.webhook_url.is_some(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmailContent {
    pub html_body: String,
    pub text_body: String,
}

#[derive(Debug, Clone)]
pub struct PushContent {
    pub title: String,
    pub body: String,
    pub data: HashMap<String, String>,
}

// Mock notification clients (would be replaced with real implementations)
#[derive(Debug, Clone)]
pub struct EmailClient;

impl EmailClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn send_email(&self, to: &str, subject: &str, html_body: &str, text_body: Option<&str>) -> Result<()> {
        info!("📧 Sending email to {} with subject: {}", to, subject);
        // Implementation would use AWS SES, SendGrid, etc.
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SmsClient;

impl SmsClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn send_sms(&self, to: &str, message: &str) -> Result<()> {
        info!("📱 Sending SMS to {}: {}", to, message);
        // Implementation would use Twilio, AWS SNS, etc.
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PushClient;

impl PushClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn send_push(&self, token: &str, content: &PushContent) -> Result<()> {
        info!("📲 Sending push notification: {}", content.title);
        // Implementation would use Firebase Cloud Messaging, Apple Push Notifications, etc.
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WebhookClient;

impl WebhookClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn send_webhook(&self, url: &str, payload: &serde_json::Value) -> Result<()> {
        info!("🔗 Sending webhook to {}", url);
        // Implementation would make HTTP POST request
        Ok(())
    }
}