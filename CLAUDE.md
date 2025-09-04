# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Architecture

This is a live bootcamp project implementing a multi-service authentication system using Rust and Axum. The project consists of two main services:

### Services Overview
- **auth-service** (port 3000): Authentication service handling user registration, login, 2FA, JWT token management, and account deletion
- **app-service** (port 8000): Frontend application service that consumes the auth service

### Authentication Service Architecture
The auth-service follows a layered architecture pattern:

- **Domain Layer** (`src/domain/`): Core business entities and validation logic
  - User management, email/password validation, authentication errors
  - Data store traits for user and token storage abstractions
- **Services Layer** (`src/services/`): Business logic implementations  
  - In-memory HashMap user store and HashSet banned token store
- **Routes Layer** (`src/routes/`): HTTP endpoint handlers
  - REST API endpoints for signup, login, logout, verify-2fa, verify-token, delete-account
- **Utils Layer** (`src/utils/`): Cross-cutting concerns
  - JWT token management, configuration handling, CORS setup, authentication utilities

### Key Architectural Patterns
- **Dependency Injection**: AppState pattern for sharing services across routes
- **Configuration Management**: Dynamic configuration reload with file watching (`utils/config.rs`)
- **Dynamic CORS**: Environment-based CORS configuration supporting multiple origins
- **Cookie-based Authentication**: JWT tokens stored in secure HTTP-only cookies

## Development Commands

### Building Services
```bash
# Build auth-service
cd auth-service && cargo build

# Build app-service  
cd app-service && cargo build
```

### Running Services Locally (Manual)
```bash
# Install cargo-watch for hot reloading
cargo install cargo-watch

# Run auth-service with hot reload
cd auth-service
cargo watch -q -c -w src/ -w assets/ -x run

# Run app-service with hot reload (in separate terminal)
cd app-service  
cargo watch -q -c -w src/ -w assets/ -w templates/ -x run
```

### Running with Docker
```bash
# Build and run all services
docker compose build
docker compose up
```

## Service Dependencies and Environment

### Required Environment Variables
- **AUTH_SERVICE_ALLOWED_ORIGINS**: Comma-separated list of allowed CORS origins for auth-service
- **JWT_SECRET**: Secret key for JWT token signing
- **AUTH_SERVICE_IP**: IP address for auth-service (defaults to localhost)
- **AUTH_SERVICE_URL**: Full URL for auth-service (defaults to http://localhost:3000)

### Service Communication
- App-service communicates with auth-service via HTTP requests
- Authentication state is maintained through JWT tokens in cookies
- Services are orchestrated using Docker Compose with Caddy as reverse proxy

### Docker Architecture
- **auth-service**: Exposed on port 3000 internally
- **app-service**: Exposed on port 8000 internally  
- **caddy**: Acts as reverse proxy handling external traffic on ports 80/443
- **Network**: All services communicate through `backend` Docker network

## Testing and Development

### Hot Reloading
Both services use `cargo watch` for automatic rebuilds on file changes:
- auth-service watches `src/` and `assets/` directories
- app-service watches `src/`, `assets/`, and `templates/` directories

### Service Endpoints
- **auth-service**: http://localhost:3000
- **app-service**: http://localhost:8000 (main application interface)

### Key Files for Development
- Configuration management: `auth-service/src/utils/config.rs`
- CORS setup: `auth-service/src/utils/dynamic_cors.rs`
- JWT handling: `auth-service/src/utils/auth.rs`
- Main application state: `auth-service/src/app_state.rs`