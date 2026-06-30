//! ############################################################################
//! @file       scoring.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      Compliance scoring dispatcher — evaluate(event, standard) -> findings + score.
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
//! Evaluates compliance controls against a scan result
//! and produces Vec<Finding> + ComplianceScore.

use crate::qem::compliance::ComplianceScore;
use crate::qem::finding::Finding;
use crate::qem::metadata::ScanEvent;

/// Evaluate all requested standards against the scan event.
/// Returns (findings_for_this_standard, score).
pub fn evaluate(
    event: &ScanEvent,
    standard: &str,
) -> anyhow::Result<(Vec<Finding>, ComplianceScore)> {
    let findings = match standard {
        "pci-dss" => standards::pci_dss::evaluate(event),
        "hipaa" => standards::hipaa::evaluate(event),
        "soc2" => standards::soc2::evaluate(event),
        "fisma" => standards::fisma::evaluate(event),
        "pqc" => standards::pqc::evaluate(event),
        other => anyhow::bail!("unknown standard: {other}"),
    }?;

    let score = ComplianceScore::from_findings(&findings);
    Ok((findings, score))
}

use crate::compliance::standards;
