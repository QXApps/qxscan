//! ############################################################################
//! @file       cef.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      CEF exporter — Common Event Format for syslog / SIEM ingestion.
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
use std::io::Write;

use chrono::Utc;

use crate::about::{PRODUCT, VENDOR};
use crate::export::Exporter;
use crate::qem::finding::Finding;
use crate::qem::metadata::ScanEvent;

pub struct CefExporter;

const CEF_VERSION: &str = "CEF:0";

impl Exporter for CefExporter {
    fn export(&self, events: &[ScanEvent], writer: &mut dyn Write) -> anyhow::Result<()> {
        for event in events {
            for finding in &event.findings {
                writeln!(writer, "{}", to_cef(event, finding))?;
            }
        }
        Ok(())
    }
}

fn to_cef(event: &ScanEvent, finding: &Finding) -> String {
    let version = crate::about::BUILD;
    let severity = cef_severity(&finding.severity);
    let ts = Utc::now().format("%b %d %H:%M:%S").to_string();
    let status = match finding.status {
        crate::qem::finding::FindingStatus::Pass => "PASS",
        crate::qem::finding::FindingStatus::Fail => "FAIL",
        crate::qem::finding::FindingStatus::Warn => "WARN",
        crate::qem::finding::FindingStatus::NotApplicable => "N/A",
    };

    format!(
        "{ts} {CEF_VERSION}|{VENDOR}|{PRODUCT}|{version}|{control_id}|{title}|{severity}|\
         src={host} dpt={port} cs1={standard} cs1Label=Standard \
         cs2={status} cs2Label=Status msg={detail}",
        control_id = finding.control_id,
        title = finding.title,
        host = event.target.host,
        port = event.target.port,
        standard = finding.standard,
        detail = finding.detail,
    )
}

fn cef_severity(sev: &crate::qem::finding::Severity) -> u8 {
    use crate::qem::finding::Severity::*;
    match sev {
        Info => 1,
        Low => 3,
        Medium => 5,
        High => 7,
        Critical => 10,
    }
}
