# 🔧 **SecureGuard Subscription Administration Guide**

## 🎯 **Admin Management Overview**

The SecureGuard subscription admin system provides comprehensive tools for managing subscription plans (abo models), features, limits, and user migrations. This guide covers all administrative capabilities for subscription management.

## 📊 **Admin Dashboard Features**

### **🏗️ Plan Management**
- ✅ **Create new subscription plans** with custom features and limits
- ✅ **Update existing plans** with impact validation
- ✅ **Delete/deactivate plans** with user migration options  
- ✅ **Clone plans** for quick setup of similar tiers
- ✅ **Bulk operations** for managing multiple plans

### **🔍 Analytics & Monitoring**
- ✅ **Plan usage statistics** - users, revenue, adoption rates
- ✅ **Feature adoption metrics** - which features are most/least used
- ✅ **User distribution** across subscription tiers
- ✅ **Revenue analysis** by plan and billing cycle
- ✅ **Limit utilization** tracking (devices, API keys, storage)

### **👥 User Management**
- ✅ **View users by subscription plan**
- ✅ **Migrate users between plans** (individual or bulk)
- ✅ **Monitor plan limit violations**
- ✅ **Trial management** and conversion tracking

## 🚀 **API Endpoints Reference**

### **📋 Plan Management Endpoints**

```bash
# Get all subscription plans
GET /api/admin/subscriptions/plans
GET /api/admin/subscriptions/plans?include_inactive=true

# Get specific plan
GET /api/admin/subscriptions/plans/{plan_id}

# Create new plan
POST /api/admin/subscriptions/plans
{
  "plan_name": "SecureGuard Pro Plus",
  "plan_slug": "pro_plus", 
  "display_name": "SecureGuard Pro Plus",
  "description": "Enhanced professional plan with advanced features",
  "max_devices": 50,
  "max_api_keys": 75,
  "features": {
    "real_time_monitoring": true,
    "advanced_threat_detection": true,
    "custom_rules": true,
    "api_access": true,
    "priority_support": true,
    "audit_logs": true,
    "integrations_enabled": true,
    "vulnerability_scanning": true,
    "compliance_reporting": true,
    "remote_response": true,
    "custom_dashboards": true,
    "bulk_operations": false
  },
  "retention": {
    "log_retention_days": 180,
    "alert_history_days": 365
  },
  "pricing": {
    "monthly_price_cents": 4999,
    "yearly_price_cents": 49999
  },
  "sort_order": 5
}

# Update plan
PUT /api/admin/subscriptions/plans/{plan_id}
{
  "max_devices": 75,
  "features": {
    "bulk_operations": true,
    "compliance_reporting": true
  },
  "pricing": {
    "monthly_price_cents": 5999
  }
}

# Delete/deactivate plan
DELETE /api/admin/subscriptions/plans/{plan_id}
DELETE /api/admin/subscriptions/plans/{plan_id}?force=true
```

### **📊 Analytics Endpoints**

```bash
# Get plan statistics
GET /api/admin/subscriptions/plans/stats
GET /api/admin/subscriptions/plans/stats?include_revenue=true&date_range=last_30_days

# Get specific plan statistics
GET /api/admin/subscriptions/plans/{plan_id}/stats

# Get plan usage details
GET /api/admin/subscriptions/plans/{plan_id}/usage

# Get feature usage across all plans
GET /api/admin/subscriptions/features/usage

# Get available features list
GET /api/admin/subscriptions/features
```

### **👥 User Management Endpoints**

```bash
# Get users on specific plan
GET /api/admin/subscriptions/plans/{plan_id}/users
GET /api/admin/subscriptions/plans/{plan_id}/users?limit=100&offset=0

# Migrate users between plans
POST /api/admin/subscriptions/plans/migrate
{
  "from_plan_id": "uuid-here",
  "to_plan_id": "uuid-here", 
  "user_ids": ["user1-uuid", "user2-uuid"], // Optional - if null, migrates all
  "migration_date": "2024-01-01T00:00:00Z", // Optional - if null, immediate
  "send_notifications": true,
  "reason": "Plan consolidation - migrating Pro users to Pro Plus"
}

# Validate plan changes before applying
POST /api/admin/subscriptions/plans/{plan_id}/validate
{
  "max_devices": 10,
  "features": {
    "advanced_threat_detection": false
  }
}
```

### **🔧 System Management Endpoints**

```bash
# Export all plans configuration
GET /api/admin/subscriptions/plans/export

# Import plans configuration
POST /api/admin/subscriptions/plans/import

# Check subscription system health
GET /api/admin/subscriptions/health

# Bulk operations
POST /api/admin/subscriptions/plans/bulk-update
POST /api/admin/subscriptions/plans/bulk-activate
POST /api/admin/subscriptions/plans/bulk-deactivate
```

## 🏗️ **Creating New Subscription Plans**

### **Step-by-Step Plan Creation**

1. **Plan Basic Information**
   ```json
   {
     "plan_name": "SecureGuard Premium",
     "plan_slug": "premium",
     "display_name": "SecureGuard Premium", 
     "description": "Advanced security for enterprise teams"
   }
   ```

2. **Device & API Limits**
   ```json
   {
     "max_devices": 100,
     "max_api_keys": 150
   }
   ```
   - Use `-1` for unlimited (Enterprise plans)
   - Ensure API keys ≥ devices for logical consistency

3. **Feature Configuration**
   ```json
   {
     "features": {
       "real_time_monitoring": true,
       "advanced_threat_detection": true,
       "custom_rules": true,
       "api_access": true,
       "priority_support": true,
       "audit_logs": true,
       "integrations_enabled": true,
       "vulnerability_scanning": true,
       "compliance_reporting": true,
       "remote_response": true,
       "custom_dashboards": true,
       "bulk_operations": true
     }
   }
   ```

4. **Data Retention**
   ```json
   {
     "retention": {
       "log_retention_days": 365,
       "alert_history_days": 730
     }
   }
   ```

5. **Pricing Structure**
   ```json
   {
     "pricing": {
       "monthly_price_cents": 7999,  // $79.99/month
       "yearly_price_cents": 79999   // $799.99/year (17% savings)
     }
   }
   ```

### **Plan Validation Rules**

- ✅ **Plan slug must be unique**
- ✅ **Device limits ≥ 1 or -1 (unlimited)**
- ✅ **API key limits ≥ device limits** (logical consistency)
- ✅ **Retention days ≥ 1**
- ✅ **Pricing ≥ 0**
- ✅ **Feature dependencies respected** (e.g., advanced_threat_detection requires real_time_monitoring)

## 🔄 **Plan Modification & Migration**

### **Safe Plan Updates**

Before updating a plan, always validate the impact:

```bash
POST /api/admin/subscriptions/plans/{plan_id}/validate
{
  "max_devices": 20,  // Reducing from 25
  "features": {
    "advanced_threat_detection": false  // Removing feature
  }
}
```

**Response indicates impact:**
```json
{
  "is_valid": true,
  "warnings": [
    "3 users exceed new device limit of 20",
    "Features being removed: advanced_threat_detection"
  ],
  "errors": [],
  "affected_users": 15,
  "impact_analysis": {
    "users_exceeding_device_limit": 3,
    "users_exceeding_api_key_limit": 0,
    "features_being_removed": ["advanced_threat_detection"],
    "data_retention_impact": "No impact",
    "estimated_revenue_impact": -5000
  }
}
```

### **User Migration Process**

1. **Analyze Current Distribution**
   ```bash
   GET /api/admin/subscriptions/plans/{old_plan_id}/users
   ```

2. **Validate Migration Impact**
   ```bash
   POST /api/admin/subscriptions/plans/{new_plan_id}/validate
   ```

3. **Execute Migration**
   ```bash
   POST /api/admin/subscriptions/plans/migrate
   {
     "from_plan_id": "old-plan-uuid",
     "to_plan_id": "new-plan-uuid",
     "send_notifications": true,
     "reason": "Plan upgrade - enhanced features"
   }
   ```

## 📊 **Analytics & Reporting**

### **Plan Performance Metrics**

```json
{
  "plan_usage_stats": [
    {
      "plan_id": "uuid",
      "plan_name": "Professional",
      "total_users": 1250,
      "active_users": 1180,
      "trial_users": 70,
      "monthly_revenue_cents": 3744000,
      "yearly_revenue_cents": 1200000,
      "avg_devices_per_user": 8.5,
      "avg_api_keys_per_user": 12.3,
      "feature_adoption": {
        "real_time_monitoring": {
          "users_with_access": 1250,
          "users_actively_using": 1050,
          "adoption_percentage": 84.0
        },
        "advanced_threat_detection": {
          "users_with_access": 1250,
          "users_actively_using": 875,
          "adoption_percentage": 70.0
        }
      }
    }
  ]
}
```

### **Feature Adoption Analysis**

```json
{
  "feature_adoption": {
    "real_time_monitoring": {
      "total_plans_with_feature": 3,
      "total_users_with_access": 2500,
      "adoption_rate": 0.85
    },
    "advanced_threat_detection": {
      "total_plans_with_feature": 2,
      "total_users_with_access": 1500,
      "adoption_rate": 0.72
    }
  },
  "most_popular_features": [
    "real_time_monitoring",
    "api_access", 
    "audit_logs"
  ],
  "least_used_features": [
    "bulk_operations",
    "compliance_reporting"
  ]
}
```

## 🔐 **Security & Access Control**

### **Admin Role Requirements**

All subscription management endpoints require `SystemAdmin` role:

```rust
.layer(RequireRole::new(UserRole::SystemAdmin))
```

### **Audit Logging**

All admin actions are automatically logged:
- ✅ Plan creation/modification/deletion
- ✅ User migrations between plans
- ✅ Feature changes and impact
- ✅ Bulk operations
- ✅ System configuration changes

## 🎯 **Common Use Cases**

### **1. Creating a New Plan Tier**

**Scenario:** Adding a "Pro Plus" plan between Professional and Enterprise

```bash
POST /api/admin/subscriptions/plans
{
  "plan_name": "Pro Plus",
  "plan_slug": "pro_plus",
  "display_name": "SecureGuard Pro Plus",
  "description": "Enhanced Professional plan with compliance features",
  "max_devices": 50,
  "max_api_keys": 75,
  "features": {
    "real_time_monitoring": true,
    "advanced_threat_detection": true,
    "custom_rules": true,
    "api_access": true,
    "priority_support": true,
    "audit_logs": true,
    "integrations_enabled": true,
    "vulnerability_scanning": true,
    "compliance_reporting": true,  // New feature
    "remote_response": true,
    "custom_dashboards": true,
    "bulk_operations": false
  },
  "retention": {
    "log_retention_days": 180,
    "alert_history_days": 365
  },
  "pricing": {
    "monthly_price_cents": 4999,
    "yearly_price_cents": 49999
  },
  "sort_order": 35  // Between Pro (30) and Enterprise (40)
}
```

### **2. Adjusting Plan Limits**

**Scenario:** Increasing device limits due to customer feedback

```bash
# Validate impact first
POST /api/admin/subscriptions/plans/{professional_plan_id}/validate
{
  "max_devices": 35,  // Increasing from 25
  "max_api_keys": 50  // Increasing from 35
}

# Apply the changes
PUT /api/admin/subscriptions/plans/{professional_plan_id}
{
  "max_devices": 35,
  "max_api_keys": 50
}
```

### **3. Feature Rollout**

**Scenario:** Rolling out new "Compliance Reporting" feature to Professional+ plans

```bash
# Add to Professional plan
PUT /api/admin/subscriptions/plans/{professional_plan_id}
{
  "features": {
    "compliance_reporting": true
  }
}

# Add to Enterprise plan  
PUT /api/admin/subscriptions/plans/{enterprise_plan_id}
{
  "features": {
    "compliance_reporting": true
  }
}
```

### **4. Plan Consolidation**

**Scenario:** Merging two similar plans

```bash
# Migrate all users from old plan to new plan
POST /api/admin/subscriptions/plans/migrate
{
  "from_plan_id": "old-professional-plan-uuid",
  "to_plan_id": "new-professional-plus-plan-uuid",
  "send_notifications": true,
  "reason": "Plan consolidation - enhanced features at same price"
}

# Deactivate old plan
DELETE /api/admin/subscriptions/plans/{old_plan_id}
```

### **5. Emergency Feature Disable**

**Scenario:** Temporarily disable a feature due to technical issues

```bash
# Disable advanced threat detection across all plans
PUT /api/admin/subscriptions/plans/{professional_plan_id}
{
  "features": {
    "advanced_threat_detection": false
  }
}

PUT /api/admin/subscriptions/plans/{enterprise_plan_id}
{
  "features": {
    "advanced_threat_detection": false
  }
}
```

## 📈 **Business Intelligence**

### **Revenue Analysis**

```bash
GET /api/admin/subscriptions/plans/stats?include_revenue=true&date_range=last_quarter
```

**Key Metrics:**
- Revenue per plan tier
- ARPU (Average Revenue Per User)
- Conversion rates between tiers
- Churn analysis by plan
- Trial-to-paid conversion rates

### **Growth Optimization**

- **Plan Performance:** Identify most/least popular plans
- **Feature Adoption:** See which features drive upgrades
- **Limit Analysis:** Find optimal limits that encourage upgrades
- **Pricing Elasticity:** Test price changes impact on conversions

## 🚨 **Monitoring & Alerts**

### **System Health Monitoring**

```bash
GET /api/admin/subscriptions/health
```

**Health Check Includes:**
- ✅ Database connectivity
- ✅ Plan configuration validity  
- ✅ User subscription consistency
- ✅ Usage tracking accuracy
- ✅ Payment system integration

### **Automated Alerts**

- ⚠️ **Plan limit violations** - Users exceeding their subscription limits
- ⚠️ **Feature access errors** - Users trying to access unavailable features
- ⚠️ **Payment failures** - Subscription payment processing issues
- ⚠️ **System inconsistencies** - Data integrity problems

## 💼 **Best Practices**

### **Plan Design Guidelines**

1. **Logical Progression**
   - Each tier should provide clear value over the previous
   - Limits should scale appropriately with pricing
   - Feature combinations should make business sense

2. **API Key Logic**
   - Always allow at least 1 backup key per plan
   - Scale API keys with device limits + integration needs
   - Enterprise plans should have unlimited or very high limits

3. **Feature Dependencies**
   - Advanced features should require basic features
   - Don't create impossible feature combinations
   - Consider technical dependencies in the agent

4. **Retention Policies**
   - Higher tiers get longer retention
   - Balance cost vs. customer value
   - Consider compliance requirements

### **Migration Best Practices**

1. **Always Validate First**
   - Check impact before making changes
   - Identify users who might be affected
   - Plan communication strategy

2. **Gradual Rollouts**
   - Test with small user groups first
   - Monitor for issues during migration
   - Have rollback plan ready

3. **Clear Communication**
   - Notify users of plan changes
   - Explain new features and benefits
   - Provide migration timelines

## 🎯 **Summary**

The SecureGuard subscription admin system provides:

✅ **Complete Plan Management** - Create, update, delete subscription plans
✅ **Feature Control** - Add/remove features from plans dynamically  
✅ **Limit Management** - Adjust device, API key, and retention limits
✅ **User Migration** - Move users between plans safely
✅ **Analytics & Insights** - Track plan performance and feature adoption
✅ **Validation & Safety** - Prevent breaking changes with impact analysis
✅ **Bulk Operations** - Efficiently manage multiple plans
✅ **Audit Logging** - Track all administrative changes
✅ **Health Monitoring** - Ensure system integrity

This system gives you complete control over your subscription business model while maintaining data integrity and user experience! 🚀