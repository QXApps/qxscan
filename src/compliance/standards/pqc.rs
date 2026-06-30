//! ############################################################################
//! @file       pqc.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      PQC Readiness (CNSA 2.0) compliance controls — hybrid key exchange detection.
//!
//! @details
//!
//! ### REVISION HISTORY
//! | Date       | Version | Author                  | Description |
//! |------------|---------|-------------------------|-------------|
//! | 2026-06-02 | 1.0.0   | Phaneendra Bhattiprolu  | Initial implementation. |
//! |            |         |                         |             |
//!
//! ### COMMENTS / NOTES
//! ############################################################################
use crate::qem::finding::{Finding, FindingStatus, Severity};
use crate::qem::metadata::ScanEvent;

pub fn evaluate(event: &ScanEvent) -> anyhow::Result<Vec<Finding>> {
    let mut findings = Vec::new();

    let Some(tls) = &event.tls else {
        findings.push(Finding {
            control_id: "PQC-1.1".into(),
            standard: "pqc".into(),
            status: FindingStatus::NotApplicable,
            severity: Severity::Info,
            title: "Post-quantum cryptography readiness".into(),
            detail: "No TLS session to evaluate for PQC readiness".into(),
            remediation: None,
        });
        return Ok(findings);
    };

    // PQC-1.1: PQC-hybrid key exchange detection
    // Rules (locked): pqc_hybrid=true → pass, pqc_hybrid=false → warn
    let pqc_hybrid_detected = tls.pqc_hybrid;
    findings.push(Finding {
        control_id: "PQC-1.1".into(),
        standard: "pqc".into(),
        status: if pqc_hybrid_detected {
            FindingStatus::Pass
        } else {
            FindingStatus::Warn
        },
        severity: Severity::Info,
        title: "PQC-hybrid key exchange detection".into(),
        detail: if pqc_hybrid_detected {
            format!("PQC-hybrid key exchange detected: {}", tls.cipher)
        } else {
            format!("Cipher: {} (no PQC hybrid detected)", tls.cipher)
        },
        remediation: None,
    });

    // PQC-1.2: Certificate cryptographic strength
    let has_valid_cert = tls.cert.as_ref().is_some_and(|c| c.days_to_expiry >= 0);
    findings.push(Finding {
        control_id: "PQC-1.2".into(),
        standard: "pqc".into(),
        status: if has_valid_cert {
            FindingStatus::Pass
        } else {
            FindingStatus::Warn
        },
        severity: Severity::Low,
        title: "Certificate cryptographic strength".into(),
        detail: match &tls.cert {
            Some(c) => format!("Subject: {}, issuer: {}", c.subject, c.issuer),
            None => "No certificate metadata captured".into(),
        },
        remediation: if has_valid_cert {
            None
        } else {
            Some("Transition to PQC-ready certificates (hybrid certs with ML-KEM)".into())
        },
    });

    // PQC-1.3: Cipher agility for PQC migration
    // Rules: pqc_hybrid=true,TLS 1.3 → pass | pqc_hybrid=false,TLS 1.3 → warn |
    //        pqc_hybrid=false,TLS 1.2 → fail
    let is_tls13 = crate::tls::cipher::is_tls13(&tls.negotiated_version);
    let status_pqc13 = if pqc_hybrid_detected {
        FindingStatus::Pass
    } else if is_tls13 {
        FindingStatus::Warn
    } else {
        FindingStatus::Fail
    };
    findings.push(Finding {
        control_id: "PQC-1.3".into(),
        standard: "pqc".into(),
        status: status_pqc13,
        severity: Severity::Info,
        title: "Cipher agility for PQC migration".into(),
        detail: format!(
            "Negotiated protocol: {}, cipher: {}",
            tls.negotiated_version, tls.cipher
        ),
        remediation: None,
    });

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qem::metadata::TargetInfo;
    use crate::qem::observation::{CertInfo, TlsInfo};
    use chrono::{TimeDelta, Utc};

    fn make_event(tls: Option<TlsInfo>) -> ScanEvent {
        let mut event = ScanEvent::new(TargetInfo {
            host: "test.example.com".into(),
            ip: Some("1.2.3.4".into()),
            port: 443,
            service: "https".into(),
        });
        event.tls = tls;
        event
    }

    fn make_tls(version: &str, cipher: &str, pqc: bool, cert_days: i64) -> TlsInfo {
        TlsInfo {
            negotiated_version: version.into(),
            cipher: cipher.into(),
            forward_secrecy: true,
            pqc_hybrid: pqc,
            cert: Some(CertInfo {
                subject: "CN=test.example.com".into(),
                issuer: "CN=TestCA".into(),
                not_before: Utc::now(),
                not_after: Utc::now() + TimeDelta::days(cert_days),
                days_to_expiry: cert_days,
                san: vec!["test.example.com".into()],
                serial: Some("01".into()),
            }),
        }
    }

    #[test]
    fn pqc_passes() {
        let tls = make_tls("TLSv1.3", "TLS_AES_256_GCM_SHA384", false, 90);
        let findings = evaluate(&make_event(Some(tls))).unwrap();
        assert!(findings.iter().all(|f| f.status != FindingStatus::Fail));
    }

    #[test]
    fn pqc_detects_hybrid() {
        let tls = make_tls("TLSv1.3", "TLS_KYBER_MLKEM_HYBRID", true, 90);
        let findings = evaluate(&make_event(Some(tls))).unwrap();
        assert!(findings
            .iter()
            .any(|f| f.status == FindingStatus::Pass && f.control_id == "PQC-1.1"));
    }

    #[test]
    fn pqc_no_tls_is_not_applicable() {
        let findings = evaluate(&make_event(None)).unwrap();
        assert!(findings
            .iter()
            .any(|f| f.status == FindingStatus::NotApplicable));
    }
}
