//! ############################################################################
//! @file       about.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-27
//! @brief      Canonical product identity — single source of truth for all
//!             exporters, reports, and CLI output.
//!
//! @details    Every exporter, every report, every HTML page, every JSON,
//!             every CEF, every OCSF record uses exactly the same metadata
//!             by importing from this module.
//!
//! ### REVISION HISTORY
//! | Date       | Version | Author                  | Description |
//! |------------|---------|-------------------------|-------------|
//! | 2026-06-27 | 1.0.0   | Phaneendra Bhattiprolu  | Initial. |
//! |            |         |                         |             |
//!
//! ### COMMENTS / NOTES
//! ############################################################################

/// Product name used in CLI banners, report titles, and exporter metadata.
pub const PRODUCT: &str = "QXScan";

/// QEM — QX Event Model. The canonical observation format at the heart
/// of QXScan. Referenced by exporters as the source schema.
pub const ENGINE: &str = "QEM";

/// Current QEM schema version. Must be bumped on breaking changes
/// to the ScanEvent struct or any of its sub-types.
pub const SCHEMA: &str = "1";

/// Vendor string for CEF, syslog, and other SIEM formats.
pub const VENDOR: &str = "QuantX";

/// Build version from Cargo.toml, embedded at compile time.
pub const BUILD: &str = env!("CARGO_PKG_VERSION");
