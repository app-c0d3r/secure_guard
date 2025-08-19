import { WebTracerProvider } from '@opentelemetry/sdk-trace-web';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http';
import { BatchSpanProcessor } from '@opentelemetry/sdk-trace-base';
import { Resource } from '@opentelemetry/resources';
import { SemanticResourceAttributes } from '@opentelemetry/semantic-conventions';
import { registerInstrumentations } from '@opentelemetry/instrumentation';
import { FetchInstrumentation } from '@opentelemetry/instrumentation-fetch';
import { XMLHttpRequestInstrumentation } from '@opentelemetry/instrumentation-xml-http-request';
import { trace, context, SpanStatusCode } from '@opentelemetry/api';

// Configuration
const OTEL_EXPORTER_OTLP_ENDPOINT = import.meta.env.VITE_OTEL_EXPORTER_OTLP_ENDPOINT || 'http://localhost:4318';
const SERVICE_NAME = 'secureguard-frontend';
const SERVICE_VERSION = '1.0.0';
const ENVIRONMENT = import.meta.env.VITE_ENVIRONMENT || 'development';

let tracerProvider: WebTracerProvider | null = null;

/**
 * Initialize OpenTelemetry for the frontend application
 */
export function initTelemetry(): void {
  // Don't initialize in test environment
  if (import.meta.env.MODE === 'test') {
    return;
  }

  try {
    // Create resource with service information
    const resource = new Resource({
      [SemanticResourceAttributes.SERVICE_NAME]: SERVICE_NAME,
      [SemanticResourceAttributes.SERVICE_VERSION]: SERVICE_VERSION,
      [SemanticResourceAttributes.DEPLOYMENT_ENVIRONMENT]: ENVIRONMENT,
      'browser.user_agent': navigator.userAgent,
      'browser.language': navigator.language,
    });

    // Create tracer provider
    tracerProvider = new WebTracerProvider({
      resource,
    });

    // Configure OTLP exporter
    const exporter = new OTLPTraceExporter({
      url: `${OTEL_EXPORTER_OTLP_ENDPOINT}/v1/traces`,
      headers: {},
    });

    // Add span processor with batching
    tracerProvider.addSpanProcessor(
      new BatchSpanProcessor(exporter, {
        maxQueueSize: 100,
        maxExportBatchSize: 10,
        scheduledDelayMillis: 500,
        exportTimeoutMillis: 30000,
      })
    );

    // Register provider
    tracerProvider.register();

    // Register instrumentations
    registerInstrumentations({
      instrumentations: [
        new FetchInstrumentation({
          propagateTraceHeaderCorsUrls: [
            /^http:\/\/localhost:8080\/.*/,
            /^https:\/\/api\.secureguard\..*/,
          ],
          clearTimingResources: true,
          applyCustomAttributesOnSpan: (span, request, response) => {
            span.setAttribute('http.request.body.size', request.headers.get('content-length') || 0);
            if (response) {
              span.setAttribute('http.response.body.size', response.headers.get('content-length') || 0);
            }
          },
        }),
        new XMLHttpRequestInstrumentation({
          propagateTraceHeaderCorsUrls: [
            /^http:\/\/localhost:8080\/.*/,
            /^https:\/\/api\.secureguard\..*/,
          ],
        }),
      ],
    });

    console.log('OpenTelemetry initialized successfully');
  } catch (error) {
    console.error('Failed to initialize OpenTelemetry:', error);
  }
}

/**
 * Get the global tracer instance
 */
export function getTracer(name: string = 'secureguard-frontend') {
  return trace.getTracer(name, SERVICE_VERSION);
}

/**
 * Create a custom span for manual instrumentation
 */
export function createSpan(name: string, fn: () => Promise<any>) {
  const tracer = getTracer();
  return tracer.startActiveSpan(name, async (span) => {
    try {
      const result = await fn();
      span.setStatus({ code: SpanStatusCode.OK });
      return result;
    } catch (error) {
      span.setStatus({
        code: SpanStatusCode.ERROR,
        message: error instanceof Error ? error.message : 'Unknown error',
      });
      span.recordException(error as Error);
      throw error;
    } finally {
      span.end();
    }
  });
}

/**
 * Add custom attributes to the current span
 */
export function addSpanAttributes(attributes: Record<string, any>) {
  const span = trace.getActiveSpan();
  if (span) {
    Object.entries(attributes).forEach(([key, value]) => {
      span.setAttribute(key, value);
    });
  }
}

/**
 * Record an error in the current span
 */
export function recordError(error: Error, attributes?: Record<string, any>) {
  const span = trace.getActiveSpan();
  if (span) {
    span.recordException(error, attributes);
    span.setStatus({
      code: SpanStatusCode.ERROR,
      message: error.message,
    });
  }
}

/**
 * Create a span for React component lifecycle
 */
export function traceComponent(componentName: string, lifecycle: string) {
  const tracer = getTracer();
  return tracer.startActiveSpan(`${componentName}.${lifecycle}`, (span) => {
    span.setAttribute('component.name', componentName);
    span.setAttribute('component.lifecycle', lifecycle);
    return span;
  });
}

/**
 * Trace API calls with custom attributes
 */
export async function traceApiCall<T>(
  operationName: string,
  apiCall: () => Promise<T>,
  attributes?: Record<string, any>
): Promise<T> {
  return createSpan(`api.${operationName}`, async () => {
    const span = trace.getActiveSpan();
    if (span && attributes) {
      addSpanAttributes(attributes);
    }
    return apiCall();
  });
}

/**
 * Performance monitoring utilities
 */
export const performance = {
  /**
   * Measure and report component render time
   */
  measureRender(componentName: string, startTime: number) {
    const duration = Date.now() - startTime;
    const span = trace.getActiveSpan();
    if (span) {
      span.setAttribute('render.component', componentName);
      span.setAttribute('render.duration_ms', duration);
    }
  },

  /**
   * Measure and report route change time
   */
  measureRouteChange(from: string, to: string, duration: number) {
    const tracer = getTracer();
    const span = tracer.startSpan('route.change');
    span.setAttributes({
      'route.from': from,
      'route.to': to,
      'route.duration_ms': duration,
    });
    span.end();
  },

  /**
   * Report Web Vitals metrics
   */
  reportWebVitals(metric: any) {
    const tracer = getTracer();
    const span = tracer.startSpan(`webvitals.${metric.name}`);
    span.setAttributes({
      'webvitals.name': metric.name,
      'webvitals.value': metric.value,
      'webvitals.rating': metric.rating,
      'webvitals.delta': metric.delta,
      'webvitals.id': metric.id,
    });
    span.end();
  },
};

/**
 * User interaction tracking
 */
export const interactions = {
  /**
   * Track button clicks
   */
  trackClick(buttonName: string, attributes?: Record<string, any>) {
    const tracer = getTracer();
    const span = tracer.startSpan('user.click');
    span.setAttributes({
      'interaction.type': 'click',
      'interaction.target': buttonName,
      ...attributes,
    });
    span.end();
  },

  /**
   * Track form submissions
   */
  trackFormSubmit(formName: string, success: boolean, attributes?: Record<string, any>) {
    const tracer = getTracer();
    const span = tracer.startSpan('user.form_submit');
    span.setAttributes({
      'interaction.type': 'form_submit',
      'interaction.form': formName,
      'interaction.success': success,
      ...attributes,
    });
    span.end();
  },

  /**
   * Track search queries
   */
  trackSearch(query: string, resultCount: number) {
    const tracer = getTracer();
    const span = tracer.startSpan('user.search');
    span.setAttributes({
      'interaction.type': 'search',
      'search.query_length': query.length,
      'search.result_count': resultCount,
    });
    span.end();
  },
};

/**
 * Shutdown telemetry gracefully
 */
export async function shutdownTelemetry(): Promise<void> {
  if (tracerProvider) {
    await tracerProvider.shutdown();
    tracerProvider = null;
  }
}

// Auto-initialize on import if in browser environment
if (typeof window !== 'undefined') {
  initTelemetry();

  // Shutdown on page unload
  window.addEventListener('beforeunload', () => {
    shutdownTelemetry();
  });
}