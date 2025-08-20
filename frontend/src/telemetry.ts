// Simplified telemetry module with minimal dependencies
console.log('Telemetry module loaded - OpenTelemetry disabled for now');

// Stub implementations for backwards compatibility
export function initTelemetry(): void {
  console.log('OpenTelemetry initialization skipped');
}

export function getTracer(_name: string = 'secureguard-frontend') {
  return {
    startActiveSpan: (_name: string, fn: any) => {
      return fn({
        setStatus: () => {},
        recordException: () => {},
        end: () => {},
        setAttribute: () => {},
        setAttributes: () => {}
      });
    },
    startSpan: (_name: string) => ({
      setStatus: () => {},
      recordException: () => {},
      end: () => {},
      setAttribute: () => {},
      setAttributes: () => {}
    })
  };
}

export function createSpan(_name: string, fn: () => Promise<any>) {
  return fn();
}

export function addSpanAttributes(_attributes: Record<string, any>) {
  // No-op
}

export function recordError(error: Error, _attributes?: Record<string, any>) {
  console.error('Error recorded:', error.message);
}

export function traceComponent(_componentName: string, _lifecycle: string) {
  return {
    setAttribute: () => {},
    end: () => {}
  };
}

export async function traceApiCall<T>(
  _operationName: string,
  apiCall: () => Promise<T>,
  _attributes?: Record<string, any>
): Promise<T> {
  return apiCall();
}

export const performance = {
  measureRender(componentName: string, startTime: number) {
    const duration = Date.now() - startTime;
    console.debug(`Render time for ${componentName}: ${duration}ms`);
  },

  measureRouteChange(from: string, to: string, duration: number) {
    console.debug(`Route change from ${from} to ${to}: ${duration}ms`);
  },

  reportWebVitals(metric: any) {
    console.debug('Web Vitals:', metric);
  },
};

export const interactions = {
  trackClick(buttonName: string, _attributes?: Record<string, any>) {
    console.debug(`Button clicked: ${buttonName}`);
  },

  trackFormSubmit(formName: string, success: boolean, _attributes?: Record<string, any>) {
    console.debug(`Form submitted: ${formName}, success: ${success}`);
  },

  trackSearch(query: string, resultCount: number) {
    console.debug(`Search performed: query length ${query.length}, results: ${resultCount}`);
  },
};

export async function shutdownTelemetry(): Promise<void> {
  console.log('Telemetry shutdown');
}

// Auto-initialize on import if in browser environment
if (typeof window !== 'undefined') {
  initTelemetry();

  // Shutdown on page unload
  window.addEventListener('beforeunload', () => {
    shutdownTelemetry();
  });
}