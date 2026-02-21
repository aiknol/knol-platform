# Knol Docs

## Core

- `docker-stack.md`: Local Docker usage with separated OSS/proprietary compose files.
- `oss-vs-commercial.md`: Open-source vs paid boundary, packaging model, and service mapping.
- `memory-as-a-service-blueprint.md`: Technical architecture and implementation blueprint.
- `memory-as-a-service-business-strategy.md`: Product and GTM strategy.
- `automated-marketing-strategy.md`: Compliance-first automation strategy for multi-channel posting, rate-limit-safe scheduling, and full-funnel marketing workflows.
- `ARCHITECTURE.html`: Platform architecture overview.
- `COMPETITIVE_STRATEGY.html`: Competitive strategy analysis.
- `Knol-Deployment-Guide.html`: Deployment guide.
- `Knol-Zero-Cost-Marketing-Plan.md`: Marketing execution plan.
- `knol-docs/`: Interactive static docs bundle (`architecture.html`, `business.html`, `dataflow.html`, `deployment.html`, `technical.html`).

## Notes

- Product name is **Knol**.
- Some long-form architecture/strategy docs preserve historical `memory-*` service naming for conceptual context.
- Runtime separation:
  - OSS: `docker-compose.oss.yml`
  - Proprietary overlay: `docker-compose.proprietary.yml`
