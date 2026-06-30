//! ############################################################################
//! @file       mod.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      CLI definitions, argument parsing, and top-level dispatch.
//!
//! @details    Part of the qxscan CLI layer. Implements the mod.rs command
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
//! CLI definitions (clap) and top-level dispatch.
//! No business logic here — only argument parsing and handler calls.

pub mod export;
pub mod report;
pub mod scan;
pub mod schedule;
pub mod server;
pub mod version;

use clap::{Parser, Subcommand};

// Manual ValueEnum impl for ServiceType so scanner/service.rs stays clap-free.
// This is the only place where ServiceType ↔ CLI string conversion lives.
impl clap::ValueEnum for crate::scanner::service::ServiceType {
    fn value_variants<'a, 'b>() -> &'a [Self] {
        &[
            Self::Https,
            Self::Smtp,
            Self::Imap,
            Self::Pop3,
            Self::Postgres,
            Self::Mysql,
            Self::Ldap,
            Self::Ftp,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Https => clap::builder::PossibleValue::new("https"),
            Self::Smtp => clap::builder::PossibleValue::new("smtp"),
            Self::Imap => clap::builder::PossibleValue::new("imap"),
            Self::Pop3 => clap::builder::PossibleValue::new("pop3"),
            Self::Postgres => clap::builder::PossibleValue::new("postgres"),
            Self::Mysql => clap::builder::PossibleValue::new("mysql"),
            Self::Ldap => clap::builder::PossibleValue::new("ldap"),
            Self::Ftp => clap::builder::PossibleValue::new("ftp"),
        })
    }
}

#[derive(Parser)]
#[command(
    name = "qxscan",
    about = "On-Prem Security & Compliance Scanner",
    version,
    propagate_version = true
)]
pub struct Cli {
    /// Enable verbose debug output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Config file path (used by daemon mode)
    #[arg(long, global = true)]
    pub config: Option<std::path::PathBuf>,

    /// Internal: run as daemon (spawned by `server start`)
    #[arg(long, global = true, hide = true)]
    pub daemon: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan a target for TLS posture and compliance
    Scan(scan::ScanArgs),

    /// Manage the qxscan daemon (start | stop | restart | status)
    Server(server::ServerArgs),

    /// Schedule periodic scans (daily, weekly, monthly, hourly, or cron)
    Schedule(schedule::ScheduleArgs),

    /// Export a scan report to an observability format
    Export(export::ExportArgs),

    /// Render a stored report or list stored reports
    Report(report::ReportArgs),

    /// Print version and build metadata
    Version(version::VersionArgs),
}

pub fn dispatch(cli: Cli) -> anyhow::Result<u8> {
    if cli.daemon {
        let config_path = cli
            .config
            .as_deref()
            .unwrap_or(std::path::Path::new("qxscan.toml"));
        let config = crate::server::Config::load(config_path)?;
        return crate::server::run_daemon_foreground(&config);
    }

    match cli.command {
        Some(Commands::Scan(args)) => {
            scan::run(args, cli.verbose, cli.quiet, cli.config.as_deref())
        }
        Some(Commands::Server(args)) => server::run(args),
        Some(Commands::Schedule(args)) => schedule::run(args),
        Some(Commands::Export(args)) => export::run(args),
        Some(Commands::Report(args)) => report::run(args),
        Some(Commands::Version(args)) => version::run(args),
        None => {
            use clap::CommandFactory;
            Cli::command().print_help()?;
            Ok(0)
        }
    }
}
