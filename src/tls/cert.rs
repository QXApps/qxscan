//! ############################################################################
//! @file       cert.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      Certificate chain parsing — extracts CertInfo from TLS certificate.
//!
//! @details    Pure-function module with no side effects, global state, or
//!             file I/O. Must be unit-testable without a network connection.
//!
//! ### REVISION HISTORY
//! | Date       | Version | Author                  | Description |
//! |------------|---------|-------------------------|-------------|
//! | 2026-06-02 | 1.0.0   | Phaneendra Bhattiprolu  | Initial implementation. |
//! |            |         |                         |             |
//!
//! ### COMMENTS / NOTES
//! * No global state, no file I/O, no println!.
//! * Unit-testable without a network connection.
//! ############################################################################
use chrono::{DateTime, NaiveDateTime, Utc};

use crate::qem::observation::CertInfo;

pub fn from_x509(cert: &openssl::x509::X509Ref) -> anyhow::Result<CertInfo> {
    let subject = cert
        .subject_name()
        .entries()
        .map(|entry| {
            let key = entry.object().nid().short_name().unwrap_or("?");
            let value = entry
                .data()
                .to_string()
                .map(|v| v.to_string())
                .unwrap_or_else(|_| "<non-utf8>".to_string());
            format!("{key}={value}")
        })
        .collect::<Vec<_>>()
        .join(", ");

    let issuer = cert
        .issuer_name()
        .entries()
        .map(|entry| {
            let key = entry.object().nid().short_name().unwrap_or("?");
            let value = entry
                .data()
                .to_string()
                .map(|v| v.to_string())
                .unwrap_or_else(|_| "<non-utf8>".to_string());
            format!("{key}={value}")
        })
        .collect::<Vec<_>>()
        .join(", ");

    let not_before = parse_asn1_time(cert.not_before())?;
    let not_after = parse_asn1_time(cert.not_after())?;
    let days_to_expiry = (not_after - Utc::now()).num_days();

    let serial = cert
        .serial_number()
        .to_bn()
        .ok()
        .and_then(|bn| bn.to_hex_str().ok().map(|s| s.to_string()));

    let san = cert
        .subject_alt_names()
        .map(|names| {
            names
                .iter()
                .filter_map(|name| name.dnsname().map(ToString::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(CertInfo {
        subject,
        issuer,
        not_before,
        not_after,
        days_to_expiry,
        san,
        serial,
    })
}

fn parse_asn1_time(time: &openssl::asn1::Asn1TimeRef) -> anyhow::Result<DateTime<Utc>> {
    let raw = time.to_string();
    let naive = NaiveDateTime::parse_from_str(&raw, "%b %e %H:%M:%S %Y GMT")?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}
