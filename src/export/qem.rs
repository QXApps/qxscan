//! ############################################################################
//! @file       qem.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-27
//! @brief      QEM exporter — canonical QEM JSON passthrough (one event per line).
//!
//! @details
//!
//! ### REVISION HISTORY
//! | Date       | Version | Author                  | Description |
//! |------------|---------|-------------------------|-------------|
//! | 2026-06-27 | 1.0.0   | Phaneendra Bhattiprolu  | Extracted from cli/export.rs inline passthrough. |
//! |            |         |                         |             |
//!
//! ### COMMENTS / NOTES
//! ############################################################################
//! QEM (QX Event Model) canonical exporter.
//! Writes each ScanEvent as a single-line JSON record.
//! This is the reference format — all other exporters convert from QEM.

use std::io::Write;

use crate::export::Exporter;
use crate::qem::metadata::ScanEvent;

pub struct QemExporter;

impl Exporter for QemExporter {
    fn export(&self, events: &[ScanEvent], writer: &mut dyn Write) -> anyhow::Result<()> {
        for event in events {
            writeln!(writer, "{}", serde_json::to_string(event)?)?;
        }
        Ok(())
    }
}
