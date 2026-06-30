//! ############################################################################
//! @file       scan.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      qxscan scan subcommand — target scanning and compliance evaluation.
//!
//! @details    Part of the qxscan CLI layer. Implements the scan.rs command
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
//! `qxscan scan` subcommand arguments and handler.

use std::time::{Duration, Instant};

use clap::Args;

use crate::qem::finding::FindingStatus;
use crate::qem::metadata::{ScanEvent, ScanStatus};

#[derive(Args)]
pub struct ScanArgs {
    /// Target host (hostname, IP, or host:port)
    #[arg(conflicts_with = "targets_file")]
    pub target: Option<String>,

    /// File containing one target per line
    #[arg(long, conflicts_with = "target")]
    pub targets_file: Option<std::path::PathBuf>,

    /// Port override (default: 443)
    #[arg(long, default_value_t = 443)]
    pub port: u16,

    /// Service type preset
    #[arg(long, value_enum, default_value = "https")]
    pub service: crate::scanner::service::ServiceType,

    /// Compliance standards to evaluate (comma-separated)
    /// e.g. pci-dss,hipaa,soc2,fisma,pqc
    #[arg(long, value_delimiter = ',')]
    pub standards: Vec<String>,

    /// Output format
    #[arg(long, value_enum, default_value = "terminal")]
    pub output: OutputFormat,

    /// Path to write the report file (required for json/html output)
    #[arg(long)]
    pub report_file: Option<std::path::PathBuf>,

    /// Connection timeout in seconds
    #[arg(long, default_value_t = 10)]
    pub timeout: u64,

    /// Skip TLS certificate verification
    #[arg(long)]
    pub no_verify: bool,

    /// Number of concurrent scan workers
    #[arg(long, default_value_t = 4)]
    pub concurrency: usize,
}

#[derive(clap::ValueEnum, Clone)]
pub enum OutputFormat {
    Terminal,
    Json,
    Html,
}

/// Run a scan. Returns the process exit code:
///   0 = all controls passed
///   3 = at least one FAIL finding
///   4 = target unreachable / timeout
pub fn run(
    args: ScanArgs,
    verbose: bool,
    quiet: bool,
    config: Option<&std::path::Path>,
) -> anyhow::Result<u8> {
    let start = Instant::now();
    let timeout = Duration::from_secs(args.timeout);
    let targets = crate::scanner::resolve_targets(
        args.target.as_deref(),
        args.targets_file.as_deref(),
        args.port,
        &args.service,
    )?;

    let standards = if args.standards.is_empty() {
        vec!["pci-dss".to_string()]
    } else {
        args.standards.clone()
    };

    // Initialize database connection (best-effort — scan succeeds without DB)
    let db_runtime = match init_db_for_scan(config) {
        Ok((pool, rt)) => Some((pool, rt)),
        Err(e) => {
            if !quiet {
                eprintln!("note: database unavailable, results won't be persisted: {e}");
            }
            None
        }
    };

    let mut events = Vec::new();
    for target in targets {
        if verbose && !quiet {
            eprintln!("[scan] probing {}:{}", target.host, target.port);
        }
        let probed = crate::scanner::probe_target(&target, timeout, args.no_verify);
        let mut event = ScanEvent::new(probed.target.clone());
        event.overall_status = probed.status.clone();
        event.tls = probed.tls;

        if event.overall_status == ScanStatus::Pass {
            for standard in &standards {
                let (findings, score) = crate::compliance::scoring::evaluate(&event, standard)?;
                event.findings.extend(findings);
                event.compliance.insert(standard.replace('-', "_"), score);
            }

            if event
                .findings
                .iter()
                .any(|f| f.status == FindingStatus::Fail)
            {
                event.overall_status = ScanStatus::Fail;
            } else if event
                .findings
                .iter()
                .any(|f| f.status == FindingStatus::Warn)
            {
                event.overall_status = ScanStatus::Warn;
            }
        }

        event.scan_duration_ms = start.elapsed().as_millis() as u64;
        events.push(event);
    }

    // Save results: always to JSON, and to database if available
    for event in &events {
        if let Err(e) = crate::schedule::runner::save_result(event) {
            if !quiet {
                eprintln!("warning: failed to save scan result file: {e}");
            }
        }
        if let Some((ref pool, ref rt)) = db_runtime {
            if let Err(e) = rt.block_on(crate::schedule::runner::save_result_to_db(pool, event)) {
                if !quiet {
                    eprintln!("warning: failed to save scan result to database: {e}");
                }
            }
        }
    }

    // Compute exit code: 4 (unreachable/timeout) > 3 (FAIL finding) > 0 (pass).
    // UnsupportedProtocol and NoTls fall through to 0 — the target was
    // reachable but doesn't speak acceptable TLS; that's a scan result, not
    // a connectivity failure.
    let exit_code = if events.iter().any(|e| {
        matches!(
            e.overall_status,
            ScanStatus::Timeout | ScanStatus::ConnectionFailed | ScanStatus::Error
        )
    }) {
        4
    } else if events
        .iter()
        .any(|e| e.findings.iter().any(|f| f.status == FindingStatus::Fail))
    {
        3
    } else {
        0
    };

    // Render output and optionally save to DB
    for event in &events {
        let content: String;
        let fmt: &str;

        match args.output {
            OutputFormat::Terminal => {
                if !quiet {
                    print!("{}", crate::report::terminal::render(event));
                }
                continue;
            }
            OutputFormat::Json => {
                content = crate::report::json::render(event)?;
                fmt = "json";
                if let Some(path) = &args.report_file {
                    std::fs::write(path, &content)?;
                    if !quiet {
                        eprintln!("wrote JSON report: {}", path.display());
                    }
                } else {
                    println!("{content}");
                }
            }
            OutputFormat::Html => {
                content = crate::report::html::render(event)?;
                fmt = "html";
                if let Some(path) = &args.report_file {
                    std::fs::write(path, &content)?;
                    if !quiet {
                        eprintln!("wrote HTML report: {}", path.display());
                    }
                } else {
                    println!("{content}");
                }
            }
        }

        // Save rendered report to database if available
        if let Some((ref pool, ref rt)) = db_runtime {
            let file_path = args
                .report_file
                .as_ref()
                .map(|p| p.to_string_lossy().to_string());
            if let Err(e) = rt.block_on(crate::server::state::insert_scan_report(
                pool,
                &event.scan_id.to_string(),
                fmt,
                &content,
                file_path.as_deref(),
            )) {
                if !quiet {
                    eprintln!("warning: failed to save report to database: {e}");
                }
            }
        }
    }

    Ok(exit_code)
}

/// Try to initialize a database connection for scan result persistence.
/// Accepts an optional config file path; falls back to "qxscan.toml" when None.
/// Returns the pool and runtime so callers can use block_on for saves.
/// Best-effort — returns an error if the database is unavailable so callers
/// can choose to continue without persistence.
fn init_db_for_scan(
    config_path: Option<&std::path::Path>,
) -> anyhow::Result<(sqlx::SqlitePool, tokio::runtime::Runtime)> {
    let config =
        crate::server::Config::load(config_path.unwrap_or(std::path::Path::new("qxscan.toml")))?;
    let rt = tokio::runtime::Runtime::new()?;
    let pool = rt.block_on(crate::server::state::init_pool(&config.database.url))?;
    Ok((pool, rt))
}
