# SecureGuard Security Monitoring & Alerting Guide

## Overview

This guide provides comprehensive strategies for monitoring SecureGuard's security events, setting up alerts, and implementing incident response procedures based on the enhanced logging system.

## 🔍 Security Event Categories

### Authentication & Authorization Events

**High Priority Events:**
- Failed login attempts (potential brute force)
- Account lockouts (security policy violations)
- Multiple failed password attempts
- Invalid JWT token usage
- Privilege escalation attempts

**Medium Priority Events:**
- Password changes
- Successful logins from new locations/IPs
- API key generation/deletion
- Role changes
- Long-inactive account activity

**Low Priority Events:**
- Successful routine logins
- Normal password verification
- Standard API key usage
- Regular token refresh

### API Security Events

**Critical Events:**
- Multiple failed API key validation attempts
- Expired API key usage attempts
- API key brute force patterns
- Unusual API call patterns
- Rate limiting violations

**Important Events:**
- New API key creation
- API key deletion
- High API usage spikes
- API response errors (4xx/5xx)

### Agent Security Events

**Critical Events:**
- Agent registration failures
- Invalid agent authentication
- Device limit violations
- Agent communication tampering
- Unexpected agent disconnections

**Monitoring Events:**
- New agent registrations
- Agent status changes
- Heartbeat irregularities
- Agent update events

## 📊 Log Analysis Patterns

### Brute Force Attack Detection

**Pattern Recognition:**
```bash
# Detect multiple failed login attempts from same IP
cat logs/security-audit.log.* | \
  jq -r 'select(.fields.event == "login_failed") | "\(.fields.ip_address // "unknown") \(.timestamp)"' | \
  sort | uniq -c | sort -nr | head -10

# Alternative: Count by username
cat logs/security-audit.log.* | \
  jq -r 'select(.fields.event == "login_failed") | "\(.fields.username) \(.timestamp)"' | \
  sort | uniq -c | sort -nr | head -10
```

**Alert Thresholds:**
- **Critical**: >20 failed attempts in 5 minutes
- **Warning**: >10 failed attempts in 15 minutes  
- **Info**: >5 failed attempts in 1 hour

### Account Security Monitoring

**Suspicious Login Patterns:**
```bash
# Find accounts with frequent lockouts
cat logs/security-audit.log.* | \
  jq -r 'select(.audit == "account_locked_login_attempt") | .fields.username' | \
  sort | uniq -c | sort -nr

# Analyze login timing patterns
cat logs/security-audit.log.* | \
  jq -r 'select(.fields.event | contains("login")) | "\(.fields.username) \(.timestamp | strptime("%Y-%m-%dT%H:%M:%S") | strftime("%H"))"' | \
  sort | uniq -c
```

### API Key Abuse Detection

**High-Risk API Key Patterns:**
```bash
# Find API keys with high failure rates
cat logs/security-audit.log.* | \
  jq -r 'select(.fields.event == "api_key_validation_failed") | .fields.key_id' | \
  sort | uniq -c | sort -nr | head -20

# Detect API key enumeration attempts
cat logs/security-audit.log.* | \
  jq -r 'select(.fields.event == "api_key_validation_failed" and .fields.reason == "key_not_found") | .fields.key_prefix' | \
  sort | uniq -c | sort -nr
```

## 🚨 Automated Alerting Configuration

### ELK Stack Alerting

**Watcher Configuration for High Failed Login Rate:**
```json
{
  "trigger": {
    "schedule": {
      "interval": "5m"
    }
  },
  "input": {
    "search": {
      "request": {
        "search_type": "query_then_fetch",
        "indices": ["secureguard-*"],
        "body": {
          "query": {
            "bool": {
              "must": [
                {"match": {"fields.event": "login_failed"}},
                {"range": {"@timestamp": {"gte": "now-5m"}}}
              ]
            }
          },
          "aggs": {
            "failed_attempts": {
              "cardinality": {
                "field": "fields.username.keyword"
              }
            }
          }
        }
      }
    }
  },
  "condition": {
    "compare": {
      "ctx.payload.aggregations.failed_attempts.value": {
        "gt": 20
      }
    }
  },
  "actions": {
    "send_email": {
      "email": {
        "to": ["security@yourcompany.com"],
        "subject": "CRITICAL: High Failed Login Rate Detected",
        "body": "{{ctx.payload.aggregations.failed_attempts.value}} unique users had failed login attempts in the last 5 minutes."
      }
    }
  }
}
```

### Grafana Alerting Rules

**Prometheus Alert Rules:**
```yaml
# /etc/prometheus/alert_rules.yml
groups:
  - name: secureguard.security
    rules:
      - alert: HighFailedLoginRate
        expr: increase(secureguard_failed_logins_total[5m]) > 20
        for: 0s
        labels:
          severity: critical
          service: secureguard
        annotations:
          summary: "High failed login rate detected"
          description: "{{ $value }} failed login attempts in the last 5 minutes"

      - alert: AccountLockoutSpike
        expr: increase(secureguard_account_lockouts_total[1h]) > 5
        for: 2m
        labels:
          severity: warning
          service: secureguard
        annotations:
          summary: "Multiple account lockouts detected"
          description: "{{ $value }} account lockouts in the last hour"

      - alert: APIKeyAbuseDetected
        expr: increase(secureguard_api_key_failures_total[10m]) > 50
        for: 1m
        labels:
          severity: critical
          service: secureguard
        annotations:
          summary: "API key abuse pattern detected"
          description: "{{ $value }} failed API key attempts in 10 minutes"

      - alert: AgentRegistrationFailures
        expr: increase(secureguard_agent_registration_failures_total[30m]) > 10
        for: 5m
        labels:
          severity: warning
          service: secureguard
        annotations:
          summary: "High agent registration failure rate"
          description: "{{ $value }} agent registration failures in 30 minutes"
```

### Slack/Teams Integration

**Webhook Alert Script:**
```bash
#!/bin/bash
# security_alert.sh - Send security alerts to Slack/Teams

WEBHOOK_URL="YOUR_WEBHOOK_URL"
ALERT_TYPE="$1"
ALERT_MESSAGE="$2"
SEVERITY="$3"

case $SEVERITY in
  "critical")
    COLOR="#FF0000"
    EMOJI="🚨"
    ;;
  "warning")
    COLOR="#FFA500"
    EMOJI="⚠️"
    ;;
  *)
    COLOR="#00FF00"
    EMOJI="ℹ️"
    ;;
esac

curl -X POST -H 'Content-type: application/json' \
  --data "{
    \"text\": \"$EMOJI SecureGuard Security Alert\",
    \"attachments\": [
      {
        \"color\": \"$COLOR\",
        \"fields\": [
          {\"title\": \"Alert Type\", \"value\": \"$ALERT_TYPE\", \"short\": true},
          {\"title\": \"Severity\", \"value\": \"$SEVERITY\", \"short\": true},
          {\"title\": \"Message\", \"value\": \"$ALERT_MESSAGE\", \"short\": false}
        ],
        \"footer\": \"SecureGuard Security Monitor\",
        \"ts\": $(date +%s)
      }
    ]
  }" \
  $WEBHOOK_URL
```

## 📈 Security Dashboards

### Grafana Dashboard Configuration

**Security Overview Dashboard:**
```json
{
  "dashboard": {
    "title": "SecureGuard Security Overview",
    "panels": [
      {
        "title": "Failed Login Attempts",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(secureguard_failed_logins_total[5m])",
            "legendFormat": "Failed Logins/sec"
          }
        ]
      },
      {
        "title": "Account Lockouts",
        "type": "stat",
        "targets": [
          {
            "expr": "increase(secureguard_account_lockouts_total[24h])",
            "legendFormat": "24h Lockouts"
          }
        ]
      },
      {
        "title": "API Key Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(secureguard_api_key_validations_total[5m])",
            "legendFormat": "Valid"
          },
          {
            "expr": "rate(secureguard_api_key_failures_total[5m])",
            "legendFormat": "Failed"
          }
        ]
      },
      {
        "title": "Agent Security Events",
        "type": "table",
        "targets": [
          {
            "expr": "topk(10, increase(secureguard_agent_events_total[1h]))",
            "format": "table"
          }
        ]
      }
    ]
  }
}
```

### Kibana Dashboard Queries

**Security Event Analysis Queries:**
```
# Top failed login usernames (last 24h)
GET /secureguard-*/search
{
  "query": {
    "bool": {
      "must": [
        {"match": {"fields.event": "login_failed"}},
        {"range": {"@timestamp": {"gte": "now-24h"}}}
      ]
    }
  },
  "aggs": {
    "top_users": {
      "terms": {
        "field": "fields.username.keyword",
        "size": 10
      }
    }
  }
}

# API key failure patterns (last 1h)
GET /secureguard-*/search
{
  "query": {
    "bool": {
      "must": [
        {"match": {"fields.event": "api_key_validation_failed"}},
        {"range": {"@timestamp": {"gte": "now-1h"}}}
      ]
    }
  },
  "aggs": {
    "by_reason": {
      "terms": {
        "field": "fields.reason.keyword"
      }
    },
    "timeline": {
      "date_histogram": {
        "field": "@timestamp",
        "calendar_interval": "5m"
      }
    }
  }
}
```

## 🔧 Custom Monitoring Scripts

### Real-Time Security Monitor

**Python Security Monitor:**
```python
#!/usr/bin/env python3
"""
SecureGuard Real-time Security Monitor
Watches log files and sends alerts for security events
"""
import json
import time
import subprocess
from datetime import datetime, timedelta
from collections import defaultdict

class SecurityMonitor:
    def __init__(self):
        self.failed_logins = defaultdict(list)
        self.api_failures = defaultdict(list)
        self.lockout_events = []
        
    def check_failed_logins(self, event):
        """Monitor for brute force login attempts"""
        if event.get('fields', {}).get('event') == 'login_failed':
            username = event.get('fields', {}).get('username')
            timestamp = datetime.fromisoformat(event['timestamp'].replace('Z', '+00:00'))
            
            self.failed_logins[username].append(timestamp)
            
            # Clean old entries (> 15 minutes)
            cutoff = datetime.now() - timedelta(minutes=15)
            self.failed_logins[username] = [
                t for t in self.failed_logins[username] if t > cutoff
            ]
            
            # Alert if too many failures
            if len(self.failed_logins[username]) >= 10:
                self.send_alert(
                    "CRITICAL", 
                    f"Brute force detected for user {username}",
                    f"{len(self.failed_logins[username])} failures in 15 minutes"
                )
                
    def check_api_abuse(self, event):
        """Monitor for API key abuse"""
        if event.get('fields', {}).get('event') == 'api_key_validation_failed':
            key_prefix = event.get('fields', {}).get('key_prefix', 'unknown')
            timestamp = datetime.fromisoformat(event['timestamp'].replace('Z', '+00:00'))
            
            self.api_failures[key_prefix].append(timestamp)
            
            # Clean old entries (> 5 minutes)
            cutoff = datetime.now() - timedelta(minutes=5)
            self.api_failures[key_prefix] = [
                t for t in self.api_failures[key_prefix] if t > cutoff
            ]
            
            # Alert if too many failures
            if len(self.api_failures[key_prefix]) >= 20:
                self.send_alert(
                    "WARNING",
                    f"API key abuse detected",
                    f"{len(self.api_failures[key_prefix])} failures for key {key_prefix}"
                )
    
    def send_alert(self, severity, title, message):
        """Send security alert"""
        subprocess.run([
            "./security_alert.sh",
            title,
            message,
            severity.lower()
        ])
        print(f"{datetime.now()}: {severity} - {title}: {message}")
    
    def monitor_logs(self):
        """Main monitoring loop"""
        print("Starting SecureGuard security monitor...")
        
        # Use tail -F to follow log rotation
        cmd = ["tail", "-F", "logs/security-audit.log.*"]
        process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        
        try:
            for line in iter(process.stdout.readline, b''):
                try:
                    event = json.loads(line.decode('utf-8').strip())
                    self.check_failed_logins(event)
                    self.check_api_abuse(event)
                except json.JSONDecodeError:
                    continue  # Skip non-JSON lines
                except Exception as e:
                    print(f"Error processing event: {e}")
                    
        except KeyboardInterrupt:
            print("Stopping security monitor...")
            process.terminate()

if __name__ == "__main__":
    monitor = SecurityMonitor()
    monitor.monitor_logs()
```

### Log Aggregation Script

**Bash Security Report Generator:**
```bash
#!/bin/bash
# generate_security_report.sh - Daily security summary

DATE=${1:-$(date +%Y-%m-%d)}
SECURITY_LOG="logs/security-audit.log.$DATE"
REPORT_FILE="security_report_$DATE.txt"

echo "SecureGuard Security Report - $DATE" > $REPORT_FILE
echo "============================================" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Failed login summary
echo "FAILED LOGIN ATTEMPTS" >> $REPORT_FILE
echo "---------------------" >> $REPORT_FILE
cat $SECURITY_LOG | \
  jq -r 'select(.fields.event == "login_failed") | .fields.username' | \
  sort | uniq -c | sort -nr | head -20 >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Account lockout summary  
echo "ACCOUNT LOCKOUTS" >> $REPORT_FILE
echo "----------------" >> $REPORT_FILE
cat $SECURITY_LOG | \
  jq -r 'select(.audit == "account_locked_login_attempt") | "\(.fields.username) - \(.fields.failed_attempts) attempts"' | \
  sort | uniq >> $REPORT_FILE
echo "" >> $REPORT_FILE

# API key issues
echo "API KEY SECURITY EVENTS" >> $REPORT_FILE  
echo "-----------------------" >> $REPORT_FILE
echo "Failed validations by key:" >> $REPORT_FILE
cat $SECURITY_LOG | \
  jq -r 'select(.fields.event == "api_key_validation_failed") | .fields.key_prefix' | \
  sort | uniq -c | sort -nr >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Agent security events
echo "AGENT SECURITY EVENTS" >> $REPORT_FILE
echo "---------------------" >> $REPORT_FILE
cat $SECURITY_LOG | \
  jq -r 'select(.fields.event | contains("agent")) | "\(.fields.event) - \(.fields.reason // "N/A")"' | \
  sort | uniq -c | sort -nr >> $REPORT_FILE

echo "" >> $REPORT_FILE
echo "Report generated: $(date)" >> $REPORT_FILE

# Email report
if command -v mail &> /dev/null; then
    mail -s "SecureGuard Security Report - $DATE" security@yourcompany.com < $REPORT_FILE
fi

echo "Security report generated: $REPORT_FILE"
```

## 🎯 Incident Response Procedures

### Critical Security Event Response

**Step 1: Immediate Assessment**
```bash
# Quick security event check
tail -50 logs/security-audit.log.$(date +%Y-%m-%d) | jq 'select(.level == "ERROR" or .level == "WARN")'

# Check for ongoing attacks
tail -100 logs/security-audit.log.$(date +%Y-%m-%d) | \
  jq -r 'select(.fields.event == "login_failed") | .timestamp' | \
  tail -20
```

**Step 2: Account Protection**
```sql
-- Immediately lock suspicious accounts
UPDATE users.users 
SET account_locked_until = NOW() + INTERVAL '1 hour',
    is_active = false
WHERE username IN ('suspicious_user1', 'suspicious_user2');

-- Revoke active sessions
DELETE FROM users.user_sessions 
WHERE user_id IN (
  SELECT user_id FROM users.users 
  WHERE username IN ('suspicious_user1', 'suspicious_user2')
);
```

**Step 3: API Security Response**
```sql
-- Disable compromised API keys
UPDATE users.api_keys 
SET is_active = false, 
    revoked_at = NOW(),
    revoked_reason = 'Security incident'
WHERE key_prefix IN ('sg_abc123', 'sg_def456');

-- Audit API key usage
SELECT key_id, key_name, usage_count, last_used 
FROM users.api_keys 
WHERE last_used > NOW() - INTERVAL '1 hour'
ORDER BY usage_count DESC;
```

### Post-Incident Analysis

**Security Event Timeline Reconstruction:**
```bash
#!/bin/bash
# incident_timeline.sh - Reconstruct security incident timeline

INCIDENT_START="$1"  # e.g., "2024-08-19T10:00:00"
INCIDENT_END="$2"    # e.g., "2024-08-19T11:00:00"

echo "Security Incident Timeline: $INCIDENT_START to $INCIDENT_END"
echo "=============================================================="

# Extract all security events in timeframe
cat logs/security-audit.log.* | \
  jq -r --arg start "$INCIDENT_START" --arg end "$INCIDENT_END" '
    select((.timestamp >= $start) and (.timestamp <= $end)) | 
    "\(.timestamp) [\(.level)] \(.fields.event // "unknown") - \(.message)"
  ' | sort

echo ""
echo "Event Summary:"
echo "-------------"

# Count events by type
cat logs/security-audit.log.* | \
  jq -r --arg start "$INCIDENT_START" --arg end "$INCIDENT_END" '
    select((.timestamp >= $start) and (.timestamp <= $end)) | 
    .fields.event // "unknown"
  ' | sort | uniq -c | sort -nr
```

## 📊 Key Performance Indicators (KPIs)

### Security Metrics to Monitor

**Daily Metrics:**
- Failed login attempts per day
- Account lockouts per day
- API key failures per day
- New API keys created per day
- Agent registration failures per day
- Password policy violations per day

**Weekly Trends:**
- Failed login attempt patterns (by day of week/hour)
- Top targeted usernames
- API key abuse patterns
- Geographic distribution of failed attempts (if IP logging enabled)
- Agent security event trends

**Monthly Analysis:**
- Security incident frequency and severity
- Account lockout trends
- API key lifecycle (creation to expiration)
- Password policy effectiveness
- Security alert false positive rates

### Compliance Reporting

**SOC 2 Type II Requirements:**
```bash
# Monthly compliance report
./scripts/compliance_report.sh $(date -d "last month" +%Y-%m)

# Key metrics for compliance:
# - All authentication events logged
# - Failed attempts tracked and alerted
# - Account lockout events documented
# - API key operations audited
# - Regular security monitoring demonstrated
```

---

**Additional Resources:**
- [Logging & Monitoring Guide](Logging_Monitoring_Guide.md) - Complete logging documentation
- [Production Deployment Checklist](Production_Deployment_Checklist.md) - Deployment security checklist
- [Telemetry & Observability Guide](Telemetry_Observability_Guide.md) - OpenTelemetry integration