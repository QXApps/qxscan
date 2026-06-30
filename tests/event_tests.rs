//! Integration tests for the event module — ScanEvent and ComplianceScore.
//!
//! Tests cover:
//!   src/qem/metadata.rs — ScanEvent construction
//!   src/qem/compliance.rs — ComplianceScore grading

use qxscan::qem::compliance::ComplianceScore;
use qxscan::qem::finding::{Finding, FindingStatus, Severity};
use qxscan::qem::metadata::{ScanEvent, TargetInfo};

// ── ScanEvent construction ───────────────────────────────────

#[test]
fn scan_event_new_has_uuid_and_timestamp() {
    let target = TargetInfo {
        host: "example.com".into(),
        ip: Some("1.2.3.4".into()),
        port: 443,
        service: "https".into(),
    };
    let event = ScanEvent::new(target);

    assert_eq!(event.schema_version, "1");
    assert!(!event.scan_id.to_string().is_empty());
    assert_eq!(event.target.host, "example.com");
    assert_eq!(event.target.port, 443);
    assert!(event.findings.is_empty());
    assert!(event.compliance.is_empty());
}

// ── ComplianceScore grading ──────────────────────────────────

#[test]
fn grade_a_plus() {
    assert_eq!(ComplianceScore::grade_from_score(100), "A+");
}

#[test]
fn grade_a() {
    assert_eq!(ComplianceScore::grade_from_score(99), "A");
    assert_eq!(ComplianceScore::grade_from_score(90), "A");
}

#[test]
fn grade_b() {
    assert_eq!(ComplianceScore::grade_from_score(89), "B");
    assert_eq!(ComplianceScore::grade_from_score(80), "B");
}

#[test]
fn grade_c() {
    assert_eq!(ComplianceScore::grade_from_score(79), "C");
    assert_eq!(ComplianceScore::grade_from_score(70), "C");
}

#[test]
fn grade_d() {
    assert_eq!(ComplianceScore::grade_from_score(69), "D");
    assert_eq!(ComplianceScore::grade_from_score(60), "D");
}

#[test]
fn grade_f() {
    assert_eq!(ComplianceScore::grade_from_score(59), "F");
    assert_eq!(ComplianceScore::grade_from_score(30), "F");
    assert_eq!(ComplianceScore::grade_from_score(0), "F");
}

// ── ComplianceScore from_findings ─────────────────────────────

fn make_finding(control_id: &str, status: FindingStatus) -> Finding {
    Finding {
        control_id: control_id.into(),
        standard: "pci-dss".into(),
        status,
        severity: Severity::High,
        title: "test".into(),
        detail: "test detail".into(),
        remediation: None,
    }
}

#[test]
fn from_findings_all_pass() {
    let findings = vec![
        make_finding("PCI-DSS-4.2.1", FindingStatus::Pass),
        make_finding("PCI-DSS-4.2.1.cipher", FindingStatus::Pass),
        make_finding("PCI-DSS-4.2.1.fs", FindingStatus::Pass),
        make_finding("PCI-DSS-4.2.1.1", FindingStatus::Pass),
    ];
    let score = ComplianceScore::from_findings(&findings);
    assert_eq!(score.controls_total, 4);
    assert_eq!(score.controls_passed, 4);
    assert_eq!(score.controls_failed, 0);
    assert_eq!(score.score, 100);
    assert_eq!(score.grade, "A+");
}

#[test]
fn from_findings_all_fail() {
    let findings = vec![
        make_finding("PCI-DSS-4.2.1", FindingStatus::Fail),
        make_finding("PCI-DSS-4.2.1.cipher", FindingStatus::Fail),
    ];
    let score = ComplianceScore::from_findings(&findings);
    assert_eq!(score.controls_total, 2);
    assert_eq!(score.controls_passed, 0);
    assert_eq!(score.controls_failed, 2);
    assert_eq!(score.score, 0);
    assert_eq!(score.grade, "F");
}

#[test]
fn from_findings_mixed() {
    let findings = vec![
        make_finding("A", FindingStatus::Pass),
        make_finding("B", FindingStatus::Fail),
        make_finding("C", FindingStatus::Pass),
        make_finding("D", FindingStatus::Pass),
    ];
    let score = ComplianceScore::from_findings(&findings);
    assert_eq!(score.controls_total, 4);
    assert_eq!(score.controls_passed, 3);
    assert_eq!(score.controls_failed, 1);
    assert_eq!(score.score, 75); // 3/4 * 100 = 75
    assert_eq!(score.grade, "C");
}

#[test]
fn from_findings_warned_counted_in_total() {
    let findings = vec![
        make_finding("A", FindingStatus::Pass),
        make_finding("B", FindingStatus::Warn),
    ];
    let score = ComplianceScore::from_findings(&findings);
    assert_eq!(score.controls_total, 2);
    assert_eq!(score.controls_passed, 1);
    assert_eq!(score.controls_warned, 1);
    assert_eq!(score.score, 50); // warned counts in total but not passed
}

#[test]
fn from_findings_empty() {
    let findings: Vec<Finding> = vec![];
    let score = ComplianceScore::from_findings(&findings);
    assert_eq!(score.controls_total, 0);
    assert_eq!(score.score, 0);
    assert_eq!(score.grade, "F");
}
