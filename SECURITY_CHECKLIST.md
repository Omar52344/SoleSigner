# Security Checklist for Production Deployment

## Before Going Live

### 1. Environment & Configuration
- [ ] Change `JWT_SECRET` to a strong, random string (min 32 chars)
- [ ] Use strong, unique passwords for database users
- [ ] Ensure `DATABASE_URL` does not contain credentials in logs
- [ ] Set `BCRYPT_COST` to at least 12 (higher for more security)
- [ ] Disable debug logging in production (`RUST_LOG=info`)
- [ ] Enable JSON logs if using log aggregation (`JSON_LOGS=true`)

### 2. Database Security
- [ ] PostgreSQL is configured with SSL/TLS (or use a cloud provider with TLS)
- [ ] Database firewall limits connections to the application server only
- [ ] Regular backups are scheduled (see `scripts/backup.sh`)
- [ ] The database user has minimal required privileges (no superuser)

### 3. Network & Infrastructure
- [ ] Application runs behind a reverse proxy (Nginx, Caddy, cloud load balancer)
- [ ] HTTPS is enforced (redirect HTTP to HTTPS)
- [ ] CORS origins are restricted to known frontend domains
- [ ] Rate limiting is implemented (not yet in code – consider adding)
- [ ] Server OS and dependencies are up to date

### 4. Application Code
- [ ] All dependencies are up to date (`cargo audit` passes)
- [ ] No sensitive data (secrets, keys) is committed to Git
- [ ] Error messages do not leak stack traces or internal details
- [ ] Input validation is applied on all user‑provided data
- [ ] SQL injection is prevented by using parameterized queries (sqlx)
- [ ] Authentication is required for admin endpoints (JWT verification)
- [ ] Nullifier uniqueness prevents double‑voting
- [ ] Election status and date checks prevent voting outside the allowed window
- [ ] Private elections enforce whitelist checks

### 5. Vote Integrity
- [ ] Each vote produces a cryptographically signed receipt
- [ ] Merkle tree root is computed when election is sealed
- [ ] Receipts can be independently verified via `/vote/verify‑receipt`
- [ ] Ballot secrecy is preserved (choices are encrypted in the database)
- [ ] Nullifier derivation uses a per‑election salt

### 6. Monitoring & Logging
- [ ] Logs are collected and monitored for anomalies
- [ ] Metrics for request counts, error rates, and vote volume are tracked
- [ ] Alerts are set up for critical errors (database down, 5xx responses)
- [ ] Logs do not contain sensitive data (document numbers, passwords, JWT tokens)

### 7. Operational Security
- [ ] Access to production servers is restricted (SSH keys, minimal personnel)
- [ ] Secrets are managed via environment variables or a secret manager
- [ ] Regular security updates are applied to the OS and runtime
- [ ] Incident response plan is documented

## Post‑Deployment Tests

- [ ] Run the stress test (`cargo run --bin stress`) to verify concurrency handling
- [ ] Attempt SQL injection attacks on all text inputs (should be blocked by validation)
- [ ] Try to vote twice with the same document number (should be rejected)
- [ ] Verify that a closed election rejects new votes
- [ ] Confirm that private elections block non‑whitelisted documents
- [ ] Check that admin endpoints require a valid JWT token
- [ ] Validate that receipts can be verified using the public endpoint

## Ongoing Maintenance

- [ ] Regularly rotate JWT secret (requires all existing tokens to expire)
- [ ] Monitor database disk space and performance
- [ ] Review logs for suspicious patterns (e.g., many failed login attempts)
- [ ] Keep dependencies updated and re‑run `cargo audit`
- [ ] Periodically test backup restoration

## Emergency Procedures

- **Suspected breach**: Rotate all secrets (JWT, database passwords), audit logs, review recent votes.
- **Database corruption**: Restore from latest backup, verify Merkle roots match.
- **DDoS / high load**: Enable rate limiting, scale horizontally if possible.

---

*This checklist is a starting point. Adapt it to your specific deployment environment and compliance requirements.*