/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_OTEL_EXPORTER_OTLP_ENDPOINT?: string
  readonly VITE_ENVIRONMENT?: string
  readonly VITE_DISABLE_OTEL?: string
  readonly MODE: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}