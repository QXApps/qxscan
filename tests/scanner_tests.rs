//! Integration tests for the scanner module — ServiceType and target resolution.
//!
//! Tests cover:
//!   src/scanner/service.rs — ServiceType::default_port() and ServiceType::slug()
//!   src/scanner/mod.rs — resolve_targets()

use std::io::Write;

use qxscan::scanner::service::ServiceType;

// ── ServiceType default_port ─────────────────────────────────

#[test]
fn service_default_ports() {
    assert_eq!(ServiceType::Https.default_port(), 443);
    assert_eq!(ServiceType::Smtp.default_port(), 587);
    assert_eq!(ServiceType::Imap.default_port(), 993);
    assert_eq!(ServiceType::Pop3.default_port(), 995);
    assert_eq!(ServiceType::Postgres.default_port(), 5432);
    assert_eq!(ServiceType::Mysql.default_port(), 3306);
    assert_eq!(ServiceType::Ldap.default_port(), 636);
    assert_eq!(ServiceType::Ftp.default_port(), 990);
}

// ── ServiceType slug ─────────────────────────────────────────

#[test]
fn service_slugs() {
    assert_eq!(ServiceType::Https.slug(), "https");
    assert_eq!(ServiceType::Smtp.slug(), "smtp");
    assert_eq!(ServiceType::Imap.slug(), "imap");
    assert_eq!(ServiceType::Pop3.slug(), "pop3");
    assert_eq!(ServiceType::Postgres.slug(), "postgres");
    assert_eq!(ServiceType::Mysql.slug(), "mysql");
    assert_eq!(ServiceType::Ldap.slug(), "ldap");
    assert_eq!(ServiceType::Ftp.slug(), "ftp");
}

// ── Target resolution ────────────────────────────────────────

#[test]
fn resolve_single_target() {
    let targets =
        qxscan::scanner::resolve_targets(Some("example.com"), None, 443, &ServiceType::Https)
            .unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].host, "example.com");
    assert_eq!(targets[0].port, 443);
    assert!(matches!(targets[0].service, ServiceType::Https));
}

#[test]
fn resolve_target_with_custom_port() {
    let targets =
        qxscan::scanner::resolve_targets(Some("example.com:8443"), None, 443, &ServiceType::Https)
            .unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].host, "example.com");
    assert_eq!(targets[0].port, 8443);
}

#[test]
fn resolve_targets_default_port() {
    let targets =
        qxscan::scanner::resolve_targets(Some("db.internal"), None, 5432, &ServiceType::Postgres)
            .unwrap();
    assert_eq!(targets[0].host, "db.internal");
    assert_eq!(targets[0].port, 5432);
    assert!(matches!(targets[0].service, ServiceType::Postgres));
}

#[test]
fn resolve_targets_from_file() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "host1.example.com").unwrap();
    writeln!(file, "host2.example.com:9443").unwrap();
    writeln!(file, "# comment line").unwrap();
    writeln!(file, "").unwrap();
    writeln!(file, "host3.example.com").unwrap();
    file.flush().unwrap();

    let targets =
        qxscan::scanner::resolve_targets(None, Some(file.path()), 443, &ServiceType::Https)
            .unwrap();

    // Should only parse non-empty, non-comment lines
    assert_eq!(targets.len(), 3);
    assert_eq!(targets[0].host, "host1.example.com");
    assert_eq!(targets[0].port, 443);
    assert_eq!(targets[1].host, "host2.example.com");
    assert_eq!(targets[1].port, 9443);
    assert_eq!(targets[2].host, "host3.example.com");
    assert_eq!(targets[2].port, 443);
}

#[test]
fn resolve_targets_both_sources() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "file-target.example.com").unwrap();
    file.flush().unwrap();

    let targets = qxscan::scanner::resolve_targets(
        Some("cli-target.example.com"),
        Some(file.path()),
        443,
        &ServiceType::Https,
    )
    .unwrap();

    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0].host, "cli-target.example.com");
    assert_eq!(targets[1].host, "file-target.example.com");
}

#[test]
fn resolve_targets_no_source_errors() {
    let result = qxscan::scanner::resolve_targets(None, None, 443, &ServiceType::Https);
    assert!(result.is_err());
}
