//! Integration tests for the report module — terminal, JSON, and HTML renderers.
//!
//! Tests cover:
//!   src/report/terminal.rs
//!   src/report/json.rs
//!   src/report/html.rs

use chrono::{TimeDelta, Utc};
use qxscan::qem::compliance::ComplianceScore;
use qxscan::qem::finding::{Finding, FindingStatus, Severity};
use qxscan::qem::metadata::{ScanEvent, TargetInfo};
use qxscan::qem::observation::{CertInfo, TlsInfo};

/// Helper: build a ScanEvent with TLS info, findings, and compliance scores.
fn make_test_event() -> ScanEvent {
    let mut event = ScanEvent::new(TargetInfo {
        host: "test.example.com".into(),
        ip: Some("10.0.0.1".into()),
        port: 443,
        service: "https".into(),
    });

    event.tls = Some(TlsInfo {
        negotiated_version: "TLSv1.3".into(),
        cipher: "TLS_AES_256_GCM_SHA384".into(),
        forward_secrecy: true,
        pqc_hybrid: false,
        cert: Some(CertInfo {
            subject: "CN=test.example.com".into(),
            issuer: "CN=TestCA".into(),
            not_before: Utc::now(),
            not_after: Utc::now() + TimeDelta::days(90),
            days_to_expiry: 90,
            san: vec!["test.example.com".into()],
            serial: Some("01".into()),
        }),
    });

    event.findings = vec![
        Finding {
            control_id: "PCI-DSS-4.2.1".into(),
            standard: "pci-dss".into(),
            status: FindingStatus::Pass,
            severity: Severity::Critical,
            title: "Strong TLS protocol required".into(),
            detail: "Negotiated: TLSv1.3".into(),
            remediation: None,
        },
        Finding {
            control_id: "PCI-DSS-4.2.1.fs".into(),
            standard: "pci-dss".into(),
            status: FindingStatus::Pass,
            severity: Severity::High,
            title: "Forward secrecy required".into(),
            detail: "Forward secrecy: true".into(),
            remediation: None,
        },
    ];

    let pci_score = ComplianceScore::from_findings(&event.findings);
    event.compliance.insert("pci_dss".into(), pci_score);

    event
}

// ── Terminal renderer ────────────────────────────────────────

#[test]
fn terminal_render_has_header() {
    let event = make_test_event();
    let output = qxscan::report::terminal::render(&event);
    assert!(output.contains("QXScan Report"));
    assert!(output.contains("test.example.com:443"));
    assert!(output.contains("TLSv1.3"));
    assert!(output.contains("TLS_AES_256_GCM_SHA384"));
}

#[test]
fn terminal_render_shows_findings() {
    let event = make_test_event();
    let output = qxscan::report::terminal::render(&event);
    assert!(output.contains("PCI-DSS-4.2.1"));
    assert!(output.contains("Findings (2)"));
}

#[test]
fn terminal_render_shows_compliance_scores() {
    let event = make_test_event();
    let output = qxscan::report::terminal::render(&event);
    assert!(output.contains("Compliance Scores"));
    assert!(output.contains("pci_dss"));
}

#[test]
fn terminal_render_no_tls_event() {
    let event = ScanEvent::new(TargetInfo {
        host: "no-tls.example.com".into(),
        ip: None,
        port: 80,
        service: "http".into(),
    });
    let output = qxscan::report::terminal::render(&event);
    assert!(output.contains("no-tls.example.com:80"));
    // Should not contain TLS section
    assert!(!output.contains("TLS:"));
}

// ── JSON renderer ────────────────────────────────────────────

#[test]
fn json_render_produces_valid_json() {
    let event = make_test_event();
    let output = qxscan::report::json::render(&event).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["target"]["host"], "test.example.com");
    assert_eq!(parsed["target"]["port"], 443);
    assert_eq!(parsed["schema_version"], "1");
}

#[test]
fn json_render_includes_findings() {
    let event = make_test_event();
    let output = qxscan::report::json::render(&event).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    let findings = parsed["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0]["control_id"], "PCI-DSS-4.2.1");
}

// ── HTML renderer ────────────────────────────────────────────

#[test]
fn html_render_produces_valid_html() {
    let event = make_test_event();
    let output = qxscan::report::html::render(&event).unwrap();
    assert!(output.starts_with("<!DOCTYPE html>"));
    assert!(output.contains("<html lang=\"en\">"));
    assert!(output.contains("</html>"));
}

#[test]
fn html_render_shows_target_info() {
    let event = make_test_event();
    let output = qxscan::report::html::render(&event).unwrap();
    assert!(output.contains("QXScan Report"));
    assert!(output.contains("test.example.com:443"));
    assert!(output.contains("TLSv1.3"));
}

#[test]
fn html_render_shows_findings() {
    let event = make_test_event();
    let output = qxscan::report::html::render(&event).unwrap();
    assert!(output.contains("PCI-DSS-4.2.1"));
    assert!(output.contains("Findings (2)"));
}

#[test]
fn html_render_shows_compliance_scores() {
    let event = make_test_event();
    let output = qxscan::report::html::render(&event).unwrap();
    assert!(output.contains("pci_dss"));
}
