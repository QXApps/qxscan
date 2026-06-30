//! ############################################################################
//! @file       runner.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      Scan runner loop — scheduled scan execution, result persistence, and daemon loop.
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
//! Scan runner — executes due schedules and stores results.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use chrono::Utc;
use sqlx::SqlitePool;
use tokio::runtime::Runtime;

use crate::compliance::scoring::evaluate;
use crate::qem::finding::FindingStatus;
use crate::qem::metadata::{ScanEvent, ScanStatus};
use crate::scanner::service::ServiceType;
use crate::scanner::{probe_target, resolve_targets};
use crate::server::state::{self, Schedule};

const RESULTS_DIR: &str = "results";

/// Save scan result to a JSON file in the results directory.
pub fn save_result(event: &ScanEvent) -> anyhow::Result<PathBuf> {
    let dir = PathBuf::from(RESULTS_DIR);
    std::fs::create_dir_all(&dir)?;

    let ts = event.scanned_at.format("%Y%m%d_%H%M%S");
    let host = event.target.host.replace(':', "_");
    let filename = format!("scan_{ts}_{host}_{}.json", event.target.port);
    let path = dir.join(&filename);

    let json = serde_json::to_string_pretty(event)?;
    std::fs::write(&path, json)?;
    log::info!("saved scan result: {}", path.display());
    Ok(path)
}

/// Save scan result to the database via the connection pool.
pub async fn save_result_to_db(pool: &SqlitePool, event: &ScanEvent) -> anyhow::Result<()> {
    state::insert_scan_event(pool, event).await
}

/// Run a scheduled scan, storing results in both SQLite and JSON files.
pub async fn run_scheduled_scan(
    pool: &SqlitePool,
    schedule: &Schedule,
    timeout_secs: u64,
    _concurrency: usize,
) -> anyhow::Result<Vec<ScanEvent>> {
    let timeout = Duration::from_secs(timeout_secs);
    let standards = if schedule.standards.is_empty() {
        vec!["pci-dss".to_string()]
    } else {
        schedule.standards.clone()
    };

    let targets_file = &schedule.targets_file;
    if !targets_file.exists() {
        anyhow::bail!("targets file not found: {}", targets_file.display());
    }

    let targets = resolve_targets(None, Some(targets_file.as_path()), 443, &ServiceType::Https)?;

    let mut events = Vec::new();
    for target in &targets {
        let start = Instant::now();
        let probed = probe_target(target, timeout, true);
        let mut event = ScanEvent::new(probed.target.clone());
        event.overall_status = probed.status.clone();
        event.tls = probed.tls;

        if event.overall_status == ScanStatus::Pass {
            for standard in &standards {
                match evaluate(&event, standard) {
                    Ok((findings, score)) => {
                        event.findings.extend(findings);
                        event.compliance.insert(standard.replace('-', "_"), score);
                    }
                    Err(e) => {
                        log::warn!("compliance evaluation failed for {standard}: {e}");
                    }
                }
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

        // Store in database
        state::insert_scan_event(pool, &event).await?;

        // Also save to JSON file
        if let Err(e) = save_result(&event) {
            log::warn!("failed to save JSON result: {e}");
        }

        events.push(event);
    }

    Ok(events)
}

/// Main daemon loop — polls for due schedules every 60 seconds.
pub fn daemon_loop(pool: SqlitePool, timeout_secs: u64, concurrency: usize) {
    let rt = match Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            log::error!("failed to create tokio runtime for daemon loop: {e}");
            return;
        }
    };

    loop {
        let schedules = match rt.block_on(state::get_due_schedules(&pool)) {
            Ok(s) => s,
            Err(e) => {
                log::error!("failed to query due schedules: {e}");
                std::thread::sleep(Duration::from_secs(60));
                continue;
            }
        };

        for schedule in &schedules {
            log::info!(
                "running scheduled scan: {} ({})",
                schedule.label.as_deref().unwrap_or("unnamed"),
                schedule.cron_expr
            );

            match rt.block_on(run_scheduled_scan(
                &pool,
                schedule,
                timeout_secs,
                concurrency,
            )) {
                Ok(events) => {
                    log::info!("scan completed: {} targets scanned", events.len());
                }
                Err(e) => {
                    log::error!("scheduled scan failed: {e}");
                }
            }

            let next =
                match crate::schedule::cron::next_run_from_cron(&schedule.cron_expr, &Utc::now()) {
                    Ok(dt) => dt,
                    Err(e) => {
                        log::error!(
                            "failed to compute next run for schedule {}: {e}",
                            schedule.id
                        );
                        Utc::now() + chrono::Duration::hours(1)
                    }
                };

            if let Err(e) = rt.block_on(state::update_next_run(
                &pool,
                &schedule.id.to_string(),
                &next,
            )) {
                log::error!("failed to update next_run: {e}");
            }
        }

        log::debug!("scheduler loop: sleeping 60s");
        std::thread::sleep(Duration::from_secs(60));
    }
}
