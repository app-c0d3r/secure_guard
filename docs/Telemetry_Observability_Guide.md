# SecureGuard Telemetry & Observability Guide

## Overview

SecureGuard implements comprehensive observability using OpenTelemetry, providing distributed tracing, metrics collection, and performance monitoring across both backend and frontend applications.

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Frontend      │────▶│  OTEL Collector │────▶│     Jaeger      │
│  (Browser SDK)  │     │                 │     │   (Tracing)     │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                               │                          
┌─────────────────┐            │              ┌─────────────────┐
│    Backend      │────────────┘              │   Prometheus    │
│  (Rust OTLP)    │───────────────────────────│   (Metrics)     │
└─────────────────┘                           └─────────────────┘
                                                        │
                                              ┌─────────────────┐
                                              │    Grafana      │
                                              │ (Visualization) │
                                              └─────────────────┘
```

## Quick Start

### 1. Start Telemetry Infrastructure

```bash
# Start Jaeger, Prometheus, Grafana, and OTEL Collector
docker-compose -f docker-compose.telemetry.yml up -d

# Verify services are running
docker-compose -f docker-compose.telemetry.yml ps
```

### 2. Access Telemetry UIs

- **Jaeger UI**: http://localhost:16686
- **Prometheus UI**: http://localhost:9090
- **Grafana**: http://localhost:3001 (admin/secureguard)
- **API Metrics**: http://localhost:8080/metrics

### 3. Configure Environment Variables

```bash
# Backend (.env)
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
ENVIRONMENT=development

# Frontend (.env.local)
VITE_OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
VITE_ENVIRONMENT=development
```

## Backend Telemetry

### Features

- **Distributed Tracing**: Automatic trace propagation across services
- **HTTP Middleware**: Automatic instrumentation of all HTTP endpoints
- **Database Tracking**: Query performance monitoring
- **Custom Metrics**: Business-specific metrics collection
- **Error Tracking**: Automatic error capture and reporting

### Available Metrics

```rust
// API Metrics
api.requests         // Total API requests (counter)
api.errors          // Total API errors (counter)
api.duration        // Request duration (histogram)

// Agent Metrics
agents.active       // Active agents (gauge)
security.events     // Security events processed (counter)

// WebSocket Metrics
websocket.connections  // Active connections (gauge)

// Database Metrics
database.queries       // Total queries (counter)
database.query.duration // Query duration (histogram)
```

### Custom Instrumentation

```rust
use opentelemetry::trace::Tracer;
use secureguard_api::telemetry;

// Trace a custom operation
async fn process_data() {
    let tracer = global::tracer("secureguard-api");
    let span = tracer.span_builder("process_data")
        .with_attributes(vec![
            KeyValue::new("data.type", "security_event"),
        ])
        .start(&tracer);
    
    // Your code here
    
    span.end();
}

// Trace database operations
let result = telemetry::trace_db_query("get_user", async {
    sqlx::query("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
}).await?;
```

## Frontend Telemetry

### Features

- **Automatic Instrumentation**: Fetch and XHR requests
- **Component Tracking**: React component lifecycle monitoring
- **User Interactions**: Click, form, and search tracking
- **Performance Metrics**: Web Vitals and render performance
- **Error Boundary Integration**: Automatic error reporting

### Usage Examples

```typescript
import { traceApiCall, interactions, performance } from '@/telemetry';

// Trace API calls
const data = await traceApiCall('fetchAgents', 
  () => api.get('/agents'),
  { endpoint: '/agents', method: 'GET' }
);

// Track user interactions
interactions.trackClick('submit-button', {
  form: 'login',
  timestamp: Date.now()
});

// Track form submissions
interactions.trackFormSubmit('login-form', success, {
  duration: endTime - startTime
});

// Measure component render time
useEffect(() => {
  const startTime = Date.now();
  return () => {
    performance.measureRender('Dashboard', startTime);
  };
}, []);
```

## Grafana Dashboards

### Importing Dashboards

1. Access Grafana at http://localhost:3001
2. Login with admin/secureguard
3. Go to Dashboards → Import
4. Use provided dashboard JSON files or create custom ones

### Key Metrics to Monitor

#### API Performance
- Request rate (req/s)
- Error rate (%)
- P50/P95/P99 latencies
- Endpoint-specific metrics

#### System Health
- Active agents count
- WebSocket connections
- Database connection pool status
- Memory and CPU usage

#### Security Events
- Events per second
- Event severity distribution
- Alert trigger rate
- Threat detection accuracy

## Production Deployment

### 1. Secure OTLP Endpoints

```yaml
# Use TLS for production
OTEL_EXPORTER_OTLP_ENDPOINT=https://otel-collector.secureguard.com:4317
OTEL_EXPORTER_OTLP_HEADERS="Authorization=Bearer <token>"
```

### 2. Configure Sampling

```rust
// Reduce sampling in production to control volume
let trace_config = trace::config()
    .with_sampler(Sampler::TraceIdRatioBased(0.1)) // 10% sampling
    .with_resource(resource);
```

### 3. Set Resource Limits

```yaml
# docker-compose.telemetry.yml
services:
  otel-collector:
    deploy:
      resources:
        limits:
          memory: 1G
          cpus: '1.0'
```

### 4. Enable Data Retention Policies

```yaml
# prometheus.yml
global:
  external_labels:
    environment: 'production'
    
storage:
  tsdb:
    retention.time: 30d
    retention.size: 10GB
```

## Troubleshooting

### No Traces Appearing

1. Check OTLP endpoint connectivity:
```bash
curl -X POST http://localhost:4318/v1/traces \
  -H "Content-Type: application/json" \
  -d '{}'
```

2. Verify environment variables:
```bash
echo $OTEL_EXPORTER_OTLP_ENDPOINT
```

3. Check collector logs:
```bash
docker-compose -f docker-compose.telemetry.yml logs otel-collector
```

### High Memory Usage

1. Adjust batch processor settings:
```yaml
processors:
  batch:
    timeout: 5s
    send_batch_size: 512
```

2. Enable memory limiter:
```yaml
processors:
  memory_limiter:
    limit_mib: 256
    spike_limit_mib: 64
```

### Missing Metrics

1. Verify Prometheus scrape config:
```bash
curl http://localhost:8080/metrics
```

2. Check Prometheus targets:
   - Go to http://localhost:9090/targets
   - Verify all targets are UP

## Performance Impact

### Backend
- Tracing overhead: ~2-5% CPU increase
- Memory usage: ~50-100MB additional
- Latency impact: <1ms per request

### Frontend
- Bundle size: ~30KB gzipped
- Runtime overhead: <2% CPU
- Network: 1-2 additional requests per page

## Best Practices

### 1. Use Semantic Conventions
```rust
// Use standard attribute names
span.set_attribute(KeyValue::new("http.method", "POST"));
span.set_attribute(KeyValue::new("http.status_code", 200));
```

### 2. Add Business Context
```typescript
// Include business-relevant attributes
span.setAttributes({
  'user.id': userId,
  'tenant.id': tenantId,
  'feature.flag': featureFlagValue
});
```

### 3. Batch Operations
```rust
// Batch multiple operations under single span
let span = tracer.span_builder("batch_process").start(&tracer);
for item in items {
    process_item(item).await;
}
span.end();
```

### 4. Handle Sensitive Data
```typescript
// Don't log sensitive information
span.setAttribute('user.email_hash', hashEmail(email)); // Good
span.setAttribute('user.email', email); // Bad
```

## Alerts Configuration

### Example Prometheus Alert Rules

```yaml
groups:
  - name: api_alerts
    rules:
      - alert: HighErrorRate
        expr: rate(api_errors[5m]) > 0.05
        for: 5m
        annotations:
          summary: "High API error rate"
          
      - alert: SlowAPIResponse
        expr: histogram_quantile(0.95, api_duration) > 1
        for: 10m
        annotations:
          summary: "API P95 latency above 1s"
```

## Integration with CI/CD

### Performance Testing
```bash
# Run load test with telemetry
npm run test:load
# Check metrics during test
curl http://localhost:8080/metrics | grep api_duration
```

### Deployment Verification
```yaml
# GitHub Actions example
- name: Verify Telemetry
  run: |
    curl -f http://localhost:8080/health
    curl -f http://localhost:8080/metrics
```

## Support

For telemetry-related issues:
1. Check this documentation
2. Review collector logs
3. Verify network connectivity
4. Contact the DevOps team

## Resources

- [OpenTelemetry Documentation](https://opentelemetry.io/docs/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)