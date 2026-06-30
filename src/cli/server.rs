//! ############################################################################
//! @file       server.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      qxscan server subcommand — daemon lifecycle management (start | stop | restart | status).
//!
//! @details    Part of the qxscan CLI layer. Implements the server.rs command
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
//! `qxscan server` subcommand — daemon lifecycle.

use clap::{Args, Subcommand};

use crate::server::Config;

#[derive(Args)]
pub struct ServerArgs {
    #[command(subcommand)]
    pub action: ServerAction,
}

#[derive(Subcommand)]
pub enum ServerAction {
    /// Start the qxscan daemon
    Start {
        /// Config file path
        #[arg(long, default_value = "qxscan.toml")]
        config: std::path::PathBuf,
    },
    /// Stop the running daemon
    Stop {
        /// Config file path
        #[arg(long, default_value = "qxscan.toml")]
        config: std::path::PathBuf,
    },
    /// Restart the daemon (reload config)
    Restart {
        /// Config file path
        #[arg(long, default_value = "qxscan.toml")]
        config: std::path::PathBuf,
    },
    /// Show daemon status (PID, uptime, next scheduled scan)
    Status {
        /// Config file path
        #[arg(long, default_value = "qxscan.toml")]
        config: std::path::PathBuf,
    },
}

pub fn run(args: ServerArgs) -> anyhow::Result<u8> {
    match args.action {
        ServerAction::Start { config } => {
            let config = Config::load(&config)?;
            let msg = crate::server::start_daemon(&config)?;
            println!("{msg}");
            Ok(0)
        }
        ServerAction::Stop { config } => {
            let config = Config::load(&config)?;
            let msg = crate::server::stop_daemon(&config)?;
            println!("{msg}");
            Ok(0)
        }
        ServerAction::Restart { config } => {
            let config = Config::load(&config)?;
            let _ = crate::server::stop_daemon(&config)?;
            let msg = crate::server::start_daemon(&config)?;
            println!("{msg}");
            Ok(0)
        }
        ServerAction::Status { config } => {
            let config = Config::load(&config)?;
            let status = crate::server::daemon_status(&config)?;
            print!("{status}");
            Ok(0)
        }
    }
}
