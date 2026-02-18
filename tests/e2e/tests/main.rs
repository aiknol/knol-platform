#![allow(dead_code, unused_comparisons)]
// =============================================================================
// Knol — End-to-End Test Suite
// =============================================================================
//
// 100% coverage of all 8 services, 39 HTTP endpoints, 1 NATS consumer,
// 6 scheduled jobs, and cross-service integration flows.
//
// Run:
//   cargo test --manifest-path tests/e2e/Cargo.toml
//
// Requires:
//   - All 8 services running (via docker compose)
//   - Postgres with migrations applied
//   - Redis running
//   - NATS running
//   - A test tenant with valid API key
//
// Environment:
//   GATEWAY_URL    = http://localhost:8080  (default)
//   WRITE_URL      = http://localhost:8081
//   RETRIEVE_URL   = http://localhost:8082
//   ADMIN_URL      = http://localhost:8084
//   JOBS_URL       = http://localhost:8085
//   BILLING_URL    = http://localhost:8086
//   INGEST_URL     = http://localhost:8087
//   TEST_API_KEY   = test-api-key-for-e2e
//   TEST_TENANT_ID = <uuid of test tenant>
// =============================================================================

mod harness;
mod test_gateway;
mod test_write;
mod test_retrieve;
mod test_graph;
mod test_admin;
mod test_jobs;
mod test_billing;
mod test_ingest;
mod test_cross_service;
mod test_deploy;
