# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SecureGuard is a cloud-native cybersecurity platform with agent-based endpoint protection, implemented as a Rust workspace with microservices architecture. The system combines real-time threat detection with centralized management and GDPR-compliant data handling.

## Development Commands

### Build Commands
```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build -p secureguard-api
cargo build -p secureguard-agent  
cargo build -p secureguard-shared

# Release build
cargo build --release
```

### Testing Commands
```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p secureguard-api

# Run tests with output
cargo test -- --nocapture

# Property-based testing (when implemented)
cargo test --features property-tests
```

### Code Quality Commands
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run clippy lints
cargo clippy

# Run clippy with all warnings
cargo clippy -- -W clippy::all -W clippy::nursery -W clippy::cargo -W clippy::pedantic

# Check for unsafe code (should fail due to workspace config)
cargo clippy -- -D unsafe-code
```

### Database Commands
```bash
# Start development database
docker-compose up -d

# Run migrations (requires sqlx-cli)
sqlx migrate run

# Create new migration
sqlx migrate add <migration_name>
```

## Architecture Overview

### Workspace Structure
- `secureguard-api`: REST API server using Axum framework
- `secureguard-agent`: Lightweight endpoint agent for threat detection
- `secureguard-shared`: Common types, utilities, and domain models
- `secureguard-cli`: Command-line interface (planned)

### Key Technologies
- **Runtime**: Rust 1.75+ with async/await
- **Web Framework**: Axum 0.7+ for high-performance API
- **Database**: PostgreSQL 15+ with schema-based multi-tenancy
- **Cache/Queue**: Redis 7+ for caching and message streams
- **Serialization**: MessagePack (rmp-serde) for agent communication
- **Security**: AES-256-GCM encryption, Argon2 password hashing

### Database Schema Organization
- `users`: User management and authentication
- `agents`: Endpoint agent registration and status
- `tenants`: Multi-tenant organization structure
- `threats`: Security events with time-based partitioning

### Communication Model
- **Heartbeat Service**: Regular agent status updates (pull-based)
- **WebSocket Connection**: Real-time command & control (push-based)
- **Hybrid Approach**: Combines reliability with real-time responsiveness

## Development Standards

### Performance Requirements
- API response times: < 100ms
- Agent CPU usage: < 1%
- Agent RAM usage: < 50MB
- System uptime: 99.9% target

### Security Requirements
- Memory-safe Rust code (unsafe code denied in workspace config)
- Defense-in-depth architecture with Zero-Trust model
- JWT-based authentication with RBAC authorization
- GDPR compliance with built-in data retention policies

### Code Style
- Use `rustfmt` for consistent formatting
- Enable all clippy warning categories (all, nursery, cargo, pedantic)
- Follow Rust naming conventions: PascalCase for types, snake_case for functions
- Use `thiserror` for custom errors, `anyhow` for error context

### Testing Approach
- Test-Driven Development (TDD) methodology
- Unit tests for core logic
- Integration tests for API and database interactions
- Property-based testing for comprehensive input validation

## Infrastructure

### Local Development
```bash
# Start required services
docker-compose up -d

# Verify services
docker-compose ps
```

### Environment Configuration
- Development database: PostgreSQL on port 5432
- Redis cache: Redis on port 6379
- Default credentials in docker-compose.yml (development only)

## Key Implementation Notes

- Hardware fingerprinting prevents duplicate agent registrations
- Time-partitioned security events table for efficient querying
- Two-tier caching strategy (L1 in-memory, L2 Redis distributed)
- Database migrations managed with sqlx-cli
- All database queries use parameterized builders to prevent SQL injection


* **Proactive Clarification:** 
If any part of the user's request, project architecture, or desired logic is unclear, 
immediately ask for clarification and wait for a response before proceeding.
* **Maintain Project Context:** 
Maintain an internal model of the entire application, 
including its business logic, architecture, and current roadmap. 
Review `next_steps.md` and `roadmap.md` at the start in root folder "docs" of each major task to ensure your work is aligned.
if the files are not there so plan and create them and inform and provide the result to the user.
* **Self-Correction & Critique:** Before finalizing any output, perform a self-critique. 
Review your proposed solution for potential bugs, logical inconsistencies, or alignment with project best practices.
* **Leverage Sub-Agents:** For complex tasks, create a specialized team of "sub-agents." 
For example, a "Testing Agent" to focus solely on test coverage, a "Coding Agent" for implementation, and a "Documentation Agent" 
for updating relevant files upon task completion.
* **Update Documentation:** 
Upon successful implementation and verification of a new feature, draft an update to the 
relevant documentation files (e.g., `README.md`, `API-docs.md`).
* **Parallel Tooling:** 
When multiple independent operations are required, invoke all relevant tools simultaneously to maximize efficiency.

### 1. Core Principles
Follow these rules at all times:
    * Plan First: Before coding, create a detailed, step-by-step plan. Wait for user approval.
    * Embrace TDD: Implement a Test-Driven Development workflow for verifiable changes.
    * Think Deeply: For complex tasks, use extended thinking to analyze the problem fully before acting.
    * Be Skeptical: Validate your work. Review your code and tests to ensure they meet all project standards.

### 2. Coding & Implementation Standards
    * Write clean, simple, composable functions. Avoid over-engineering.
    * Use descriptive names that match existing project vocabulary.
    * Add comments only for non-obvious logic or critical caveats.

### 3. Testing Rules
    * Colocate tests with source files (*.spec.ts) for unit tests.
    * Separate pure-logic unit tests from database-touching integration tests.
    * Prefer integration tests over excessive mocking.
    * Test entire structures in one assertion when possible.
    * Parameterize inputs instead of embedding literal values in tests.

### 4. Documentation Standards
    * use always "docs" folder for documentations files
    * never write secrets or passwords or apikey hardcoded in a file, use best practices and if require for testing it localy so use separate file and mark as gitignore
    * "Lastenheft.md": Produktkonzept
    * "SecureGuard Technical & Implementation Guide.md": system Overview & Core Principles, Technology Stack, Agent Architecture, Backend Architecture, Security & Compliance, Performance & Scalability, Testing & Quality Assurance, Implementation
    * README.md (The Front Door): A living document providing a high-level overview, setup instructions, and how to run the project. It must be complete before the first commit and updated with every major feature.
    * System Architecture: A high-level document or diagram explaining the system's components and their relationships. Create a draft during the planning phase and update it with major architectural changes.
    * API Documentation: The formal contract for all API endpoints. Document each endpoint's method, path, and request/response schema. Create this documentation before writing the code.
    * Code Comments & Docstrings: In-line documentation explaining the "what" and "why" of functions and complex logic. This should be an ongoing part of the coding process.
    * Decision Log: A record of key architectural and business decisions, including the problem, alternatives considered, and the final rationale. Document every non-trivial decision.
    * The Roadmap (roadmap.md) is a high-level, future-looking document. It outlines the major features or product milestones you plan to hit over months or even a year. It defines the "what" and "why" of your business's direction.
    * The Next Steps (next_steps.md) document is for the immediate, actionable tasks. It's a short, prioritized list of the next things that need to be done to achieve the current milestone. It defines the "how" and "now" of your work.