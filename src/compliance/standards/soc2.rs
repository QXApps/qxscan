//! ############################################################################
//! @file       soc2.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      SOC 2 Type II compliance controls — TLS, cipher, and certificate checks.
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
            control_id: "SOC2-CC6.1".into(),
            standard: "soc2".into(),
            status: FindingStatus::Fail,
            severity: Severity::Critical,
            title: "Encryption of data in transit required".into(),
            detail: "No TLS evidence captured for target".into(),
            remediation: Some("Enable TLS on the endpoint to encrypt data in transit".into()),
        });
        return Ok(findings);
    };

    let tls12_ok = tls.negotiated_version.contains("1.2");
    let tls13_ok = crate::tls::cipher::is_tls13(&tls.negotiated_version);
    findings.push(Finding {
        control_id: "SOC2-CC6.1".into(),
        standard: "soc2".into(),
        status: if tls12_ok || tls13_ok {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::Critical,
        title: "Encryption of data in transit".into(),
        detail: format!("Negotiated protocol: {}", tls.negotiated_version),
        remediation: if tls12_ok || tls13_ok {
            None
        } else {
            Some("Require TLS 1.2 or higher for all data in transit".into())
        },
    });

    findings.push(Finding {
        control_id: "SOC2-CC6.1.cipher".into(),
        standard: "soc2".into(),
        status: if crate::tls::cipher::is_strong_cipher(&tls.cipher) {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::High,
        title: "Strong cryptographic algorithms".into(),
        detail: format!("Negotiated cipher: {}", tls.cipher),
        remediation: if crate::tls::cipher::is_strong_cipher(&tls.cipher) {
            None
        } else {
            Some("Use strong ciphers: AES-GCM or ChaCha20-Poly1305".into())
        },
    });

    findings.push(Finding {
        control_id: "SOC2-CC6.1.cert".into(),
        standard: "soc2".into(),
        status: if tls.cert.as_ref().is_some_and(|c| c.days_to_expiry >= 0) {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::Medium,
        title: "Valid trust anchor for communications".into(),
        detail: match &tls.cert {
            Some(c) => format!(
                "Subject: {}, expires in {} days",
                c.subject, c.days_to_expiry
            ),
            None => "No certificate metadata captured".into(),
        },
        remediation: if tls.cert.as_ref().is_some_and(|c| c.days_to_expiry >= 0) {
            None
        } else {
            Some("Maintain valid certificates from a trusted CA".into())
        },
    });

    findings.push(Finding {
        control_id: "SOC2-CC6.1.fs".into(),
        standard: "soc2".into(),
        status: if tls.forward_secrecy {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::Medium,
        title: "Forward secrecy preferred".into(),
        detail: format!("Forward secrecy: {}", tls.forward_secrecy),
        remediation: if tls.forward_secrecy {
            None
        } else {
            Some("Enable ECDHE key exchange for forward secrecy".into())
        },
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

    fn make_tls(version: &str, cipher: &str, fs: bool, cert_days: i64) -> TlsInfo {
        TlsInfo {
            negotiated_version: version.into(),
            cipher: cipher.into(),
            forward_secrecy: fs,
            pqc_hybrid: false,
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
    fn soc2_passes() {
        let tls = make_tls("TLSv1.3", "TLS_AES_256_GCM_SHA384", true, 90);
        let findings = evaluate(&make_event(Some(tls))).unwrap();
        assert!(findings.iter().all(|f| f.status == FindingStatus::Pass));
    }

    #[test]
    fn soc2_fails_on_old_tls() {
        let tls = make_tls("TLSv1.0", "TLS_RSA_WITH_RC4_128_SHA", false, -1);
        let findings = evaluate(&make_event(Some(tls))).unwrap();
        assert!(findings.iter().any(|f| f.status == FindingStatus::Fail));
    }

    #[test]
    fn soc2_fails_on_no_tls() {
        let findings = evaluate(&make_event(None)).unwrap();
        assert!(findings.iter().any(|f| f.status == FindingStatus::Fail));
    }
}
