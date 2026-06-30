//! ############################################################################
//! @file       pci_dss.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      PCI-DSS 4.0 compliance controls — TLS version, cipher, forward secrecy, cert.
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
//! PCI_DSS compliance controls.
//!
//! Each public evaluate() function receives a &ScanEvent and returns
//! Vec<Finding>. Add controls here as the policy engine matures.

use crate::qem::finding::{Finding, FindingStatus, Severity};
use crate::qem::metadata::ScanEvent;

pub fn evaluate(event: &ScanEvent) -> anyhow::Result<Vec<Finding>> {
    let mut findings = Vec::new();

    let Some(tls) = &event.tls else {
        findings.push(Finding {
            control_id: "PCI-DSS-4.2.1".into(),
            standard: "pci-dss".into(),
            status: FindingStatus::Fail,
            severity: Severity::Critical,
            title: "TLS required".into(),
            detail: "No TLS evidence captured for target".into(),
            remediation: Some("Use a TLS-enabled endpoint and re-scan".into()),
        });
        return Ok(findings);
    };

    let version_ok = tls.negotiated_version.contains("1.2")
        || crate::tls::cipher::is_tls13(&tls.negotiated_version);
    findings.push(Finding {
        control_id: "PCI-DSS-4.2.1".into(),
        standard: "pci-dss".into(),
        status: if version_ok {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::Critical,
        title: "Strong TLS protocol required".into(),
        detail: format!("Negotiated protocol: {}", tls.negotiated_version),
        remediation: if version_ok {
            None
        } else {
            Some("Disable TLS 1.0/1.1 and require TLS 1.2+".into())
        },
    });

    let strong_cipher = crate::tls::cipher::is_strong_cipher(&tls.cipher);
    findings.push(Finding {
        control_id: "PCI-DSS-4.2.1.cipher".into(),
        standard: "pci-dss".into(),
        status: if strong_cipher {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::Critical,
        title: "Strong cipher suite required".into(),
        detail: format!("Negotiated cipher: {}", tls.cipher),
        remediation: if strong_cipher {
            None
        } else {
            Some("Prefer AES-GCM or ChaCha20-Poly1305 cipher suites".into())
        },
    });

    findings.push(Finding {
        control_id: "PCI-DSS-4.2.1.fs".into(),
        standard: "pci-dss".into(),
        status: if tls.forward_secrecy {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::High,
        title: "Forward secrecy required".into(),
        detail: format!("Forward secrecy: {}", tls.forward_secrecy),
        remediation: if tls.forward_secrecy {
            None
        } else {
            Some("Enable ECDHE/DHE based TLS key exchange".into())
        },
    });

    let cert_valid = tls.cert.as_ref().is_some_and(|c| c.days_to_expiry >= 0);
    findings.push(Finding {
        control_id: "PCI-DSS-4.2.1.1".into(),
        standard: "pci-dss".into(),
        status: if cert_valid {
            FindingStatus::Pass
        } else {
            FindingStatus::Fail
        },
        severity: Severity::Critical,
        title: "Certificate must be valid and unexpired".into(),
        detail: match &tls.cert {
            Some(cert) => format!("Certificate days_to_expiry: {}", cert.days_to_expiry),
            None => "No certificate metadata captured".into(),
        },
        remediation: if cert_valid {
            None
        } else {
            Some("Install a valid certificate chain and renew expired certs".into())
        },
    });

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qem::finding::FindingStatus;
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
    fn pci_dss_passes() {
        let tls = make_tls("TLSv1.3", "TLS_AES_256_GCM_SHA384", true, 90);
        let findings = evaluate(&make_event(Some(tls))).unwrap();
        assert!(findings.iter().all(|f| f.status == FindingStatus::Pass));
    }

    #[test]
    fn pci_dss_fails_on_old_tls() {
        let tls = make_tls("TLSv1.1", "TLS_RSA_WITH_AES_128_CBC_SHA", false, -10);
        let findings = evaluate(&make_event(Some(tls))).unwrap();
        assert!(findings.iter().any(|f| f.status == FindingStatus::Fail));
    }

    #[test]
    fn pci_dss_fails_on_no_tls() {
        let findings = evaluate(&make_event(None)).unwrap();
        assert!(findings.iter().any(|f| f.status == FindingStatus::Fail));
    }
}
