//! Opentelemetry request tracking & metrics middleware.

/// `http.host` key
pub const HTTP_HOST: opentelemetry::Key = opentelemetry::Key::from_static_str("http.host");

#[cfg(feature = "otel-metrics")]
pub mod metrics;
#[cfg(feature = "otel-tracing")]
pub mod tracing;
