# Roadmap

This document tracks the planned evolution of QXScan. The OSS releases follow
[Semantic Versioning](https://semver.org/): `0.x` means the API may evolve.

---

## v0.1.0 (completed)

- [x] Single-target scan (`qxscan scan example.com`)
- [x] Multi-standard compliance (PCI-DSS, HIPAA, SOC2, FISMA, PQC)
- [x] Structured reports — terminal, JSON, HTML
- [x] QEM (QX Event Model) — stable, versioned scan schema
- [x] Scheduler — periodic scans with cron expressions
- [x] Daemon (`qxscan server start`) — background service with SQLite state
- [x] Export — Prometheus, OCSF, CEF formats
- [x] Demo stack — Docker Compose with PQC-ready services
- [x] Test suite — unit tests, Docker integration, content assertions, classification
- [x] CI/CD — GitHub Actions build + assertion pipeline
- [x] Single binary under 10 MB — static linking, stripped

---

## v0.2.0 (planned)

- [ ] Enhanced PQC detection — deeper algorithm inspection, hybrid key exchange details
- [ ] STARTTLS improvements — protocol-aware negotiation for SMTP, IMAP, POP3, LDAP
- [ ] Additional compliance controls — finer-grained checks per standard
- [ ] More protocol coverage — MySQL, FTP, DNS-over-TLS, RDP
- [ ] Scanner provenance metadata — version, build, platform in every QEM event
- [ ] Plugin architecture for custom compliance standards
- [ ] Command-line tab completion (bash, zsh, fish)

---

## v0.3.0 (planned, tentative)

- [ ] Batch scan performance — parallel target probing
- [ ] HTTP/3 (QUIC) support
- [ ] Certificate chain validation improvements
- [ ] Differential reports — compare two scan runs
- [ ] Webhook notifications on scan completion

---

## Enterprise (separate product)

These capabilities are planned for the QuantX Enterprise platform,
built on top of QXScan OSS:

| Capability | Description |
|-----------|-------------|
| Fleet management | Manage thousands of scanners from a central console |
| Dashboard | Real-time compliance posture across all assets |
| RBAC | Role-based access control with audit logging |
| Observability | Metrics, traces, and logs for the scanning infrastructure |
| REST API | Programmatic access to scan results and configuration |
| Policies | Declarative compliance policies enforced across the fleet |
| Integrations | Native connectors for Splunk, Elastic, Datadog, ServiceNow |

---

## Non-goals (will not add to OSS)

These are explicitly out of scope for the open-source project:

- REST API
- Web UI / dashboard
- RBAC / multi-tenancy
- Fleet management
- Elastic / Splunk / Datadog connectors
- Cloud-hosted scanning

These belong in the Enterprise product line.

---

*Last updated: July 2026*
