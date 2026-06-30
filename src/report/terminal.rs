//! ############################################################################
//! @file       terminal.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-27
//! @brief      Terminal report renderer — coloured TTY output with status icons.
//!
//! @details
//!
//! ### REVISION HISTORY
//! | Date       | Version | Author                  | Description |
//! |------------|---------|-------------------------|-------------|
//! | 2026-06-27 | 1.0.0   | Phaneendra Bhattiprolu  | Extracted from cli/report.rs. |
//! |            |         |                         |             |
//!
//! ### COMMENTS / NOTES
//! ############################################################################
//! Terminal report renderer.
//! Renders a ScanEvent as a human-readable TTY summary with
//! compliance scores and individual findings.
//!
//! NOTE: All .unwrap() calls below are on writeln!(buf, ...) where
//! buf is a String — writing to a String is infallible in Rust.
//! These unwraps can never panic.
#![allow(clippy::unwrap_used)]

use std::fmt::Write;

use crate::qem::finding::FindingStatus;
use crate::qem::metadata::{ScanEvent, ScanStatus};

/// Render a ScanEvent as a terminal-formatted string.
pub fn render(event: &ScanEvent) -> String {
    let mut buf = String::new();

    writeln!(buf, "═══ {} Report ═══", crate::about::PRODUCT).unwrap();
    writeln!(buf).unwrap();
    writeln!(
        buf,
        "  Target:    {}:{} ({})",
        event.target.host, event.target.port, event.target.service
    )
    .unwrap();
    writeln!(buf, "  Scan ID:   {}", event.scan_id).unwrap();
    writeln!(
        buf,
        "  Time:      {}",
        event.scanned_at.format("%Y-%m-%d %H:%M:%S UTC")
    )
    .unwrap();
    writeln!(buf, "  Duration:  {} ms", event.scan_duration_ms).unwrap();
    writeln!(buf, "  Status:    {}", status_str(&event.overall_status)).unwrap();
    writeln!(buf).unwrap();

    // TLS info
    if let Some(ref tls) = event.tls {
        writeln!(buf, "  TLS:").unwrap();
        writeln!(buf, "    Version:         {}", tls.negotiated_version).unwrap();
        writeln!(buf, "    Cipher:          {}", tls.cipher).unwrap();
        writeln!(buf, "    Forward Secrecy: {}", tls.forward_secrecy).unwrap();
        writeln!(buf, "    PQC Hybrid:      {}", tls.pqc_hybrid).unwrap();
        writeln!(buf).unwrap();
    }

    // Compliance scores
    if !event.compliance.is_empty() {
        writeln!(buf, "  Compliance Scores:").unwrap();
        writeln!(
            buf,
            "  {:<12} {:>6} {:>5} {:>7} {:>7} {:>6}",
            "Standard", "Score", "Grade", "Passed", "Failed", "Total"
        )
        .unwrap();
        writeln!(
            buf,
            "  {:-<12} {:>6} {:>5} {:>7} {:>7} {:>6}",
            "", "", "", "", "", ""
        )
        .unwrap();
        for (standard, score) in &event.compliance {
            writeln!(
                buf,
                "  {:<12} {:>6} {:>5} {:>7} {:>7} {:>6}",
                standard,
                score.score,
                score.grade,
                score.controls_passed,
                score.controls_failed,
                score.controls_total
            )
            .unwrap();
        }
        writeln!(buf).unwrap();
    }

    // Findings
    if !event.findings.is_empty() {
        writeln!(buf, "  Findings ({}):", event.findings.len()).unwrap();
        for f in &event.findings {
            let icon = match f.status {
                FindingStatus::Pass => "✅",
                FindingStatus::Fail => "❌",
                FindingStatus::Warn => "⚠️",
                FindingStatus::NotApplicable => "➖",
            };
            let severity_label = format!("{:?}", f.severity);
            writeln!(
                buf,
                "    {icon} [{severity_label}] {} — {}",
                f.control_id, f.title
            )
            .unwrap();
            writeln!(buf, "       {}", f.detail).unwrap();
            if let Some(ref remediation) = f.remediation {
                writeln!(buf, "       Fix: {remediation}").unwrap();
            }
        }
    }

    buf
}

fn status_str(status: &ScanStatus) -> &'static str {
    match status {
        ScanStatus::Pass => "✅ Pass",
        ScanStatus::Fail => "❌ Fail",
        ScanStatus::Warn => "⚠️ Warn",
        ScanStatus::Error => "❌ Error",
        ScanStatus::Timeout => "⏰ Timeout",
        ScanStatus::NoTls => "ℹ️ No TLS",
        ScanStatus::UnsupportedProtocol => "❌ Unsupported Protocol",
        ScanStatus::ConnectionFailed => "❌ Connection Failed",
    }
}
