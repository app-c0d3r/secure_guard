use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    metrics::{reader::DefaultAggregationSelector, MeterProviderBuilder, PeriodicReader},
    propagation::TraceContextPropagator,
    runtime,
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use opentelemetry_semantic_conventions::resource::{
    DEPLOYMENT_ENVIRONMENT, SERVICE_NAME, SERVICE_VERSION,
};
use prometheus::{Encoder, TextEncoder};
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

/// Initialize OpenTelemetry with OTLP exporter for traces and Prometheus for metrics
pub fn init_telemetry() -> anyhow::Result<()> {
    // Set up resource attributes
    let resource = Resource::new(vec![
        KeyValue::new(SERVICE_NAME, "secureguard-api"),
        KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
        KeyValue::new(
            DEPLOYMENT_ENVIRONMENT,
            std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
        ),
    ]);

    // Initialize trace provider with OTLP exporter
    let trace_config = trace::config()
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(64)
        .with_max_attributes_per_span(32)
        .with_resource(resource.clone());

    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_endpoint.clone()),
        )
        .with_trace_config(trace_config)
        .install_batch(runtime::Tokio)?;

    // Set up tracing subscriber with OpenTelemetry layer
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("secureguard_api=debug,tower_http=debug,opentelemetry=trace")
    });

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    Registry::default()
        .with(env_filter)
        .with(fmt_layer)
        .with(telemetry_layer)
        .init();

    // Set global propagator for distributed tracing
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Initialize metrics with Prometheus exporter
    init_metrics(resource)?;

    tracing::info!(
        "OpenTelemetry initialized with OTLP endpoint: {}",
        otlp_endpoint
    );

    Ok(())
}

/// Initialize metrics provider with Prometheus exporter
fn init_metrics(resource: Resource) -> anyhow::Result<()> {
    let prometheus_registry = prometheus::Registry::new();
    
    let prometheus_exporter = opentelemetry_prometheus::exporter()
        .with_registry(prometheus_registry.clone())
        .build()?;

    let meter_provider = MeterProviderBuilder::default()
        .with_reader(prometheus_exporter)
        .with_resource(resource)
        .build();

    global::set_meter_provider(meter_provider);

    // Store Prometheus registry for later use
    PROMETHEUS_REGISTRY.set(prometheus_registry).map_err(|_| {
        anyhow::anyhow!("Failed to set Prometheus registry")
    })?;

    Ok(())
}

// Global Prometheus registry for metrics endpoint
use std::sync::OnceLock;
static PROMETHEUS_REGISTRY: OnceLock<prometheus::Registry> = OnceLock::new();

/// Get Prometheus metrics as string
pub fn get_metrics() -> anyhow::Result<String> {
    let registry = PROMETHEUS_REGISTRY
        .get()
        .ok_or_else(|| anyhow::anyhow!("Prometheus registry not initialized"))?;
    
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    
    Ok(String::from_utf8(buffer)?)
}

/// Shutdown telemetry providers gracefully
pub fn shutdown_telemetry() {
    tracing::info!("Shutting down OpenTelemetry providers");
    global::shutdown_tracer_provider();
}

/// Custom metrics for SecureGuard
pub mod metrics {
    use opentelemetry::{
        global,
        metrics::{Counter, Histogram, Meter, UpDownCounter},
    };
    use std::sync::OnceLock;

    pub struct Metrics {
        pub api_requests: Counter<u64>,
        pub api_errors: Counter<u64>,
        pub api_duration: Histogram<f64>,
        pub active_agents: UpDownCounter<i64>,
        pub security_events: Counter<u64>,
        pub websocket_connections: UpDownCounter<i64>,
        pub database_queries: Counter<u64>,
        pub database_query_duration: Histogram<f64>,
    }

    static METRICS: OnceLock<Metrics> = OnceLock::new();

    pub fn init() -> &'static Metrics {
        METRICS.get_or_init(|| {
            let meter = global::meter("secureguard-api");
            
            Metrics {
                api_requests: meter
                    .u64_counter("api.requests")
                    .with_description("Total number of API requests")
                    .init(),
                
                api_errors: meter
                    .u64_counter("api.errors")
                    .with_description("Total number of API errors")
                    .init(),
                
                api_duration: meter
                    .f64_histogram("api.duration")
                    .with_description("API request duration in seconds")
                    .with_unit(opentelemetry::metrics::Unit::new("s"))
                    .init(),
                
                active_agents: meter
                    .i64_up_down_counter("agents.active")
                    .with_description("Number of active agents")
                    .init(),
                
                security_events: meter
                    .u64_counter("security.events")
                    .with_description("Total number of security events processed")
                    .init(),
                
                websocket_connections: meter
                    .i64_up_down_counter("websocket.connections")
                    .with_description("Number of active WebSocket connections")
                    .init(),
                
                database_queries: meter
                    .u64_counter("database.queries")
                    .with_description("Total number of database queries")
                    .init(),
                
                database_query_duration: meter
                    .f64_histogram("database.query.duration")
                    .with_description("Database query duration in seconds")
                    .with_unit(opentelemetry::metrics::Unit::new("s"))
                    .init(),
            }
        })
    }

    pub fn get() -> &'static Metrics {
        METRICS.get().expect("Metrics not initialized")
    }
}

/// Middleware for tracing HTTP requests
use axum::{
    extract::MatchedPath,
    http::{Request, Response},
    middleware::Next,
    body::Body,
};
use opentelemetry::trace::{FutureExt, TraceContextExt, Tracer};
use std::time::Instant;

pub async fn trace_middleware(
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let start = Instant::now();
    
    let tracer = global::tracer("secureguard-api");
    let method = req.method().to_string();
    let uri = req.uri().path().to_string();
    
    // Extract matched path for better span names
    let matched_path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|mp| mp.as_str().to_string())
        .unwrap_or_else(|| uri.clone());
    
    let span = tracer
        .span_builder(format!("{} {}", method, matched_path))
        .with_kind(opentelemetry::trace::SpanKind::Server)
        .with_attributes(vec![
            KeyValue::new("http.method", method.clone()),
            KeyValue::new("http.target", uri.clone()),
            KeyValue::new("http.scheme", "http"),
        ])
        .start(&tracer);
    
    let cx = opentelemetry::Context::current_with_span(span);
    
    // Record metrics
    let metrics = metrics::get();
    metrics.api_requests.add(
        1,
        &[
            KeyValue::new("method", method.clone()),
            KeyValue::new("endpoint", matched_path.clone()),
        ],
    );
    
    // Process request with tracing context
    let response = next.run(req).with_context(cx.clone()).await;
    
    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16();
    
    // Update span with response information
    let span = cx.span();
    span.set_attribute(KeyValue::new("http.status_code", status as i64));
    
    if status >= 400 {
        metrics.api_errors.add(
            1,
            &[
                KeyValue::new("method", method.clone()),
                KeyValue::new("endpoint", matched_path.clone()),
                KeyValue::new("status", status as i64),
            ],
        );
        
        if status >= 500 {
            span.set_status(opentelemetry::trace::Status::error("Server error"));
        }
    }
    
    metrics.api_duration.record(
        duration,
        &[
            KeyValue::new("method", method),
            KeyValue::new("endpoint", matched_path),
            KeyValue::new("status", status as i64),
        ],
    );
    
    span.end();
    
    response
}

/// Database query tracing helper
pub async fn trace_db_query<F, T>(query_name: &str, query_fn: F) -> anyhow::Result<T>
where
    F: std::future::Future<Output = anyhow::Result<T>>,
{
    let tracer = global::tracer("secureguard-api");
    let span = tracer
        .span_builder(format!("db.{}", query_name))
        .with_kind(opentelemetry::trace::SpanKind::Client)
        .with_attributes(vec![
            KeyValue::new("db.system", "postgresql"),
            KeyValue::new("db.operation", query_name.to_string()),
        ])
        .start(&tracer);
    
    let cx = opentelemetry::Context::current_with_span(span);
    let start = Instant::now();
    
    let metrics = metrics::get();
    metrics.database_queries.add(
        1,
        &[KeyValue::new("operation", query_name.to_string())],
    );
    
    let result = query_fn.with_context(cx.clone()).await;
    
    let duration = start.elapsed().as_secs_f64();
    metrics.database_query_duration.record(
        duration,
        &[KeyValue::new("operation", query_name.to_string())],
    );
    
    if result.is_err() {
        cx.span().set_status(opentelemetry::trace::Status::error("Query failed"));
    }
    
    cx.span().end();
    
    result
}