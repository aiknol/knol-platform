FROM rust:1.77-bookworm AS builder

WORKDIR /app

# Copy OSS crates (needed as dependencies)
COPY ../knol-oss/crates/ /app/oss-crates/
COPY ../knol-oss/Cargo.toml /app/oss-workspace/Cargo.toml

# Copy enterprise workspace
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# Build all enterprise service binaries
RUN cargo build --release \
    --bin admin-service \
    --bin jobs-service \
    --bin billing-service \
    --bin ingest-service

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/admin-service /usr/local/bin/
COPY --from=builder /app/target/release/jobs-service /usr/local/bin/
COPY --from=builder /app/target/release/billing-service /usr/local/bin/
COPY --from=builder /app/target/release/ingest-service /usr/local/bin/
COPY migrations/ /app/migrations/

ENV RUST_LOG=info
EXPOSE 3001 3002 3003 3004

CMD ["admin-service"]
