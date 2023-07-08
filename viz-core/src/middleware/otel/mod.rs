//! Opentelemetry request tracking & metrics middleware.

pub(self) const HTTP_HOST: opentelemetry::Key = opentelemetry::Key::from_static_str("http.host");

#[cfg(feature = "otel-metrics")]
pub mod metrics;
#[cfg(feature = "otel-tracing")]
pub mod tracing;
