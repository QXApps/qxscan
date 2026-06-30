//! ############################################################################
//! @file       mod.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      QEM event module — re-exports all public QEM types.
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
//! Canonical ScanEvent types — the stable public schema for all qxscan output.
//!
//! Four-file layout following the pipeline: Scan → Evidence → Observation → Export
//!   metadata.rs    — ScanEvent envelope (scan_id, scanned_at, target, status)
//!   observation.rs — Evidence layer: TlsInfo, CertInfo (raw wire data)
//!   finding.rs     — Observation layer: Finding, FindingStatus, Severity
//!   compliance.rs  — Observation layer: ComplianceScore, grade_from_score()
//!
//! Treat these types like a public API. Do not change field names or types
//! after v1.0 without a schema_version bump.

pub mod compliance;
pub mod finding;
pub mod metadata;
pub mod observation;

#[allow(unused_imports)]
pub use compliance::*;
#[allow(unused_imports)]
pub use finding::*;
#[allow(unused_imports)]
pub use metadata::*;
#[allow(unused_imports)]
pub use observation::*;
