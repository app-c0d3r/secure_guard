# SecureGuard

A cloud-native cybersecurity platform with lightweight agent-based threat detection and real-time monitoring capabilities.

## 🚀 Current Status: Phase 1 - Core Foundation (75% Complete)

### ✅ Implemented Features
- **Backend API**: Rust + Axum REST server
- **Authentication**: JWT tokens with Argon2 password hashing
- **Agent Management**: Registration, heartbeat tracking, status monitoring
- **Database**: PostgreSQL with SQLx integration
- **Security**: Multi-layer authentication and input validation

### 🔧 Quick Start

#### Prerequisites
- Windows with Visual Studio C++ Build Tools
- Rust 1.75+ (✅ 1.89.0 installed)
- PostgreSQL 15+
- Docker (optional)

#### Installation
1. **Setup Build Tools** (Windows required):
   ```bash
   # Download and install Visual Studio Build Tools 2022
   # Select "C++ build tools" workload
   ```

2. **Database Setup**:
   ```bash
   docker-compose up -d
   # or install PostgreSQL locally
   ```

3. **Build & Run**:
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   sqlx migrate run
   cargo run -p secureguard-api
   ```

4. **Test API**:
   ```bash
   curl http://localhost:3000/health
   ```

## 📁 Project Structure

```
secure_guard/
├── crates/
│   ├── secureguard-api/     # Main API server
│   │   ├── src/
│   │   │   ├── handlers/    # HTTP request handlers  
│   │   │   ├── services/    # Business logic layer
│   │   │   ├── middleware/  # Authentication middleware
│   │   │   └── routes.rs    # API route definitions
│   │   └── Cargo.toml
│   └── secureguard-shared/  # Shared types and models
│       ├── src/
│       │   ├── models.rs    # Data models
│       │   └── errors.rs    # Error types
│       └── Cargo.toml
├── migrations/              # Database migrations
├── docs/                   # Documentation
└── docker-compose.yml     # Development services
```

## 🔌 API Endpoints

### Authentication
- `POST /api/v1/auth/register` - User registration
- `POST /api/v1/auth/login` - User login
- `GET /api/v1/auth/me` - Get current user

### Agent Management  
- `POST /api/v1/agents/register` - Register new agent
- `POST /api/v1/agents/heartbeat` - Agent status update
- `GET /api/v1/agents` - List agents

### System
- `GET /health` - Health check

## 🛠 Development

### Development Setup
```bash
# Install development tools
cargo install cargo-watch sqlx-cli cargo-audit

# Start with auto-reload
cargo watch -x "run -p secureguard-api"

# Code quality checks
cargo fmt && cargo clippy -- -D warnings
```

### Environment Variables
```bash
DATABASE_URL=postgresql://postgres:password@localhost:5432/secureguard
REDIS_URL=redis://localhost:6379  
JWT_SECRET=your-secret-key-change-in-production
PORT=3000
```

## 🏗 Architecture

### Core Principles
- **Security by Design**: Multi-layer authentication and validation
- **Performance First**: Sub-100ms API responses, <50MB RAM per agent
- **Scalability**: Horizontally scalable to 10,000+ agents  
- **Privacy Compliance**: GDPR-compliant data processing
- **KISS Principle**: Simple, maintainable, resource-efficient

### Technology Stack
- **Backend**: Rust 1.75+ with Axum web framework
- **Database**: PostgreSQL 15+ with SQLx
- **Cache**: Redis 7+ for session management
- **Authentication**: JWT with Argon2 password hashing
- **Security**: TLS, CORS, input validation, parameterized queries

## 📋 Next Steps

### Immediate Priorities
1. **Complete Phase 1**:
   - Install Visual Studio C++ Build Tools  
   - Set up PostgreSQL database
   - Implement comprehensive testing
   - Finalize development pipeline

2. **Phase 2 - MVP Features** (Months 4-6):
   - Threat Detection Engine
   - React Dashboard
   - WebSocket real-time communication
   - Event processing and alerting

## 🔒 Security Features

- **Zero-Trust Architecture**: All requests authenticated and authorized
- **Defense-in-Depth**: Multiple security layers
- **Secure by Default**: Safe defaults, explicit security configurations  
- **Privacy First**: Minimal data collection, GDPR compliance
- **Audit Ready**: Comprehensive logging and monitoring

## 📖 Documentation

All project documentation is organized in the [`docs/`](docs/) directory:

- **[📋 Documentation Index](docs/README.md)** - Complete documentation overview
- **[🚀 Setup Guide](docs/Development_Setup_Guide.md)** - Development environment setup
- **[📊 Implementation Status](docs/Implementation_Status.md)** - Current progress tracking
- **[🏗 Technical Architecture](docs/en_SecureGuard%20Technical%20&%20Implementation%20Guide.md)** - System design
- **[🗺 Roadmap](docs/Roadmap.md)** - Development phases and milestones
- **[⚡ Next Steps](docs/NEXT_STEPS.md)** - Immediate priorities

## 🤝 Contributing

1. Follow Rust style guidelines (`cargo fmt`)
2. Ensure all tests pass (`cargo test`)
3. Run clippy linter (`cargo clippy`)  
4. Security audit dependencies (`cargo audit`)

## 📄 License

This project is designed for defensive cybersecurity purposes only.

---

**Status**: Ready for Phase 1 completion and Phase 2 development ✅