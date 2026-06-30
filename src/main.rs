#![allow(clippy::doc_lazy_continuation)]

//! ############################################################################
//! @file       main.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      CLI entry point for qxscan — On-Prem Security & Compliance Scanner.
//!
//! @details    Thin router — delegates to lib.rs modules. Under 20 lines,
//!             no business logic per the module contract in CLAUDE.md.
//!
//! ### REVISION HISTORY
//! | Date       | Version | Author                  | Description |
//! |------------|---------|-------------------------|-------------|
//! | 2026-06-02 | 1.0.0   | Phaneendra Bhattiprolu  | Initial implementation. |
//! | 2026-06-27 | 1.0.1   | Phaneendra Bhattiprolu  | Moved modules to lib.rs for integration test support. |
//! |            |         |                         |             |
//!
//! ### COMMENTS / NOTES
//! * All module declarations now live in src/lib.rs.
//! ############################################################################

use clap::Parser;
use qxscan::cli::Cli;

fn main() {
    let cli = Cli::parse();

    if cli.verbose || cli.daemon {
        std::env::set_var("RUST_LOG", if cli.verbose { "debug" } else { "info" });
    }
    env_logger::init();

    match qxscan::cli::dispatch(cli) {
        Ok(code) => std::process::exit(code as i32),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
