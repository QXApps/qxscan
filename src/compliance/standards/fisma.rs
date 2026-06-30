//! ############################################################################
//! @file       fisma.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      FISMA / NIST SP 800-52r2 compliance controls — TLS profile evaluation.
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
            control_id: "FISMA-SP800-52r2.1".into(),
            standard: "fisma".into(),
            status: FindingStatus::Fail,
            severity: Severity::Critical,
            title: "NIST SP 800-52 TLS profile required".into(),
            detail: "No TLS evidence captured for target".into(),
            remediation: Some("Enable TLS 1.2 or 1.3 per NIST SP 800-52 Rev. 2".into()),
        });
        return Ok(findings);
    };

    let tls12_ok = tls.negotiated_version.contains("1.2");
    let tls13_ok = crate::tls::cipher::is_tls13(&tls.negotiated_version);
    findings.push(Finding {
        control_id: "FISMA-SP800-52r2.1".into(),
        standard: "fisma".into(),
        status: if tls13_ok || tls12_ok {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::Critical,
        title: "NIST-approved TLS protocol version".into(),
        detail: format!("Negotiated protocol: {}", tls.negotiated_version),
        remediation: if tls13_ok || tls12_ok {
            None
        } else {
            Some(
                "NIST SP 800-52 Rev. 2 requires TLS 1.2 or 1.3; disable SSL and TLS 1.0/1.1".into(),
            )
        },
    });

    let fips_cipher = crate::tls::cipher::is_strong_cipher(&tls.cipher);
    findings.push(Finding {
        control_id: "FISMA-SP800-52r2.1.cipher".into(),
        standard: "fisma".into(),
        status: if fips_cipher {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::High,
        title: "FIPS-compliant cipher suite".into(),
        detail: format!("Negotiated cipher: {}", tls.cipher),
        remediation: if fips_cipher {
            None
        } else {
            Some("NIST SP 800-52 requires FIPS-compliant ciphers (AES-GCM)".into())
        },
    });

    let cert_valid = tls.cert.as_ref().is_some_and(|c| c.days_to_expiry >= 0);
    findings.push(Finding {
        control_id: "FISMA-SP800-52r2.1.cert".into(),
        standard: "fisma".into(),
        status: if cert_valid {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::High,
        title: "Valid PKI certificate path".into(),
        detail: match &tls.cert {
            Some(c) => format!(
                "Subject: {}, expires in {} days",
                c.subject, c.days_to_expiry
            ),
            None => "No certificate metadata captured".into(),
        },
        remediation: if cert_valid {
            None
        } else {
            Some("Maintain a valid certificate chain per NIST SP 800-52".into())
        },
    });

    findings.push(Finding {
        control_id: "FISMA-SP800-52r2.1.fs".into(),
        standard: "fisma".into(),
        status: if tls.forward_secrecy {
            FindingStatus::Pass
        } else {
            FindingStatus::Warn
        },
        severity: Severity::Medium,
        title: "Forward secrecy recommended".into(),
        detail: format!("Forward secrecy: {}", tls.forward_secrecy),
        remediation: if tls.forward_secrecy {
            None
        } else {
            Some("NIST SP 800-52 recommends ECDHE for forward secrecy".into())
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
    fn fisma_passes() {
        let tls = make_tls("TLSv1.3", "TLS_AES_256_GCM_SHA384", true, 90);
        let findings = evaluate(&make_event(Some(tls))).unwrap();
        assert!(findings.iter().all(|f| f.status == FindingStatus::Pass));
    }

    #[test]
    fn fisma_fails_on_old_tls() {
        let tls = make_tls("TLSv1.0", "TLS_RSA_WITH_RC4_128_SHA", false, -1);
        let findings = evaluate(&make_event(Some(tls))).unwrap();
        assert!(findings.iter().any(|f| f.status == FindingStatus::Fail));
    }

    #[test]
    fn fisma_fails_on_no_tls() {
        let findings = evaluate(&make_event(None)).unwrap();
        assert!(findings.iter().any(|f| f.status == FindingStatus::Fail));
    }
}
