//! Opentelemetry request tracking & metrics middleware.

#[cfg(feature = "otel-metrics")]
pub mod metrics;
#[cfg(feature = "otel-tracing")]
pub mod tracing;

pub(self) const HTTP_HOST: opentelemetry::Key = opentelemetry::Key::from_static_str("http.host");
