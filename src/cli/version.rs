//! ############################################################################
//! @file       version.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      qxscan version subcommand — print version and build metadata.
//!
//! @details    Part of the qxscan CLI layer. Implements the version.rs command
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
//! `qxscan version` subcommand — print version and build metadata.

use clap::Args;

#[derive(Args)]
pub struct VersionArgs;

pub fn run(_args: VersionArgs) -> anyhow::Result<u8> {
    println!("{} {}", crate::about::PRODUCT, crate::about::BUILD);
    println!("engine: {}", crate::about::ENGINE);
    println!("schema: {}", crate::about::SCHEMA);
    println!("license: {}", env!("CARGO_PKG_LICENSE"));
    println!("edition: 2021");
    Ok(0)
}
