//! ############################################################################
//! @file       report.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      qxscan report subcommand — render, list, show, and remove stored reports.
//!
//! @details    Part of the qxscan CLI layer. Implements the report.rs command
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
//! `qxscan report` subcommand — render a stored report or list stored reports.
//!
//! Subcommands:
//!   render  Read a QEM JSON file and render it in the requested output format
//!   list    List stored reports from the database

use clap::{Args, Subcommand};
use sqlx::SqlitePool;
use tokio::runtime::Runtime;

use crate::qem::metadata::ScanEvent;
use crate::server::state;
use crate::server::Config;

#[derive(Args)]
pub struct ReportArgs {
    #[command(subcommand)]
    pub action: ReportAction,
}

#[derive(Subcommand)]
pub enum ReportAction {
    /// Render a scan report from a QEM file as terminal/JSON/HTML
    Render {
        /// Input report file (QEM JSON)
        #[arg(long)]
        from: std::path::PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "terminal")]
        format: ReportFormat,

        /// Output file path (defaults to stdout for terminal/json)
        #[arg(long)]
        out: Option<std::path::PathBuf>,
    },
    /// List stored reports from the database
    List {
        /// Filter by scan ID
        #[arg(long)]
        scan_id: Option<String>,

        /// Filter by format (json, html, terminal)
        #[arg(long)]
        format: Option<String>,

        /// Maximum number of reports to show
        #[arg(long, default_value_t = 20)]
        limit: usize,

        /// Config file path
        #[arg(long, default_value = "qxscan.toml")]
        config: std::path::PathBuf,
    },
    /// Show the full content of a stored report by ID
    Show {
        /// Report ID (full UUID or short prefix from `list`)
        id: String,

        /// Output file path (defaults to stdout)
        #[arg(long)]
        out: Option<std::path::PathBuf>,

        /// Config file path
        #[arg(long, default_value = "qxscan.toml")]
        config: std::path::PathBuf,
    },
    /// Remove a stored report by ID
    Remove {
        /// Report ID to remove
        id: String,

        /// Config file path
        #[arg(long, default_value = "qxscan.toml")]
        config: std::path::PathBuf,
    },
}

#[derive(clap::ValueEnum, Clone)]
pub enum ReportFormat {
    /// Coloured TTY summary
    Terminal,
    /// Pretty-printed QEM JSON
    Json,
    /// Self-contained HTML report
    Html,
}

fn connect(config_path: &std::path::Path) -> anyhow::Result<(SqlitePool, Runtime)> {
    let config = Config::load(config_path)?;
    let rt = Runtime::new()?;
    let pool = rt.block_on(state::init_pool(&config.database.url))?;
    Ok((pool, rt))
}

pub fn run(args: ReportArgs) -> anyhow::Result<u8> {
    match args.action {
        ReportAction::Render { from, format, out } => {
            render_from_file(from, format, out)?;
            Ok(0)
        }
        ReportAction::List {
            scan_id,
            format,
            limit,
            config,
        } => {
            list_reports(config, scan_id.as_deref(), format.as_deref(), limit)?;
            Ok(0)
        }
        ReportAction::Show { id, out, config } => {
            show_report(config, &id, out.as_deref())?;
            Ok(0)
        }
        ReportAction::Remove { id, config } => {
            remove_report(config, &id)?;
            Ok(0)
        }
    }
}

// ─── Render (existing behavior) ──────────────────────────────

fn render_from_file(
    from: std::path::PathBuf,
    format: ReportFormat,
    out: Option<std::path::PathBuf>,
) -> anyhow::Result<()> {
    let json = std::fs::read_to_string(&from)
        .map_err(|e| anyhow::anyhow!("cannot open '{}': {e}", from.display()))?;

    // Parse as array or single event
    let events: Vec<ScanEvent> = serde_json::from_str(&json)
        .or_else(|_| serde_json::from_str::<ScanEvent>(&json).map(|e| vec![e]))
        .map_err(|e| anyhow::anyhow!("invalid scan event JSON in '{}': {e}", from.display()))?;

    if events.is_empty() {
        anyhow::bail!("no scan events found in '{}'", from.display());
    }

    let event = &events[0]; // render the first event

    match format {
        ReportFormat::Json => {
            let rendered = crate::report::json::render(event)?;
            write_output(&rendered, out.as_deref())?;
        }
        ReportFormat::Html => {
            let rendered = crate::report::html::render(event)?;
            write_output(&rendered, out.as_deref())?;
        }
        ReportFormat::Terminal => {
            render_terminal(event, out.as_deref())?;
        }
    }

    Ok(())
}

// ─── List reports (new) ──────────────────────────────────────

fn list_reports(
    config: std::path::PathBuf,
    scan_id: Option<&str>,
    format: Option<&str>,
    limit: usize,
) -> anyhow::Result<()> {
    let (pool, rt) = connect(&config)?;
    let reports = rt.block_on(state::get_scan_reports(&pool, scan_id, format, limit))?;

    if reports.is_empty() {
        println!("no stored reports found");
        return Ok(());
    }

    println!(
        "  {:<8} {:<8} {:<6} {:<16} {:<37} File Path",
        "Report ID", "Scan ID", "Format", "Created", "Content Preview"
    );
    println!(
        "  {:-<8} {:-<8} {:-<6} {:-<16} {:-<37} ----------",
        "", "", "", "", ""
    );

    for r in &reports {
        let report_id = r.id.to_string();
        let scan_id = r.scan_id.to_string();
        let report_short = report_id.split('-').next().unwrap_or("?");
        let scan_short = scan_id.split('-').next().unwrap_or("?");
        let created = r.created_at.format("%Y-%m-%d %H:%M");
        let preview = content_preview(&r.content);
        let file_path = r.file_path.as_deref().unwrap_or("-");
        println!(
            "  {:<8} {:<8} {:<6} {:<16} {:<37} {}",
            report_short, scan_short, r.format, created, preview, file_path
        );
    }

    Ok(())
}

// ─── Show report by ID (new) ────────────────────────────────────

fn show_report(
    config: std::path::PathBuf,
    id: &str,
    out: Option<&std::path::Path>,
) -> anyhow::Result<()> {
    let (pool, rt) = connect(&config)?;
    let report = rt.block_on(state::get_scan_report_by_id(&pool, id))?;

    match report {
        None => anyhow::bail!("report with id '{}' not found", id),
        Some(r) => {
            if let Some(path) = out {
                std::fs::write(path, &r.content)?;
                eprintln!("wrote report to {}", path.display());
            } else {
                print!("{}", r.content);
                // Ensure trailing newline
                if !r.content.ends_with('\n') {
                    println!();
                }
            }
        }
    }
    Ok(())
}

// ─── Remove report by ID (new) ──────────────────────────────────

fn remove_report(config: std::path::PathBuf, id: &str) -> anyhow::Result<()> {
    let (pool, rt) = connect(&config)?;
    let deleted = rt.block_on(state::delete_scan_report(&pool, id))?;

    if deleted {
        println!("report {id} removed");
    } else {
        anyhow::bail!("report with id '{}' not found", id);
    }
    Ok(())
}

/// Extract a short content preview from a stored report.
fn content_preview(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.len() <= 34 {
        return trimmed.to_string();
    }
    // For JSON/HTML, show a structural summary instead of raw content
    if trimmed.starts_with('{') {
        format!("{} bytes JSON", content.len())
    } else if trimmed.starts_with('<') {
        format!("{} bytes HTML", content.len())
    } else {
        let preview: String = trimmed.chars().take(31).collect();
        format!("{preview}...")
    }
}

// ─── Shared utilities ────────────────────────────────────────

fn write_output(content: &str, path: Option<&std::path::Path>) -> anyhow::Result<()> {
    if let Some(p) = path {
        std::fs::write(p, content)?;
    } else {
        print!("{content}");
    }
    Ok(())
}

fn render_terminal(event: &ScanEvent, path: Option<&std::path::Path>) -> anyhow::Result<()> {
    let rendered = crate::report::terminal::render(event);
    write_output(&rendered, path)
}
