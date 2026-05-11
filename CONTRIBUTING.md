# Contributing to Knol

Thank you for your interest in contributing to Knol!

## License

- Contributions to `knol-oss/`, `frontend/`, `deploy/`, `tests/`, and `scripts/` are licensed under [Apache 2.0](LICENSE).
- Contributions to `knol-enterprise/` are licensed under the [Knol Enterprise License](knol-enterprise/LICENSE).

By submitting a pull request, you agree to license your contribution under the applicable license.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/knol-platform.git`
3. Create a feature branch: `git checkout -b feature/my-feature`
4. Install prerequisites: Rust 1.77+, Node.js 20+, Docker, Docker Compose

## Development Setup

```bash
# Start infrastructure (PostgreSQL, NATS, Redis, MinIO)
docker compose -f docker-compose.oss.yml up -d

# Copy environment config
cp .env.example .env

# Build OSS services
cd knol-oss && cargo build --workspace

# Build Enterprise services
cd knol-enterprise && cargo build --workspace

# Run tests
cd knol-oss && cargo test --workspace --lib
cd knol-enterprise && cargo test --workspace --lib

# Run frontend
cd frontend && npm install && npm run dev:web
```

## Pull Request Process

1. Ensure all tests pass: `cargo test --workspace --lib` in both `knol-oss/` and `knol-enterprise/`
2. Run clippy: `cargo clippy --workspace -- -D warnings`
3. Format code: `cargo fmt --all`
4. Update documentation if needed
5. Submit a PR with a clear description of the change

## Code Style

- Follow standard Rust conventions
- Use `thiserror` for error types
- Use `tracing` for logging (not `println!`)
- Write unit tests for new functionality
- Keep functions small and focused
- Run `cargo fmt` before committing

## Pre-Push Checks

Install the git hooks to run local CI before push:

```bash
./scripts/install-git-hooks.sh
```

This runs format checks and clippy on both workspaces before allowing a push.

## Reporting Issues

Please use GitHub Issues with:
- Description of the issue
- Steps to reproduce
- Expected vs. actual behavior
- Environment (OS, Rust version, etc.)

## Security

If you discover a security vulnerability, please report it responsibly. See [SECURITY.md](knol-oss/SECURITY.md) for details.
