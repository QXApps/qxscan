//! ############################################################################
//! @file       pid.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      PID file management — write, read, and remove daemon PID files.
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
//! PID file management for the qxscan daemon.

use std::path::Path;

pub fn write_pid(path: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(path.parent().unwrap_or(Path::new("/tmp")))?;
    std::fs::write(path, std::process::id().to_string())?;
    Ok(())
}

pub fn read_pid(path: &Path) -> anyhow::Result<Option<u32>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(raw.trim().parse().ok())
}

pub fn remove_pid(path: &Path) {
    let _ = std::fs::remove_file(path);
}
