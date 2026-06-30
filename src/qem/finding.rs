//! ############################################################################
//! @file       finding.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      Compliance finding — Finding, FindingStatus, Severity types.
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
//! Observation layer — interpreted compliance findings.
//! A Finding is the conclusion drawn from Evidence (observation.rs).

use serde::{Deserialize, Serialize};

/// A single compliance control finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Standard-specific control identifier. e.g. "PCI-DSS-4.2.1"
    pub control_id: String,

    /// Standard slug. e.g. "pci-dss", "hipaa", "soc2", "fisma", "pqc"
    pub standard: String,

    /// Outcome for this control.
    pub status: FindingStatus,

    /// Severity of this control.
    pub severity: Severity,

    /// Short human-readable title.
    pub title: String,

    /// Evidence detail (what was observed).
    pub detail: String,

    /// Remediation guidance (None when status is Pass).
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FindingStatus {
    Pass,
    Fail,
    Warn,
    /// Control not applicable for this service/target.
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}
