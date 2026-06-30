//! ############################################################################
//! @file       hipaa.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      HIPAA Security Rule compliance controls — TLS and certificate validation.
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
            control_id: "HIPAA-164.312.1".into(),
            standard: "hipaa".into(),
            status: FindingStatus::Fail,
            severity: Severity::Critical,
            title: "Encryption of ePHI in transit required".into(),
            detail: "No TLS evidence captured for target".into(),
            remediation: Some("Enable TLS on the endpoint to encrypt ePHI in transit".into()),
        });
        return Ok(findings);
    };

    let tls12_ok = tls.negotiated_version.contains("1.2");
    let tls13_ok = crate::tls::cipher::is_tls13(&tls.negotiated_version);
    findings.push(Finding {
        control_id: "HIPAA-164.312.1".into(),
        standard: "hipaa".into(),
        status: if tls12_ok || tls13_ok {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::Critical,
        title: "Encryption of ePHI in transit".into(),
        detail: format!("Negotiated protocol: {}", tls.negotiated_version),
        remediation: if tls12_ok || tls13_ok {
            None
        } else {
            Some("Upgrade to TLS 1.2 or higher to encrypt ePHI in transit".into())
        },
    });

    findings.push(Finding {
        control_id: "HIPAA-164.312.1.cipher".into(),
        standard: "hipaa".into(),
        status: if crate::tls::cipher::is_strong_cipher(&tls.cipher) {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::High,
        title: "Strong encryption algorithm".into(),
        detail: format!("Negotiated cipher: {}", tls.cipher),
        remediation: if crate::tls::cipher::is_strong_cipher(&tls.cipher) {
            None
        } else {
            Some("Configure server to prefer AES-GCM or ChaCha20-Poly1305 ciphers".into())
        },
    });

    findings.push(Finding {
        control_id: "HIPAA-164.312.1.integrity".into(),
        standard: "hipaa".into(),
        status: if tls12_ok || tls13_ok {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::High,
        title: "Integrity controls for ePHI".into(),
        detail: format!("Negotiated protocol: {}", tls.negotiated_version),
        remediation: if tls12_ok || tls13_ok {
            None
        } else {
            Some("TLS 1.2+ provides AEAD integrity protection required for ePHI".into())
        },
    });

    let cert_valid = tls.cert.as_ref().is_some_and(|c| c.days_to_expiry >= 0);
    let has_subject = tls.cert.as_ref().is_some_and(|c| !c.subject.is_empty());
    findings.push(Finding {
        control_id: "HIPAA-164.312.2".into(),
        standard: "hipaa".into(),
        status: if cert_valid && has_subject {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::High,
        title: "Valid certificate required for identity verification".into(),
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
            Some("Install a valid certificate from a trusted CA and renew before expiry".into())
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
    fn hipaa_passes() {
        let tls = make_tls("TLSv1.3", "TLS_AES_256_GCM_SHA384", true, 90);
        let findings = evaluate(&make_event(Some(tls))).unwrap();
        assert!(findings.iter().all(|f| f.status == FindingStatus::Pass));
    }

    #[test]
    fn hipaa_fails_on_old_tls() {
        let tls = make_tls("TLSv1.0", "TLS_RSA_WITH_RC4_128_SHA", false, -1);
        let findings = evaluate(&make_event(Some(tls))).unwrap();
        assert!(findings.iter().any(|f| f.status == FindingStatus::Fail));
    }

    #[test]
    fn hipaa_fails_on_no_tls() {
        let findings = evaluate(&make_event(None)).unwrap();
        assert!(findings.iter().any(|f| f.status == FindingStatus::Fail));
    }
}
