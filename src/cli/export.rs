//! ############################################################################
//! @file       export.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      qxscan export subcommand — emit QEM scan results in open observability formats.
//!
//! @details    Part of the qxscan CLI layer. Implements the export.rs command
//!             handler using clap for argument parsing and delegates to
//!             the appropriate library modules for business logic.
//!
//! ### REVISION HISTORY
//! | Date       | Version | Author                  | Description |
//! |------------|---------|-------------------------|-------------|
//! | 2026-06-02 | 1.0.0   | Phaneendra Bhattiprolu  | Initial implementation. |
//! |            |         |                         |             |
//!
//! ### COMMENTS / NOTES
//! * Only module allowed to import clap; delegates to library modules.
//! ############################################################################
//! `qxscan export` subcommand — export a report to an observability format.

use clap::Args;

use crate::export::{
    cef::CefExporter, ocsf::OcsfExporter, prometheus::PrometheusExporter, qem::QemExporter,
    Exporter,
};
use crate::qem::metadata::ScanEvent;

#[derive(Args)]
pub struct ExportArgs {
    /// Input report file (JSON)
    #[arg(long)]
    pub from: std::path::PathBuf,

    /// Export format / sink
    #[arg(long, value_enum)]
    pub format: ExportFormat,

    /// Output file path (for file-based formats)
    #[arg(long)]
    pub out: Option<std::path::PathBuf>,

    /// Endpoint URL (for push-based sinks: prometheus-pushgw, otlp, elastic, splunk, datadog)
    #[arg(long)]
    pub url: Option<String>,
}

#[derive(clap::ValueEnum, Clone)]
pub enum ExportFormat {
    /// Canonical QXScan Event Format (versioned JSON)
    Qef,
    /// Prometheus text exposition format (for scraping or Pushgateway)
    Prometheus,
    /// OCSF — Open Cybersecurity Schema Framework (class 2004)
    Ocsf,
    /// CEF — Common Event Format (syslog / SIEM universal)
    Cef,
}

pub fn run(args: ExportArgs) -> anyhow::Result<u8> {
    let json = std::fs::read_to_string(&args.from)
        .map_err(|e| anyhow::anyhow!("cannot open '{}': {e}", args.from.display()))?;

    let events: Vec<ScanEvent> = serde_json::from_str(&json)
        .or_else(|_| serde_json::from_str::<ScanEvent>(&json).map(|e| vec![e]))
        .map_err(|e| {
            anyhow::anyhow!("invalid scan event JSON in '{}': {e}", args.from.display())
        })?;

    let exporter: Box<dyn Exporter> = match args.format {
        ExportFormat::Qef => Box::new(QemExporter),
        ExportFormat::Prometheus => Box::new(PrometheusExporter),
        ExportFormat::Ocsf => Box::new(OcsfExporter),
        ExportFormat::Cef => Box::new(CefExporter),
    };

    let original_stdout = std::io::stdout();
    let mut output: Box<dyn std::io::Write> = if let Some(out) = &args.out {
        Box::new(std::fs::File::create(out)?)
    } else {
        Box::new(original_stdout.lock())
    };

    exporter.export(&events, &mut output)?;
    Ok(0)
}
