# Contributing to Knol

Thank you for your interest in contributing to Knol!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/knol.git`
3. Create a feature branch: `git checkout -b feature/my-feature`
4. Install prerequisites: Rust 1.75+, Docker, Docker Compose

## Development Setup

```bash
# Start infrastructure
docker compose up -d

# Build
cargo build --workspace

# Run tests
cargo test --workspace

# Run with logging
RUST_LOG=debug cargo run --bin service-gateway
```

## Pull Request Process

1. Ensure all tests pass: `cargo test --workspace`
2. Run clippy: `cargo clippy --workspace`
3. Format code: `cargo fmt --all`
4. Update documentation if needed
5. Submit a PR with a clear description

## Code Style

- Follow standard Rust conventions
- Use `thiserror` for error types
- Use `tracing` for logging (not `println!`)
- Write unit tests for new functionality
- Keep functions small and focused

## Reporting Issues

Please use GitHub Issues with the following template:
- Description of the issue
- Steps to reproduce
- Expected behavior
- Actual behavior
- Environment (OS, Rust version, etc.)
