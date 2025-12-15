# Path to Production

This document outlines the necessary steps to make this project production-ready, based on an initial analysis.

## Roadmap

### 1. Critical Bug Fixes & Security Vulnerabilities
- [ ] **Fix Redis Connection Bug:** The `auth-service` will fail in production because it cannot connect to Redis. The `REDIS_HOST_NAME` environment variable must be set to `redis` in `compose.yml`.
- [ ] **Secure Deployment Credentials:** The CI/CD pipeline uses `sshpass` with a plaintext password. This is a critical security risk and must be replaced with SSH key-based authentication.
- [ ] **Add Web Security Headers:** The Caddy reverse proxy configuration is missing essential security headers like Content-Security-Policy (CSP), HSTS, and X-Frame-Options, leaving the application vulnerable.

### 2. Deployment & Reliability
- [ ] **Implement Zero-Downtime Deployments:** The current deployment process causes service downtime. This should be replaced with a blue-green or rolling update strategy.
- [ ] **Add Service Health Checks:** The services in `compose.yml` lack health checks. Adding them will ensure that the application only serves traffic when the containers are fully operational, preventing outages from bad deploys.
- [ ] **Improve Database Readiness Checks:** The `depends_on` condition for the database is not sufficient. Implement a mechanism (e.g., a script or tool like `wait-for-it.sh`) to ensure the database is fully ready to accept connections before the `auth-service` starts.

### 3. Configuration & Code Quality
- [ ] **Make Hashing Parameters Configurable:** The Argon2 password hashing parameters in the `auth-service` are hardcoded. These should be loaded from configuration to allow for easier updates and security tuning.
- [ ] **Externalize Secrets:** Sensitive information like JWT secrets and API tokens are currently managed via environment variables and configuration files. These should be moved to a dedicated secrets management solution (e.g., HashiCorp Vault, AWS/GCP Secret Manager) for better security and auditing.
