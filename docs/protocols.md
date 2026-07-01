# Supported Protocols

QXScan probes target services over TLS and classifies them by protocol.
This document describes each supported protocol, its default port, TLS
negotiation behavior, and any known limitations.

---

## Protocol Reference

| Protocol | Slug      | Default Port | TLS Mode       | Description                         |
|----------|-----------|-------------|----------------|-------------------------------------|
| HTTPS    | `https`   | 443         | Implicit TLS   | HTTP over TLS — the most common target |
| SMTP     | `smtp`    | 587         | STARTTLS       | Mail submission with opportunistic TLS |
| IMAP     | `imap`    | 993         | Implicit TLS   | Mail retrieval over TLS              |
| POP3     | `pop3`    | 995         | Implicit TLS   | Mail retrieval over TLS              |
| PostgreSQL | `postgres` | 5432      | Implicit TLS   | Database connection with direct TLS  |
| MySQL    | `mysql`   | 3306        | Implicit TLS   | Database connection with direct TLS  |
| LDAP     | `ldap`    | 636         | Implicit TLS   | Directory services over TLS (LDAPS)  |
| FTP      | `ftp`     | 990         | Implicit TLS   | File transfer over TLS (FTPS)        |

---

## TLS Negotiation Modes

### Implicit TLS

The scanner establishes a TCP connection and immediately performs a TLS
handshake. This is the most common mode and applies to HTTPS, IMAPS, POP3S,
PostgreSQL, MySQL, LDAPS, and FTPS.

### STARTTLS (Opportunistic TLS)

The scanner connects over plain TCP, then negotiates TLS via a
protocol-specific command (e.g., `STARTTLS` for SMTP). Currently supported:

- **SMTP** (port 587): Issues `STARTTLS` after the server greeting.

STARTTLS support for IMAP, POP3, and LDAP is planned for v0.2.

---

## Service Detection

When you specify a `--service` flag, QXScan uses protocol-aware probing:

```bash
qxscan scan mail.example.com --port 587 --service smtp
```

If no `--service` is provided, the scanner defaults to `https` (port 443).
This covers the vast majority of use cases.

---

## Protocol-Specific Notes

### HTTPS

The default and most thoroughly tested protocol. Full TLS 1.2/1.3 support with
PQC-hybrid key exchange detection, certificate inspection, and cipher suite
evaluation.

### SMTP

Requires `--service smtp` and typically port 587. The scanner issues `STARTTLS`
after receiving the server banner. Certificate verification works the same as
HTTPS.

### PostgreSQL / MySQL

Database protocols use direct TLS on their respective ports. Certificate
verification is supported. The scanner does not authenticate to the database —
it only performs the TLS handshake.

### LDAP

LDAPS (LDAP over TLS) on port 636. The scanner performs a direct TLS handshake.
Plain LDAP (port 389) with STARTTLS is not yet supported.

### IMAP / POP3

Implicit TLS on ports 993 and 995 respectively. STARTTLS variants (ports 143
and 110) are planned for v0.2.

### FTP

FTPS (FTP over TLS) on port 990 using implicit TLS. Explicit FTPS (AUTH TLS)
is not yet supported.

---

## Adding a New Protocol

To add protocol support:

1. Add a variant to `ServiceType` in [`src/scanner/service.rs`](../src/scanner/service.rs).
2. Implement TLS probing in [`src/tls/handshake.rs`](../src/tls/handshake.rs).
3. Add STARTTLS negotiation if the protocol requires it.
4. Update this document.

---

*See also: [Architecture](architecture.md), [QEM Spec](qem-spec.md)*
