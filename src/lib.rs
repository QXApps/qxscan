//! ############################################################################
//! @file       lib.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-27
//! @brief      Library root for qxscan — exposes all public modules for
//!             the binary entry point and for integration tests.
//!
//! @details    Splitting module declarations into lib.rs enables integration
//!             tests in tests/ to import qxscan::*. main.rs remains a thin
//!             wrapper (< 20 lines) per the module contract in CLAUDE.md.
//!
//! ### REVISION HISTORY
//! | Date       | Version | Author                  | Description |
//! |------------|---------|-------------------------|-------------|
//! | 2026-06-27 | 1.0.0   | Phaneendra Bhattiprolu  | Extracted from main.rs. |
//! |            |         |                         |             |
//!
//! ### COMMENTS / NOTES
//! ############################################################################

#![allow(clippy::doc_lazy_continuation)]

pub mod about;
pub mod cli;
pub mod compliance;
pub mod export;
pub mod qem;
pub mod report;
pub mod scanner;
pub mod schedule;
pub mod server;
pub mod tls;
