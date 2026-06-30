//! ############################################################################
//! @file       html.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      HTML report renderer — self-contained HTML report with styling.
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
//! HTML report renderer.
//! Generates a self-contained HTML file from a ScanEvent.
//! Intentionally minimal — replace with a Tera template once the
//! schema is stable.

use crate::qem::finding::FindingStatus;
use crate::qem::metadata::ScanEvent;

pub fn render(event: &ScanEvent) -> anyhow::Result<String> {
    let mut rows = String::new();
    for f in &event.findings {
        let status_class = match f.status {
            FindingStatus::Pass => "pass",
            FindingStatus::Fail => "fail",
            FindingStatus::Warn => "warn",
            FindingStatus::NotApplicable => "na",
        };
        rows.push_str(&format!(
            "<tr class=\"{status_class}\"><td>{}</td><td>{}</td>\
             <td>{:?}</td><td>{}</td><td>{}</td></tr>\n",
            f.control_id, f.standard, f.severity, f.title, f.detail
        ));
    }

    Ok(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{product} Report — {host}</title>
<style>
  body  {{ font-family: system-ui, sans-serif; max-width: 960px; margin: 2rem auto; }}
  h1   {{ color: #1a1a2e; }}
  table{{ border-collapse: collapse; width: 100%; }}
  th,td{{ border: 1px solid #ddd; padding: 8px 12px; text-align: left; }}
  th   {{ background: #f4f4f4; }}
  tr.pass {{ background: #f0fff4; }}
  tr.fail {{ background: #fff5f5; }}
  tr.warn {{ background: #fffbeb; }}
  .score  {{ font-size: 2rem; font-weight: bold; }}
</style>
</head>
<body>
<h1>{product} Report</h1>
<p><strong>Target:</strong> {host}:{port} ({service})</p>
<p><strong>Scan ID:</strong> {scan_id}</p>
<p><strong>Scanned at:</strong> {scanned_at}</p>
<p><strong>Duration:</strong> {duration_ms}ms</p>

<h2>Compliance Scores</h2>
<table>
<tr><th>Standard</th><th>Score</th><th>Grade</th><th>Passed</th><th>Failed</th><th>Total</th></tr>
{score_rows}
</table>

<h2>Findings ({finding_count})</h2>
<table>
<tr><th>Control</th><th>Standard</th><th>Severity</th><th>Title</th><th>Detail</th></tr>
{rows}
</table>
</body>
</html>"#,
        product = crate::about::PRODUCT,
        host = event.target.host,
        port = event.target.port,
        service = event.target.service,
        scan_id = event.scan_id,
        scanned_at = event.scanned_at,
        duration_ms = event.scan_duration_ms,
        finding_count = event.findings.len(),
        score_rows = render_score_rows(event),
        rows = rows,
    ))
}

fn render_score_rows(event: &ScanEvent) -> String {
    event
        .compliance
        .iter()
        .map(|(standard, score)| {
            format!(
            "<tr><td>{standard}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            score.score, score.grade,
            score.controls_passed, score.controls_failed, score.controls_total
        )
        })
        .collect()
}

#[allow(dead_code)]
pub fn write(event: &ScanEvent, path: &std::path::Path) -> anyhow::Result<()> {
    std::fs::write(path, render(event)?)?;
    Ok(())
}
