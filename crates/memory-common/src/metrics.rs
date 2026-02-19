//! Prometheus metrics support for all services in the Memory Infrastructure platform.
//!
//! This module provides a shared `MemoryMetrics` singleton that tracks:
//! - HTTP request metrics (count, duration, status)
//! - Memory operation metrics (read, write, delete, search)
//! - Extraction latency
//! - Search performance by intent type
//! - Cache hit/miss rates
//! - Queue message processing
//! - Consolidation operations
//! - Conflict detection
//!
//! # Example
//!
//! ```ignore
//! use memory_common::metrics::{METRICS, metrics_handler};
//!
//! // Record an HTTP request
//! METRICS.record_request("GET", "/memories", 200, 0.042);
//!
//! // Record a memory operation
//! METRICS.record_memory_op("write", true);
//!
//! // Get metrics in Prometheus format
//! let handler = metrics_handler();
//! ```

use axum::{http::StatusCode, response::IntoResponse};
use lazy_static::lazy_static;
use prometheus::{Counter, CounterVec, Gauge, Histogram, HistogramVec, Registry, TextEncoder};

// Global metrics registry and collectors.
lazy_static! {
    /// The global MemoryMetrics singleton instance
    pub static ref METRICS: MemoryMetrics = MemoryMetrics::new();
}

/// Core metrics structure holding all Prometheus collectors
///
/// This struct is instantiated as a lazy_static singleton and provides
/// all metrics collection functionality for the platform.
pub struct MemoryMetrics {
    // HTTP metrics
    pub http_requests_total: CounterVec,
    pub http_request_duration_seconds: HistogramVec,

    // Memory operation metrics
    pub memory_operations_total: CounterVec,

    // Extraction metrics
    pub extraction_duration_seconds: Histogram,

    // Active memories gauge
    pub active_memories_gauge: Gauge,

    // Search metrics
    pub search_latency_seconds: HistogramVec,

    // Queue metrics
    pub queue_messages_total: CounterVec,

    // Cache metrics
    pub cache_hits_total: Counter,
    pub cache_misses_total: Counter,

    // Consolidation metrics
    pub consolidation_runs_total: CounterVec,

    // Conflict detection metrics
    pub conflicts_detected_total: CounterVec,

    // Registry for internal use
    registry: Registry,
}

impl MemoryMetrics {
    /// Create a new MemoryMetrics instance with all collectors registered
    fn new() -> Self {
        let registry = Registry::new();

        // HTTP Requests Total
        // Labels: method, path, status
        let http_requests_total = CounterVec::new(
            prometheus::Opts::new("http_requests_total", "Total HTTP requests")
                .namespace("memory")
                .subsystem("http"),
            &["method", "path", "status"],
        )
        .expect("Failed to create http_requests_total");

        // HTTP Request Duration Seconds
        // Labels: method, path
        let http_request_duration_seconds = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds",
            )
            .namespace("memory")
            .subsystem("http")
            .buckets(vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0,
            ]),
            &["method", "path"],
        )
        .expect("Failed to create http_request_duration_seconds");

        // Memory Operations Total
        // Labels: operation (write/read/delete/search), status (success/error)
        let memory_operations_total = CounterVec::new(
            prometheus::Opts::new("memory_operations_total", "Total memory operations")
                .namespace("memory")
                .subsystem("operations"),
            &["operation", "status"],
        )
        .expect("Failed to create memory_operations_total");

        // Extraction Duration Seconds
        let extraction_duration_seconds = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "extraction_duration_seconds",
                "Time spent extracting information in seconds",
            )
            .namespace("memory")
            .subsystem("extraction")
            .buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0]),
        )
        .expect("Failed to create extraction_duration_seconds");

        // Active Memories Gauge
        let active_memories_gauge = Gauge::with_opts(
            prometheus::Opts::new("active_memories_gauge", "Number of active memories")
                .namespace("memory")
                .subsystem("memories"),
        )
        .expect("Failed to create active_memories_gauge");

        // Search Latency Seconds
        // Labels: intent_type
        let search_latency_seconds = HistogramVec::new(
            prometheus::HistogramOpts::new("search_latency_seconds", "Search latency in seconds")
                .namespace("memory")
                .subsystem("search")
                .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["intent_type"],
        )
        .expect("Failed to create search_latency_seconds");

        // Queue Messages Total
        // Labels: subject, status (queued/processed/failed)
        let queue_messages_total = CounterVec::new(
            prometheus::Opts::new("queue_messages_total", "Total queue messages")
                .namespace("memory")
                .subsystem("queue"),
            &["subject", "status"],
        )
        .expect("Failed to create queue_messages_total");

        // Cache Hits Total
        let cache_hits_total = Counter::with_opts(
            prometheus::Opts::new("cache_hits_total", "Total cache hits")
                .namespace("memory")
                .subsystem("cache"),
        )
        .expect("Failed to create cache_hits_total");

        // Cache Misses Total
        let cache_misses_total = Counter::with_opts(
            prometheus::Opts::new("cache_misses_total", "Total cache misses")
                .namespace("memory")
                .subsystem("cache"),
        )
        .expect("Failed to create cache_misses_total");

        // Consolidation Runs Total
        // Labels: status (success/error)
        let consolidation_runs_total = CounterVec::new(
            prometheus::Opts::new("consolidation_runs_total", "Total consolidation runs")
                .namespace("memory")
                .subsystem("consolidation"),
            &["status"],
        )
        .expect("Failed to create consolidation_runs_total");

        // Conflicts Detected Total
        // Labels: conflict_type
        let conflicts_detected_total = CounterVec::new(
            prometheus::Opts::new("conflicts_detected_total", "Total conflicts detected")
                .namespace("memory")
                .subsystem("conflicts"),
            &["conflict_type"],
        )
        .expect("Failed to create conflicts_detected_total");

        // Register all metrics
        registry
            .register(Box::new(http_requests_total.clone()))
            .expect("Failed to register http_requests_total");
        registry
            .register(Box::new(http_request_duration_seconds.clone()))
            .expect("Failed to register http_request_duration_seconds");
        registry
            .register(Box::new(memory_operations_total.clone()))
            .expect("Failed to register memory_operations_total");
        registry
            .register(Box::new(extraction_duration_seconds.clone()))
            .expect("Failed to register extraction_duration_seconds");
        registry
            .register(Box::new(active_memories_gauge.clone()))
            .expect("Failed to register active_memories_gauge");
        registry
            .register(Box::new(search_latency_seconds.clone()))
            .expect("Failed to register search_latency_seconds");
        registry
            .register(Box::new(queue_messages_total.clone()))
            .expect("Failed to register queue_messages_total");
        registry
            .register(Box::new(cache_hits_total.clone()))
            .expect("Failed to register cache_hits_total");
        registry
            .register(Box::new(cache_misses_total.clone()))
            .expect("Failed to register cache_misses_total");
        registry
            .register(Box::new(consolidation_runs_total.clone()))
            .expect("Failed to register consolidation_runs_total");
        registry
            .register(Box::new(conflicts_detected_total.clone()))
            .expect("Failed to register conflicts_detected_total");

        Self {
            http_requests_total,
            http_request_duration_seconds,
            memory_operations_total,
            extraction_duration_seconds,
            active_memories_gauge,
            search_latency_seconds,
            queue_messages_total,
            cache_hits_total,
            cache_misses_total,
            consolidation_runs_total,
            conflicts_detected_total,
            registry,
        }
    }

    /// Get the Prometheus text format output of all registered metrics
    pub fn gather_metrics(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder
            .encode_to_string(&metric_families)
            .unwrap_or_default()
    }

    /// Record an HTTP request
    ///
    /// # Arguments
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `path` - Request path
    /// * `status` - HTTP status code
    /// * `duration` - Request duration in seconds
    pub fn record_request(&self, method: &str, path: &str, status: u16, duration: f64) {
        let status_str = status.to_string();
        self.http_requests_total
            .with_label_values(&[method, path, &status_str])
            .inc();
        self.http_request_duration_seconds
            .with_label_values(&[method, path])
            .observe(duration);
    }

    /// Record a memory operation (read, write, delete, or search)
    ///
    /// # Arguments
    /// * `operation` - Type of operation: "read", "write", "delete", or "search"
    /// * `success` - Whether the operation succeeded
    pub fn record_memory_op(&self, operation: &str, success: bool) {
        let status = if success { "success" } else { "error" };
        self.memory_operations_total
            .with_label_values(&[operation, status])
            .inc();
    }

    /// Record extraction duration
    ///
    /// # Arguments
    /// * `duration` - Time spent extracting in seconds
    pub fn record_extraction(&self, duration: f64) {
        self.extraction_duration_seconds.observe(duration);
    }

    /// Record search latency
    ///
    /// # Arguments
    /// * `intent_type` - Type of intent being searched
    /// * `duration` - Search duration in seconds
    pub fn record_search(&self, intent_type: &str, duration: f64) {
        self.search_latency_seconds
            .with_label_values(&[intent_type])
            .observe(duration);
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits_total.inc();
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses_total.inc();
    }

    /// Record a queue message
    ///
    /// # Arguments
    /// * `subject` - Message subject/topic
    /// * `status` - Message status: "queued", "processed", or "failed"
    pub fn record_queue_message(&self, subject: &str, status: &str) {
        self.queue_messages_total
            .with_label_values(&[subject, status])
            .inc();
    }

    /// Record a consolidation run
    ///
    /// # Arguments
    /// * `success` - Whether consolidation succeeded
    pub fn record_consolidation(&self, success: bool) {
        let status = if success { "success" } else { "error" };
        self.consolidation_runs_total
            .with_label_values(&[status])
            .inc();
    }

    /// Record a detected conflict
    ///
    /// # Arguments
    /// * `conflict_type` - Type of conflict detected
    pub fn record_conflict(&self, conflict_type: &str) {
        self.conflicts_detected_total
            .with_label_values(&[conflict_type])
            .inc();
    }

    /// Set the current number of active memories
    ///
    /// # Arguments
    /// * `count` - Number of active memories
    pub fn set_active_memories(&self, count: u64) {
        self.active_memories_gauge.set(count as f64);
    }

    /// Increment the active memories counter
    pub fn inc_active_memories(&self) {
        self.active_memories_gauge.inc();
    }

    /// Decrement the active memories counter
    pub fn dec_active_memories(&self) {
        self.active_memories_gauge.dec();
    }
}

/// Axum handler that returns Prometheus metrics in text format
///
/// This handler can be mounted at GET /metrics to expose metrics
/// to Prometheus scrapers.
///
/// # Example
///
/// ```ignore
/// use axum::{routing::get, Router};
/// use memory_common::metrics::metrics_handler;
///
/// let app = Router::new()
///     .route("/metrics", get(metrics_handler));
/// ```
pub async fn metrics_handler() -> impl IntoResponse {
    let metrics = METRICS.gather_metrics();
    (StatusCode::OK, metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_singleton() {
        // Verify singleton is accessible
        let metrics1 = &METRICS;
        let metrics2 = &METRICS;
        // Both references should point to the same instance
        assert!(std::ptr::eq(metrics1 as *const _, metrics2 as *const _));
    }

    #[test]
    fn test_record_request() {
        let metrics = MemoryMetrics::new();
        metrics.record_request("GET", "/api/memories", 200, 0.042);

        let output = metrics.gather_metrics();
        assert!(output.contains("http_requests_total"));
        assert!(output.contains("http_request_duration_seconds"));
    }

    #[test]
    fn test_record_memory_op_success() {
        let metrics = MemoryMetrics::new();
        metrics.record_memory_op("write", true);

        let output = metrics.gather_metrics();
        assert!(output.contains("memory_operations_total"));
        assert!(output.contains("operation=\"write\""));
        assert!(output.contains("status=\"success\""));
    }

    #[test]
    fn test_record_memory_op_error() {
        let metrics = MemoryMetrics::new();
        metrics.record_memory_op("read", false);

        let output = metrics.gather_metrics();
        assert!(output.contains("memory_operations_total"));
        assert!(output.contains("operation=\"read\""));
        assert!(output.contains("status=\"error\""));
    }

    #[test]
    fn test_record_extraction() {
        let metrics = MemoryMetrics::new();
        metrics.record_extraction(1.5);

        let output = metrics.gather_metrics();
        assert!(output.contains("extraction_duration_seconds"));
    }

    #[test]
    fn test_record_search() {
        let metrics = MemoryMetrics::new();
        metrics.record_search("user_intent", 0.35);

        let output = metrics.gather_metrics();
        assert!(output.contains("search_latency_seconds"));
        assert!(output.contains("intent_type=\"user_intent\""));
    }

    #[test]
    fn test_cache_metrics() {
        let metrics = MemoryMetrics::new();
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        let output = metrics.gather_metrics();
        assert!(output.contains("cache_hits_total"));
        assert!(output.contains("cache_misses_total"));
    }

    #[test]
    fn test_active_memories_gauge() {
        let metrics = MemoryMetrics::new();
        metrics.set_active_memories(100);

        let output = metrics.gather_metrics();
        assert!(output.contains("active_memories_gauge"));

        metrics.inc_active_memories();
        let output = metrics.gather_metrics();
        assert!(output.contains("active_memories_gauge"));

        metrics.dec_active_memories();
        let output = metrics.gather_metrics();
        assert!(output.contains("active_memories_gauge"));
    }

    #[test]
    fn test_queue_messages() {
        let metrics = MemoryMetrics::new();
        metrics.record_queue_message("memory.created", "queued");
        metrics.record_queue_message("memory.created", "processed");
        metrics.record_queue_message("memory.deleted", "failed");

        let output = metrics.gather_metrics();
        assert!(output.contains("queue_messages_total"));
        assert!(output.contains("subject=\"memory.created\""));
        assert!(output.contains("status=\"processed\""));
    }

    #[test]
    fn test_consolidation() {
        let metrics = MemoryMetrics::new();
        metrics.record_consolidation(true);
        metrics.record_consolidation(false);

        let output = metrics.gather_metrics();
        assert!(output.contains("consolidation_runs_total"));
    }

    #[test]
    fn test_conflicts() {
        let metrics = MemoryMetrics::new();
        metrics.record_conflict("version_conflict");
        metrics.record_conflict("temporal_conflict");

        let output = metrics.gather_metrics();
        assert!(output.contains("conflicts_detected_total"));
        assert!(output.contains("conflict_type=\"version_conflict\""));
    }

    #[test]
    fn test_metrics_output_format() {
        let metrics = MemoryMetrics::new();
        metrics.record_request("POST", "/api/write", 201, 0.123);
        metrics.record_memory_op("write", true);

        let output = metrics.gather_metrics();

        // Verify Prometheus text format characteristics
        assert!(!output.is_empty());
        assert!(output.contains("# HELP"));
        assert!(output.contains("# TYPE"));
    }

    #[test]
    fn test_multiple_operations() {
        let metrics = MemoryMetrics::new();

        // Simulate a realistic sequence of operations
        metrics.record_request("GET", "/api/search", 200, 0.05);
        metrics.record_memory_op("search", true);
        metrics.record_search("user_intent", 0.03);
        metrics.record_cache_hit();

        metrics.record_request("POST", "/api/write", 201, 0.1);
        metrics.record_memory_op("write", true);
        metrics.record_extraction(0.08);
        metrics.record_queue_message("memory.created", "queued");

        metrics.record_request("GET", "/api/retrieve", 500, 0.2);
        metrics.record_memory_op("read", false);

        let output = metrics.gather_metrics();

        // Verify all metrics are present
        assert!(output.contains("http_requests_total"));
        assert!(output.contains("memory_operations_total"));
        assert!(output.contains("search_latency_seconds"));
        assert!(output.contains("cache_hits_total"));
        assert!(output.contains("extraction_duration_seconds"));
        assert!(output.contains("queue_messages_total"));
    }
}
