//! ############################################################################
//! @file       mod.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      Export trait and module — pub trait Exporter + OSS format registration.
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
//! Exporter trait and OSS implementations.
//! Enterprise sinks (Elastic, Splunk, Datadog, OTLP) implement this trait
//! behind the commercial license boundary.

pub mod cef;
pub mod ocsf;
pub mod prometheus;
pub mod qem;

use crate::qem::metadata::ScanEvent;

/// Public trait — community and enterprise both implement this.
pub trait Exporter {
    /// Export a batch of scan events to the given writer.
    fn export(&self, events: &[ScanEvent], writer: &mut dyn std::io::Write) -> anyhow::Result<()>;
}
