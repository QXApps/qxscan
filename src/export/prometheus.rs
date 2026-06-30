//! ############################################################################
//! @file       prometheus.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      Prometheus exporter — OpenMetrics text exposition format.
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

use crate::export::Exporter;
use crate::qem::finding::FindingStatus;
use crate::qem::metadata::ScanEvent;

pub struct PrometheusExporter;

impl Exporter for PrometheusExporter {
    fn export(&self, events: &[ScanEvent], writer: &mut dyn Write) -> anyhow::Result<()> {
        for event in events {
            let host = &event.target.host;
            let port = event.target.port;

            writeln!(
                writer,
                "# HELP qxscan_scan_duration_ms Scan duration in milliseconds\n\
                 # TYPE qxscan_scan_duration_ms gauge\n\
                 qxscan_scan_duration_ms{{product=\"{product}\",version=\"{version}\",\
                 host=\"{host}\",port=\"{port}\"}} {}",
                event.scan_duration_ms,
                product = crate::about::PRODUCT,
                version = crate::about::BUILD,
            )?;

            for (standard, score) in &event.compliance {
                writeln!(
                    writer,
                    "# HELP qxscan_compliance_score Compliance score 0-100\n\
                     # TYPE qxscan_compliance_score gauge\n\
                     qxscan_compliance_score{{product=\"{product}\",\
                     host=\"{host}\",port=\"{port}\",standard=\"{standard}\"}} {}",
                    score.score,
                    product = crate::about::PRODUCT,
                )?;
            }

            for status in ["pass", "fail", "warn"] {
                let fs = match status {
                    "pass" => FindingStatus::Pass,
                    "fail" => FindingStatus::Fail,
                    _ => FindingStatus::Warn,
                };
                for standard in event.compliance.keys() {
                    let count = event
                        .findings
                        .iter()
                        .filter(|f| &f.standard == standard && f.status == fs)
                        .count();
                    writeln!(
                        writer,
                        "qxscan_finding_total{{product=\"{product}\",host=\"{host}\",port=\"{port}\",\
                         standard=\"{standard}\",status=\"{status}\"}} {count}",
                        product = crate::about::PRODUCT,
                    )?;
                }
            }
        }
        Ok(())
    }
}
