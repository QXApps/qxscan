//! ############################################################################
//! @file       metadata.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      ScanEvent envelope — scan_id, timestamp, target, and overall status.
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
//! ScanEvent envelope — the top-level struct emitted by every scan run.
//! Depends on observation.rs, finding.rs, compliance.rs for sub-types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::compliance::ComplianceScore;
use super::finding::Finding;
use super::observation::TlsInfo;

/// Top-level event emitted by every scan run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanEvent {
    /// Incremented on breaking schema changes. Current: "1".
    pub schema_version: String,

    /// UUID v4 — unique per scan run.
    pub scan_id: Uuid,

    /// RFC 3339 UTC timestamp when the scan started.
    pub scanned_at: DateTime<Utc>,

    /// Wall-clock duration of the scan in milliseconds.
    pub scan_duration_ms: u64,

    /// Scanner provenance — name, version, engine, platform.
    pub scanner: ScannerInfo,

    /// The scanned target.
    pub target: TargetInfo,

    /// High-level outcome.
    pub overall_status: ScanStatus,

    /// TLS probe results (None if the service does not use TLS).
    pub tls: Option<TlsInfo>,

    /// Per-control findings across all evaluated standards.
    pub findings: Vec<Finding>,

    /// Rolled-up compliance scores, keyed by standard slug.
    /// e.g. "pci_dss", "hipaa", "soc2", "fisma", "pqc"
    pub compliance: std::collections::HashMap<String, ComplianceScore>,
}

impl ScanEvent {
    pub fn new(target: TargetInfo) -> Self {
        Self {
            schema_version: crate::about::SCHEMA.into(),
            scan_id: Uuid::new_v4(),
            scanned_at: Utc::now(),
            scan_duration_ms: 0,
            scanner: ScannerInfo {
                name: crate::about::PRODUCT.into(),
                version: crate::about::BUILD.into(),
                engine: crate::about::ENGINE.into(),
                platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            },
            target,
            overall_status: ScanStatus::Pass,
            tls: None,
            findings: vec![],
            compliance: std::collections::HashMap::new(),
        }
    }
}

/// Scanner provenance metadata — included in every scan event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerInfo {
    /// Product name (e.g. "QXScan").
    pub name: String,
    /// Build version from Cargo.toml (e.g. "0.1.0").
    pub version: String,
    /// QEM engine name (e.g. "QEM"). The schema_version field above
    /// indicates the event format version.
    pub engine: String,
    /// Compile-time platform (e.g. "linux-x86_64").
    pub platform: String,
}

/// Resolved target information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    /// Input hostname or IP as provided by the user.
    pub host: String,
    /// Resolved IP address.
    pub ip: Option<String>,
    pub port: u16,
    /// Service type (https, smtp, postgres, …).
    pub service: String,
}

/// High-level scan outcome.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScanStatus {
    Pass,
    Fail,
    Warn,
    Error,
    Timeout,
    NoTls,
    UnsupportedProtocol,
    ConnectionFailed,
}
