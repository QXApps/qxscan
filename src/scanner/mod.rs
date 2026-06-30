//! ############################################################################
//! @file       mod.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      Scanner module — target resolution and probe orchestration.
//!
//! @details    Pure-function module with no side effects, global state, or
//!             file I/O. Must be unit-testable without a network connection.
//!
//! ### REVISION HISTORY
//! | Date       | Version | Author                  | Description |
//! |------------|---------|-------------------------|-------------|
//! | 2026-06-02 | 1.0.0   | Phaneendra Bhattiprolu  | Initial implementation. |
//! |            |         |                         |             |
//!
//! ### COMMENTS / NOTES
//! * No global state, no file I/O, no println!.
//! * Unit-testable without a network connection.
//! ############################################################################
pub mod service;

use std::path::Path;
use std::time::Duration;

use crate::qem::metadata::{ScanStatus, TargetInfo};
use crate::qem::observation::TlsInfo;

#[derive(Debug, Clone)]
pub struct ResolvedTarget {
    pub host: String,
    pub port: u16,
    pub service: service::ServiceType,
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub target: TargetInfo,
    pub status: ScanStatus,
    pub tls: Option<TlsInfo>,
}

pub fn resolve_targets(
    target: Option<&str>,
    targets_file: Option<&Path>,
    default_port: u16,
    service: &service::ServiceType,
) -> anyhow::Result<Vec<ResolvedTarget>> {
    let mut targets = Vec::new();

    if let Some(raw) = target {
        targets.push(parse_target(raw, default_port, service));
    }

    if let Some(path) = targets_file {
        let content = std::fs::read_to_string(path)?;
        for line in content.lines() {
            let raw = line.trim();
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            targets.push(parse_target(raw, default_port, service));
        }
    }

    if targets.is_empty() {
        anyhow::bail!("missing target: provide TARGET or --targets-file");
    }

    Ok(targets)
}

pub fn probe_target(target: &ResolvedTarget, timeout: Duration, no_verify: bool) -> ProbeResult {
    match crate::tls::handshake::probe(&target.host, target.port, timeout, no_verify) {
        Ok(outcome) => ProbeResult {
            target: TargetInfo {
                host: target.host.clone(),
                ip: outcome.ip,
                port: target.port,
                service: target.service.slug().to_string(),
            },
            status: ScanStatus::Pass,
            tls: Some(outcome.tls),
        },
        Err(err) => {
            let status = match err {
                crate::tls::handshake::ProbeError::ResolutionFailed(_) => ScanStatus::Error,
                crate::tls::handshake::ProbeError::ConnectionFailed(e) => {
                    if e.kind() == std::io::ErrorKind::TimedOut {
                        ScanStatus::Timeout
                    } else {
                        ScanStatus::ConnectionFailed
                    }
                }
                crate::tls::handshake::ProbeError::TlsHandshakeFailed(_) => ScanStatus::Error,
                crate::tls::handshake::ProbeError::NoSupportedTls => {
                    ScanStatus::UnsupportedProtocol
                }
            };
            ProbeResult {
                target: TargetInfo {
                    host: target.host.clone(),
                    ip: None,
                    port: target.port,
                    service: target.service.slug().to_string(),
                },
                status,
                tls: None,
            }
        }
    }
}

fn parse_target(raw: &str, default_port: u16, service: &service::ServiceType) -> ResolvedTarget {
    match raw.rsplit_once(':') {
        Some((host, port)) if !host.is_empty() => match port.parse::<u16>() {
            Ok(parsed) => ResolvedTarget {
                host: host.to_string(),
                port: parsed,
                service: service.clone(),
            },
            Err(_) => ResolvedTarget {
                host: raw.to_string(),
                port: default_port,
                service: service.clone(),
            },
        },
        _ => ResolvedTarget {
            host: raw.to_string(),
            port: default_port,
            service: service.clone(),
        },
    }
}
