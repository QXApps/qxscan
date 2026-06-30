//! ############################################################################
//! @file       schedule.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      qxscan schedule subcommand — periodic scan schedule management.
//!
//! @details    Part of the qxscan CLI layer. Implements the schedule.rs command
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
//! `qxscan schedule` subcommand — periodic scan management.

use chrono::Utc;
use clap::{Args, Subcommand};
use sqlx::SqlitePool;
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::server::state::{self, Schedule};
use crate::server::Config;

#[derive(Args)]
pub struct ScheduleArgs {
    #[command(subcommand)]
    pub action: ScheduleAction,
}

#[derive(Subcommand)]
pub enum ScheduleAction {
    /// Add a new schedule
    Add {
        /// Targets file
        #[arg(long)]
        targets_file: std::path::PathBuf,

        /// Named interval
        #[arg(long, value_enum, conflicts_with = "cron")]
        interval: Option<Interval>,

        /// Cron expression (e.g. "0 2 * * *")
        #[arg(long, conflicts_with = "interval")]
        cron: Option<String>,

        /// Compliance standards to evaluate
        #[arg(long, value_delimiter = ',')]
        standards: Vec<String>,

        /// Human-readable label for this schedule
        #[arg(long)]
        label: Option<String>,

        /// Config file path
        #[arg(long, default_value = "qxscan.toml")]
        config: std::path::PathBuf,
    },
    /// List active schedules
    List {
        /// Config file path
        #[arg(long, default_value = "qxscan.toml")]
        config: std::path::PathBuf,
    },
    /// Remove a schedule by ID
    Remove {
        /// Schedule ID to remove
        id: String,
        /// Config file path
        #[arg(long, default_value = "qxscan.toml")]
        config: std::path::PathBuf,
    },
    /// Show next N scheduled runs
    Preview {
        #[arg(long, default_value_t = 5)]
        count: usize,
    },
}

#[derive(clap::ValueEnum, Clone)]
pub enum Interval {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

fn connect(config_path: &std::path::Path) -> anyhow::Result<(SqlitePool, Runtime)> {
    let config = Config::load(config_path)?;
    let rt = Runtime::new()?;
    let pool = rt.block_on(state::init_pool(&config.database.url))?;
    Ok((pool, rt))
}

pub fn run(args: ScheduleArgs) -> anyhow::Result<u8> {
    match args.action {
        ScheduleAction::Add {
            targets_file,
            interval,
            cron,
            standards,
            label,
            config,
        } => {
            let (pool, rt) = connect(&config)?;

            let cron_expr = match (interval, cron) {
                (Some(iv), None) => crate::schedule::cron::interval_to_cron(&iv).to_string(),
                (None, Some(c)) => c,
                (None, None) => anyhow::bail!("provide either --interval or --cron"),
                (Some(_), Some(_)) => unreachable!(),
            };

            let standards = if standards.is_empty() {
                vec!["pci-dss".to_string()]
            } else {
                standards
            };

            let next_run = crate::schedule::cron::next_run_from_cron(&cron_expr, &Utc::now())?;

            let schedule = Schedule {
                id: Uuid::new_v4(),
                label,
                targets_file,
                cron_expr: cron_expr.clone(),
                standards,
                last_run_at: None,
                next_run_at: next_run,
                enabled: true,
            };

            rt.block_on(state::insert_schedule(&pool, &schedule))?;
            println!(
                "schedule added: {} (cron: {cron_expr}, next: {})",
                schedule.id,
                schedule.next_run_at.format("%Y-%m-%d %H:%M UTC")
            );
            Ok(0)
        }
        ScheduleAction::List { config } => {
            let (pool, rt) = connect(&config)?;
            let schedules = rt.block_on(state::list_schedules(&pool))?;
            if schedules.is_empty() {
                println!("no schedules configured");
                return Ok(0);
            }
            for s in &schedules {
                let label = s.label.as_deref().unwrap_or("unnamed");
                let last = s
                    .last_run_at
                    .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "never".into());
                let next = s.next_run_at.format("%Y-%m-%d %H:%M UTC");
                let status = if s.enabled { "enabled" } else { "disabled" };
                println!(
                    "{}  {}  targets: {}  cron: {}  last: {last}  next: {next}  {status}",
                    s.id.to_string().split('-').next().unwrap_or("?"),
                    label,
                    s.targets_file.display(),
                    s.cron_expr,
                );
            }
            Ok(0)
        }
        ScheduleAction::Remove { id, config } => {
            let (pool, rt) = connect(&config)?;
            if rt.block_on(state::delete_schedule(&pool, &id))? {
                println!("schedule {id} removed");
            } else {
                anyhow::bail!("schedule {id} not found");
            }
            Ok(0)
        }
        ScheduleAction::Preview { count } => {
            let (pool, rt) = connect(std::path::Path::new("qxscan.toml"))?;
            let schedules = rt.block_on(state::list_schedules(&pool))?;
            if schedules.is_empty() {
                println!("no schedules configured");
                return Ok(0);
            }
            let mut now = Utc::now();
            for s in &schedules {
                println!(
                    "{} ({})",
                    s.id.to_string().split('-').next().unwrap_or("?"),
                    s.label.as_deref().unwrap_or("unnamed")
                );
                for i in 0..count {
                    match crate::schedule::cron::next_run_from_cron(&s.cron_expr, &now) {
                        Ok(next) => {
                            println!("  {}: {}", i + 1, next.format("%Y-%m-%d %H:%M UTC"));
                            now = next + chrono::Duration::minutes(1);
                        }
                        Err(e) => println!("  error: {e}"),
                    }
                }
            }
            Ok(0)
        }
    }
}
