# QX Enterprise — The Upgrade Path

> **QXScan OSS is a complete scanner. QX Enterprise adds fleet management,
> not scanner features.**

---

## The Philosophy

QXScan OSS is intentionally **complete** as a scanner. It connects to
servers, collects TLS evidence, evaluates compliance, produces structured
output, and gets out of the way. It runs on a single machine, stores
history in SQLite, and emits open formats that any downstream tool can
consume.

See [ROADMAP.md](../ROADMAP.md) for the planned feature timeline across
OSS releases and Enterprise.

Enterprise features layer on top — they add **fleet management,
aggregation, alerting, and dashboards** without changing how scanning works.

```
QXScan OSS:         scan → emit QEM → done
QX Enterprise:      consume QEM → aggregate → enrich → alert → dashboard
```

---

## What You Get with OSS

| Feature | Details |
|---------|---------|
| Scanner engine | All service types, `--targets-file`, `--parallel` |
| TLS probe | Handshake, cert, cipher, PQC detection |
| Compliance engine | PCI-DSS, HIPAA, SOC2, FISMA, PQC |
| Compliance scoring + grading | A+ through F, with per-control detail |
| QEM (QX Event Model) | Permanently OSS, never proprietary |
| `qxscan scan` | Including file-based targets and parallel scanning |
| `qxscan server` daemon | Job runner — schedule-based scanning |
| `qxscan schedule` | Single machine, SQLite, system clock |
| `qxscan export` | QEM, Prometheus, OCSF, CEF — single-shot |
| `qxscan report` | Terminal, JSON, HTML — list, show, remove |
| Local scan history | SQLite, trend on single machine |
| `Exporter` trait | Public, documented, implementable by anyone |

---

## What Enterprise Adds

### Fleet Management

| Feature | Description |
|---------|-------------|
| Multi-node coordination | Centrally schedule scans across 1000s of nodes |
| Agent deployment | Managed agents on every target network |
| Fleet dashboard | Single pane for all scan nodes |
| Multi-site support | Scan across data centers, clouds, and offices |

### Alerting & Notifications

| Feature | Description |
|---------|-------------|
| Alert engine | IF score < threshold THEN notify |
| PagerDuty | Incident management integration |
| Slack / Teams | Channel notifications |
| Email | SMTP-based alerting |
| Custom alert rules | Per-standard, per-severity thresholds |

### Enterprise Integrations

| Feature | Description |
|---------|-------------|
| Elastic connector | Bidirectional continuous push + pull |
| Splunk HEC connector | Bidirectional |
| Datadog connector | Bidirectional |
| OpenTelemetry (OTLP) | Bidirectional |
| Continuous push | Retry, batching, compression, backpressure |

### Management Features

| Feature | Description |
|---------|-------------|
| Web UI + dashboards | Never in OSS |
| REST API | Query, trigger, aggregate across nodes |
| SSO / OIDC / SAML / LDAP | Enterprise authentication |
| RBAC + teams | Role-based access control |
| Compliance trend analytics | Cross-fleet, org-level rollup |
| Asset inventory | Tags, owners, business unit, criticality |
| Custom policy sets | Finance, Healthcare, Production policies |
| Audit log | Immutable change history |

---

## The Upgrade Path

### How OSS feeds Enterprise

```
OSS user runs:   qxscan scan example.com --output json --out scan.json
                 → produces QEM JSON

Enterprise does: consume QEM JSON → aggregate across fleet
                 → enrich with CMDB data → alert on threshold breaches
                 → display in web dashboard
```

### No vendor lock-in

QEM is the integration point. Any tool can produce or consume it:

```
GitHub Project A → exports QEM → GitHub Project B → reads QEM
```

If you build QEM integrations today, they will work with both OSS and
Enterprise tomorrow.

### Migration path

```
Phase 1 — OSS (today)
  └── Single node, SQLite, manual or cron-driven scanning

Phase 2 — OSS + scripting
  └── Multiple nodes, cron schedule, custom aggregation scripts

Phase 3 — Enterprise (future)
  └── Fleet management, central dashboard, automated alerting
```

Each phase is independent. You can stay on Phase 1 indefinitely and get
full value from the scanner.

---

## OSS / Enterprise Boundary (Detailed)

### OSS owns

- Scanner engine (all service types)
- TLS probe (handshake, cert, cipher, PQC detection)
- Compliance engine (all 5 standards)
- Compliance scoring + grading
- QEM — forever open
- All CLI subcommands (scan, server, schedule, export, report, version)
- Local SQLite history
- `Exporter` trait

### Enterprise owns

- Fleet management and agent coordination
- Web UI + dashboards
- SSO / OIDC / SAML / LDAP
- RBAC + teams
- Elastic / Splunk / Datadog / OTLP connectors
- Continuous push (retry, batching, compression)
- Alerting engine (PagerDuty, Slack, Teams, email)
- Custom policy sets
- Compliance trend analytics
- Audit log

### The `ExportFormat` boundary

The `ExportFormat` enum in `cli/export.rs` lists OSS formats only:
`qem`, `prometheus`, `ocsf`, `cef`. Enterprise format variants must never
appear in this enum — they belong in the Enterprise repository.

---

## FAQ

### Is OSS feature-limited?

No. QXScan OSS is a **complete TLS security and compliance scanner**.
It scans servers, evaluates compliance, and exports results. Nothing is
artificially limited or time-bombed.

### Will OSS features be moved to Enterprise?

No. The OSS/Enterprise boundary is fixed. Scanner, TLS, compliance, QEM,
and CLI features are permanently OSS.

### Can I use OSS in production?

Yes. QXScan OSS is production-ready for single-node scanning. It's used
in production environments today.

### What if I need fleet management?

Start with OSS on each node. Use the QEM export format to aggregate
results. When your fleet grows beyond what manual aggregation can handle,
evaluate QX Enterprise.

### Is QEM going to become proprietary?

**No.** QEM is permanently open source. Its ecosystem value comes from
any tool being able to produce or consume it.
