//! ############################################################################
//! @file       compliance.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      Compliance scoring — ComplianceScore, grade_from_score(), from_findings().
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
//! Observation layer — rolled-up compliance scores.
//! ComplianceScore aggregates Findings (from finding.rs) into a grade.

use serde::{Deserialize, Serialize};

use super::finding::{Finding, FindingStatus};

/// Rolled-up compliance score for a single standard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScore {
    /// Numeric score 0–100.
    pub score: u8,

    /// Letter grade: A+, A, B, C, D, F.
    pub grade: String,

    pub controls_passed: u32,
    pub controls_failed: u32,
    pub controls_warned: u32,
    pub controls_total: u32,
}

impl ComplianceScore {
    /// Derive grade from numeric score.
    pub fn grade_from_score(score: u8) -> String {
        match score {
            100 => "A+",
            90..=99 => "A",
            80..=89 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F",
        }
        .into()
    }

    pub fn from_findings(findings: &[Finding]) -> Self {
        let total = findings.len() as u32;
        let passed = findings
            .iter()
            .filter(|f| f.status == FindingStatus::Pass)
            .count() as u32;
        let failed = findings
            .iter()
            .filter(|f| f.status == FindingStatus::Fail)
            .count() as u32;
        let warned = findings
            .iter()
            .filter(|f| f.status == FindingStatus::Warn)
            .count() as u32;
        let score = (passed * 100).checked_div(total).unwrap_or(0) as u8;
        Self {
            grade: Self::grade_from_score(score),
            score,
            controls_passed: passed,
            controls_failed: failed,
            controls_warned: warned,
            controls_total: total,
        }
    }
}
