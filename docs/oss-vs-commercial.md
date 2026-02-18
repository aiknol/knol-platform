# Knol OSS vs Commercial Boundary

This document defines what stays open source and what is monetized in managed and enterprise offerings.

## Strategy Summary

Knol follows an open-core model:

- Keep core memory infrastructure open for trust, adoption, and developer-led growth.
- Monetize managed operations and enterprise risk reduction.
- Avoid paywalling capabilities developers expect in a modern memory stack.

## Capability Matrix

| Area | OSS (Self-host) | Cloud / Enterprise (Paid) |
|---|---|---|
| Core APIs | Memory write/retrieve APIs, graph APIs, SDKs | Higher throughput tiers, premium SLO-backed endpoints |
| Core services | `gateway`, `write-service`, `retrieve-service`, `graph-service` | Hosted control plane, managed upgrades, autoscaling controls |
| Data model | Vector + BM25 + graph primitives, memory/episode/entity/edge schema | Managed backup/restore, PITR, DR guarantees, multi-region |
| Security baseline | Tenant model, RLS primitives, baseline auth integration points | SSO/SAML/SCIM, enterprise RBAC, policy packs |
| Admin tooling | Base config, credential management, basic audit views | Governance workflows, approvals, audit export connectors |
| Background jobs | Consolidation/conflict logic in code | Managed scheduling, job SLOs, replay tooling |
| Deployability | Docker Compose/Kubernetes self-host | Fully managed onboarding, migration support, TAM |
| Support | Community support | Priority support, dedicated channels, SLA commitments |

## Service Mapping

### OSS

- `knol-gateway-1`
- `knol-write-service-1`
- `knol-retrieve-service-1`
- `knol-graph-service-1`
- SDKs and public docs
- Core schema and migrations

### Commercial

- `knol-admin-service-1` as managed governance plane
- `knol-jobs-service-1` as managed reliability/quality plane
- `knol-billing-service-1` for commercial metering and enforcement
- Hosted ops stack for SLA, compliance, and support

## Product Packaging

### OSS (Free)

- Full self-hosted core platform
- Hybrid retrieval primitives
- Base admin controls
- Community support

### Builder (Cloud)

- Managed core services
- Predictable usage envelopes and overages
- Email support
- Basic observability

### Growth (Cloud)

- Higher limits and throughput
- Managed consolidation/conflict jobs
- Admin dashboard and advanced audit retention
- Priority support + uptime SLA

### Enterprise

- SSO/SAML/SCIM
- Compliance package and audit exports
- Dedicated VPC or BYOC options
- Contractual SLA and named support

## Anti-Pattern to Avoid

Do not paywall core retrieval quality primitives:

- Vector retrieval
- BM25/text retrieval
- Graph retrieval
- Core fusion/routing primitives

Keep paid moat in:

- Reliability guarantees
- Compliance and governance
- Enterprise identity and controls
- Operational convenience and support
