# SecureGuard Telemetry Quick Start Guide

## 🚀 5-Minute Setup

Get telemetry running in 5 minutes with these simple steps:

### Step 1: Start Infrastructure (1 minute)
```bash
# From project root
docker-compose -f docker-compose.telemetry.yml up -d
```

### Step 2: Verify Services (30 seconds)
```bash
docker-compose -f docker-compose.telemetry.yml ps
```

All services should show as "Up":
- secureguard-jaeger
- secureguard-prometheus  
- secureguard-grafana
- secureguard-otel-collector

### Step 3: Start SecureGuard (1 minute)
```bash
# Terminal 1: Start API
cargo run -p secureguard-api

# Terminal 2: Start Frontend
cd frontend && npm run dev
```

### Step 4: Access Monitoring UIs (30 seconds)

| Service | URL | Purpose |
|---------|-----|---------|
| **Jaeger** | http://localhost:16686 | View distributed traces |
| **Prometheus** | http://localhost:9090 | Query metrics |
| **Grafana** | http://localhost:3001 | View dashboards |
| **Metrics** | http://localhost:8080/metrics | Raw metrics endpoint |

**Grafana Login:** admin / secureguard

### Step 5: Generate Test Data (2 minutes)
```bash
# Make some API calls to generate traces
curl http://localhost:8080/api/v1/health
curl http://localhost:8080/api/v1/agents

# Or use the frontend
# Navigate through different pages at http://localhost:3000
```

## 📊 View Your First Trace

1. Open Jaeger: http://localhost:16686
2. Select Service: `secureguard-api`
3. Click "Find Traces"
4. Click on any trace to see the detailed request flow

## 📈 View Metrics

1. Open Prometheus: http://localhost:9090
2. Try these queries:
   ```promql
   # API request rate
   rate(api_requests[5m])
   
   # Error rate
   rate(api_errors[5m])
   
   # P95 latency
   histogram_quantile(0.95, rate(api_duration_bucket[5m]))
   ```

## 🎨 Grafana Dashboards

1. Open Grafana: http://localhost:3001
2. Go to Dashboards → Browse
3. Import dashboard:
   - Click "New" → "Import"
   - Use ID: 1860 (Node Exporter Full)
   - Select Prometheus data source

## 🔍 Common Queries

### Jaeger Traces
- **Find slow requests**: Set min duration to 1s
- **Find errors**: Add tag `error=true`
- **Find by endpoint**: Add tag `http.target=/api/v1/agents`

### Prometheus Metrics
```promql
# Top 5 slowest endpoints
topk(5, histogram_quantile(0.95, rate(api_duration_bucket[5m]))) by (endpoint)

# Request rate by endpoint
sum(rate(api_requests[5m])) by (endpoint)

# Active agents
agents_active

# WebSocket connections
websocket_connections
```

## 🛠️ Troubleshooting

### No traces appearing?
```bash
# Check if OTLP endpoint is accessible
curl -X POST http://localhost:4318/v1/traces -H "Content-Type: application/json" -d '{}'

# Check API logs for telemetry initialization
cargo run -p secureguard-api 2>&1 | grep -i telemetry
```

### No metrics in Prometheus?
```bash
# Check if metrics endpoint is working
curl http://localhost:8080/metrics

# Check Prometheus targets
# Go to http://localhost:9090/targets
# All targets should be "UP"
```

### Grafana not loading?
```bash
# Check container logs
docker-compose -f docker-compose.telemetry.yml logs grafana

# Restart Grafana
docker-compose -f docker-compose.telemetry.yml restart grafana
```

## 🏃 Performance Tips

### Development
- Full tracing enabled (100% sampling)
- All metrics collected
- Detailed span attributes

### Production
- Reduce sampling to 10%
- Set resource limits
- Enable TLS on endpoints
- Use persistent storage

## 📚 Learn More

- [Full Telemetry Guide](Telemetry_Observability_Guide.md)
- [OpenTelemetry Docs](https://opentelemetry.io/docs/)
- [Jaeger Tutorial](https://www.jaegertracing.io/docs/latest/getting-started/)
- [Prometheus Queries](https://prometheus.io/docs/prometheus/latest/querying/basics/)

## 💡 Pro Tips

1. **Correlate traces and metrics**: Use trace ID in Prometheus queries
2. **Custom spans**: Add business context to traces
3. **Alert on SLOs**: Set up alerts for service level objectives
4. **Dashboard templates**: Export and share Grafana dashboards
5. **Trace sampling**: Adjust sampling for cost vs visibility balance

## 🔄 Stop Telemetry

When you're done:
```bash
# Stop all telemetry services
docker-compose -f docker-compose.telemetry.yml down

# Remove volumes (optional, will delete data)
docker-compose -f docker-compose.telemetry.yml down -v
```

---

**Need help?** Check the [full documentation](Telemetry_Observability_Guide.md) or open an issue.