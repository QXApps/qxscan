//! ############################################################################
//! @file       observation.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      TLS evidence layer — raw wire data: TlsInfo, CertInfo.
//!
//! @details    Part of the QEM (QX Event Model) schema module. This file is
//!             part of the stable public API and must not break backward
//!             compatibility without a schema_version bump.
//!
//! ### REVISION HISTORY
//! | Date       | Version | Author                  | Description |
//! |------------|---------|-------------------------|-------------|
//! | 2026-06-02 | 1.0.0   | Phaneendra Bhattiprolu  | Initial implementation. |
//! |            |         |                         |             |
//!
//! ### COMMENTS / NOTES
//! * External deps only: serde, chrono, uuid, std.
//! * No dependency on any other qxscan module.
//! * Every public struct must be Serialize + Deserialize.
//! ############################################################################
//! Evidence layer — raw wire data, uninterpreted.
//! TlsInfo and CertInfo are what we observed on the wire, nothing more.
//! No compliance meaning is assigned here — that lives in finding.rs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// TLS handshake and certificate details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsInfo {
    pub negotiated_version: String,
    pub cipher: String,
    pub forward_secrecy: bool,
    pub pqc_hybrid: bool,
    pub cert: Option<CertInfo>,
}

/// Certificate metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertInfo {
    pub subject: String,
    pub issuer: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    /// Convenience: days until expiry (negative = already expired).
    pub days_to_expiry: i64,
    pub san: Vec<String>,
    pub serial: Option<String>,
}
