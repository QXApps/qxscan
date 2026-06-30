//! ############################################################################
//! @file       ocsf.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      OCSF exporter — OCSF class 2004 Vulnerability Finding format.
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

use serde_json::{json, Value};

use crate::export::Exporter;
use crate::qem::finding::Finding;
use crate::qem::metadata::ScanEvent;

pub struct OcsfExporter;

impl Exporter for OcsfExporter {
    fn export(&self, events: &[ScanEvent], writer: &mut dyn Write) -> anyhow::Result<()> {
        for event in events {
            for finding in &event.findings {
                let record = to_ocsf(event, finding);
                writeln!(writer, "{}", serde_json::to_string(&record)?)?;
            }
        }
        Ok(())
    }
}

fn to_ocsf(event: &ScanEvent, finding: &Finding) -> Value {
    json!({
        "class_uid":    2004,
        "class_name":   "Vulnerability Finding",
        "time":         event.scanned_at.timestamp_millis(),
        "activity_id":  1,
        "severity_id":  severity_id(&finding.severity),
        "status":       ocsf_status(&finding.status),
        "message":      finding.title,
        "metadata": {
            "version":   "1.0.0",
            "product": {
                "name":    crate::about::PRODUCT,
                "version": crate::about::BUILD,
            }
        },
        "finding_info": {
            "uid":         finding.control_id,
            "title":       finding.title,
            "desc":        finding.detail,
            "remediation": finding.remediation,
            "related_events": [{
                "product": finding.standard,
            }]
        },
        "resource": {
            "hostname": event.target.host,
            "ip":       event.target.ip,
            "port":     event.target.port,
            "type":     event.target.service,
        },
        "scan": {
            "uid":  event.scan_id,
            "name": crate::about::PRODUCT,
        }
    })
}

fn severity_id(sev: &crate::qem::finding::Severity) -> u8 {
    use crate::qem::finding::Severity::*;
    match sev {
        Info => 1,
        Low => 2,
        Medium => 3,
        High => 4,
        Critical => 5,
    }
}

fn ocsf_status(status: &crate::qem::finding::FindingStatus) -> &'static str {
    use crate::qem::finding::FindingStatus::*;
    match status {
        Pass => "Pass",
        Fail => "Fail",
        Warn => "Warn",
        NotApplicable => "Not Applicable",
    }
}
