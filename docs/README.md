# Documentation

> **QXScan — On-Prem Security & Compliance Scanner**

---

## Quick navigation

| For this | Go here |
|----------|---------|
| Installing and running QXScan | [Getting Started](getting-started.md) |
| Understanding the pipeline and modules | [Architecture](architecture.md) |
| QEM event model specification | [QEM Spec](qem-spec.md) |
| Compliance standards and scoring | [Compliance](compliance.md) |
| Exporting results to observability formats | [Export](export.md) |
| Daemon lifecycle and scheduler | [Daemon](daemon.md) |
| Building, testing, and CI | [Testing](testing.md) |
| Contributing code | [Contributing](contributing.md) |
| OSS vs Enterprise feature boundary | [Enterprise](enterprise.md) |

---

## Map

```
docs/
├── README.md            ← This file
├── getting-started.md   ─── CLI surface, install, quick start
├── architecture.md      ─── Pipeline, source tree, module contracts
├── qem-spec.md          ─── QX Event Model (publishable spec)
├── compliance.md        ─── Standards, controls, scoring
├── export.md            ─── Output formats (Prometheus, OCSF, CEF)
├── daemon.md            ─── Background service, scheduler, SQLite
├── testing.md           ─── Build, test, lint, demo suite
├── contributing.md      ─── Coding conventions, PR checklist
└── enterprise.md        ─── OSS/Enterprise boundary, upgrade path
```

---

## Audience

| Document | Best for |
|----------|----------|
| Getting Started | New users, operators |
| Architecture | Developers, integrators |
| QEM Spec | Tool builders, SIEM engineers |
| Compliance | Compliance officers, security engineers |
| Export | Platform engineers, observability teams |
| Daemon | SREs, operations |
| Testing | CI/CD engineers, QA |
| Contributing | Open source contributors |
| Enterprise | Procurement, architects |
