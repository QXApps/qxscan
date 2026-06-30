//! ############################################################################
//! @file       json.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      JSON report renderer — pretty-printed QEM JSON output.
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
//! JSON report renderer — serialises ScanEvent to pretty-printed JSON.

use crate::qem::metadata::ScanEvent;

pub fn render(event: &ScanEvent) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(event)?)
}

#[allow(dead_code)]
pub fn write(event: &ScanEvent, path: &std::path::Path) -> anyhow::Result<()> {
    let json = render(event)?;
    std::fs::write(path, json)?;
    Ok(())
}
