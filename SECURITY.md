# Security Policy

## Reporting a Vulnerability

QXScan is a security tool — finding and fixing vulnerabilities in it is a
priority. If you discover a security issue, please report it privately.

### Contact

**Do not open a public GitHub issue.** Send details to:

- **GitHub Security Advisory**: Use the "Report a Vulnerability" button
  at https://github.com/QXApps/qxscan/security/advisories
- **Email**: `security@qxapps.net` (if available)

### What to include

- Description of the vulnerability
- Steps to reproduce (config, target, command used)
- Affected version(s)
- Potential impact
- Suggested fix (optional)

### Response timeline

| Timeframe | Action |
|-----------|--------|
| 48 hours | Acknowledgment of receipt |
| 7 days | Initial assessment and severity classification |
| 30 days | Fix released or detailed plan communicated |

### Scope

The following are **in scope** for security reports:

- The `qxscan` binary itself (buffer overflows, crashes, code execution)
- TLS handling and certificate validation bypasses
- SQL injection or data leakage through the state store
- Credential exposure via logs or error messages
- Command injection via CLI arguments or config files

The following are **out of scope**:

- Theoretical attacks requiring local file system access
- Dependency CVAs with no demonstrated exploit path
- Social engineering of QXScan maintainers

## Responsible Disclosure

We ask that you:

1. Report vulnerabilities privately (as above)
2. Allow reasonable time for a fix before public disclosure
3. Do not exploit the vulnerability beyond what's needed to demonstrate it

We commit to:

1. Respond promptly and keep you informed
2. Credit you in release notes (if desired)
3. Fix validated issues in a timely manner

## Security Features

QXScan is designed with the following security properties in mind:

- **Single binary** — no runtime dependencies, minimal attack surface
- **No network daemon** — the scanner connects outbound only
- **SQLite by default** — no external database credentials needed
- **Static linking** — no shared library loading
- **Credential safety** — passwords use env var expansion, never CLI flags
- **Password masking** — connection URLs are masked in all log output

---

*QXScan OSS — Security-first design*
