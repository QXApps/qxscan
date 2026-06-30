//! ############################################################################
//! @file       mod.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      Daemon lifecycle — config, start/stop/restart/status, and HTTP server.
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
//! Daemon lifecycle: start, stop, restart, status.

pub mod pid;
pub mod state;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;

use std::time::Duration;

use serde::Deserialize;
use sqlx::SqlitePool;
use tokio::runtime::Runtime;

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_pid_file")]
    pub pid_file: PathBuf,
    #[serde(default = "default_bind")]
    pub bind: String,
}

fn default_pid_file() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".qxscan").join("qxscan.pid"))
        .unwrap_or_else(|| PathBuf::from("/tmp/qxscan.pid"))
}

fn default_bind() -> String {
    "127.0.0.1:9412".into()
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ScanConfig {
    #[serde(default = "default_timeout")]
    pub timeout_s: u64,
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    #[serde(default)]
    #[allow(dead_code)]
    pub standards: Vec<String>,
}

fn default_timeout() -> u64 {
    10
}
fn default_concurrency() -> usize {
    4
}

/// Database configuration section from qxscan.toml
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_url")]
    pub url: String,
    #[serde(default = "default_max_connections")]
    #[allow(dead_code)]
    pub max_connections: u32,
    #[serde(default = "default_connect_timeout")]
    #[allow(dead_code)]
    pub connect_timeout_s: u64,
}

fn default_db_url() -> String {
    state::default_db_url()
}
fn default_max_connections() -> u32 {
    5
}
fn default_connect_timeout() -> u64 {
    10
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: default_db_url(),
            max_connections: default_max_connections(),
            connect_timeout_s: default_connect_timeout(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub scan: ScanConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
}

impl Config {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        if path.exists() {
            let raw = std::fs::read_to_string(path)?;
            Ok(toml::from_str(&raw)?)
        } else {
            Ok(Config::default())
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                pid_file: default_pid_file(),
                bind: default_bind(),
            },
            scan: ScanConfig {
                timeout_s: default_timeout(),
                concurrency: default_concurrency(),
                standards: vec![],
            },
            database: DatabaseConfig::default(),
        }
    }
}

pub fn start_daemon(config: &Config) -> anyhow::Result<String> {
    let pid_path = &config.server.pid_file;

    if let Some(pid) = pid::read_pid(pid_path)? {
        if is_pid_alive(pid) {
            anyhow::bail!("daemon is already running (PID {pid})");
        }
        pid::remove_pid(pid_path);
    }

    let child = std::process::Command::new(std::env::current_exe()?)
        .arg("--daemon")
        .arg("--config")
        .arg(config_path_from_args())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn daemon process: {e}"))?;

    let pid = child.id();
    pid::write_pid(pid_path)?;
    Ok(format!("daemon started (PID {pid})"))
}

pub fn stop_daemon(config: &Config) -> anyhow::Result<String> {
    let pid_path = &config.server.pid_file;
    match pid::read_pid(pid_path)? {
        None => Ok("daemon is not running".into()),
        Some(pid) => {
            if !is_pid_alive(pid) {
                pid::remove_pid(pid_path);
                return Ok(format!("daemon PID {pid} is not running; cleaned up"));
            }
            let _ = Command::new("kill").args([&pid.to_string()]).status();
            for _ in 0..10 {
                if !is_pid_alive(pid) {
                    pid::remove_pid(pid_path);
                    return Ok("daemon stopped".into());
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            let _ = Command::new("kill").args(["-9", &pid.to_string()]).status();
            pid::remove_pid(pid_path);
            Ok("daemon force-stopped (SIGKILL)".into())
        }
    }
}

pub fn daemon_status(config: &Config) -> anyhow::Result<String> {
    use std::fmt::Write;

    let pid_path = &config.server.pid_file;
    let mut out = String::new();
    match pid::read_pid(pid_path)? {
        None => {
            out.push_str("daemon: not running\n");
        }
        Some(pid) => {
            if is_pid_alive(pid) {
                writeln!(out, "daemon: running (PID {pid})").unwrap();
                writeln!(out, "pid file: {}", pid_path.display()).unwrap();
                writeln!(
                    out,
                    "state db: {}",
                    state::mask_password(&config.database.url)
                )
                .unwrap();
                writeln!(out, "bind: {}", config.server.bind).unwrap();

                let db_url = &config.database.url;
                let rt = Runtime::new()?;
                match rt.block_on(state::init_pool(db_url)) {
                    Ok(pool) => {
                        match rt.block_on(state::get_recent_scans(&pool, 5)) {
                            Ok(scans) => {
                                writeln!(out).unwrap();
                                writeln!(out, "recent scans:").unwrap();
                                for event in &scans {
                                    writeln!(
                                        out,
                                        "  {} → {}:{} ({:?})",
                                        event.scan_id.to_string().split('-').next().unwrap_or("?"),
                                        event.target.host,
                                        event.target.port,
                                        event.overall_status,
                                    )
                                    .unwrap();
                                }
                            }
                            Err(e) => writeln!(out, "  (error reading scans: {e})").unwrap(),
                        }
                        match rt.block_on(state::list_schedules(&pool)) {
                            Ok(schedules) => {
                                writeln!(out).unwrap();
                                writeln!(out, "schedules ({}):", schedules.len()).unwrap();
                                for s in &schedules {
                                    let label = s.label.as_deref().unwrap_or("unnamed");
                                    let next = s.next_run_at.format("%Y-%m-%d %H:%M UTC");
                                    writeln!(
                                        out,
                                        "  {} ({label}) next: {next}",
                                        s.id.to_string().split('-').next().unwrap_or("?")
                                    )
                                    .unwrap();
                                }
                            }
                            Err(e) => writeln!(out, "  (error reading schedules: {e})").unwrap(),
                        }
                    }
                    Err(e) => writeln!(out, "  (error connecting to database: {e})").unwrap(),
                }
            } else {
                writeln!(out, "daemon: PID {pid} found but process is not running").unwrap();
                writeln!(out, "  (run 'qxscan server start' to restart)").unwrap();
                pid::remove_pid(pid_path);
            }
        }
    }
    Ok(out)
}

pub fn run_daemon_foreground(config: &Config) -> anyhow::Result<u8> {
    let rt = Runtime::new()?;
    let pool = rt.block_on(state::init_pool(&config.database.url))?;
    pid::write_pid(&config.server.pid_file)?;

    let http_pool = pool.clone();
    let http_bind = config.server.bind.clone();
    let http_handle = {
        std::thread::spawn(move || {
            if let Err(e) = run_http_server(&http_bind, http_pool) {
                log::error!("HTTP server stopped: {e}");
            }
        })
    };

    let db_url = config.database.url.clone();
    let timeout = config.scan.timeout_s;
    let concurrency = config.scan.concurrency;
    let schedule_handle = std::thread::spawn(move || {
        let sched_rt = match Runtime::new() {
            Ok(r) => r,
            Err(e) => {
                log::error!("failed to create runtime for scheduler: {e}");
                return;
            }
        };
        match sched_rt.block_on(state::init_pool(&db_url)) {
            Ok(sched_pool) => {
                crate::schedule::runner::daemon_loop(sched_pool, timeout, concurrency);
            }
            Err(e) => log::error!("failed to open state db for scheduler: {e}"),
        }
    });

    ctrlc_handler(pool.clone(), &config.server.pid_file);

    let _ = http_handle.join();
    let _ = schedule_handle.join();
    Ok(0)
}

fn ctrlc_handler(_pool: SqlitePool, pid_file: &std::path::Path) {
    let pid_file = pid_file.to_owned();
    ctrlc::set_handler(move || {
        log::info!("shutting down daemon");
        pid::remove_pid(&pid_file);
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");
}

fn config_path_from_args() -> PathBuf {
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--config" && i + 1 < args.len() {
            return PathBuf::from(&args[i + 1]);
        }
    }
    PathBuf::from("qxscan.toml")
}

fn is_pid_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run_http_server(bind: &str, pool: SqlitePool) -> anyhow::Result<()> {
    let listener = TcpListener::bind(bind)?;
    log::info!("HTTP server listening on {bind}");

    for stream in listener.incoming() {
        let pool = pool.clone();
        match stream {
            Ok(mut stream) => {
                std::thread::spawn(move || {
                    handle_client(&mut stream, &pool);
                });
            }
            Err(e) => {
                log::error!("connection error: {e}");
            }
        }
    }
    Ok(())
}

fn handle_client(stream: &mut std::net::TcpStream, pool: &SqlitePool) {
    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    let request = String::from_utf8_lossy(&buf[..n]);
    let (status, content_type, body) = route_request(&request, pool);
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

fn route_request(request: &str, pool: &SqlitePool) -> (&'static str, &'static str, String) {
    let path = request
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("/");

    match path {
        "/status" => {
            let rt = match Runtime::new() {
                Ok(r) => r,
                Err(_) => {
                    return (
                        "500 Internal Server Error",
                        "text/plain",
                        "runtime error\n".into(),
                    )
                }
            };
            let scans = rt.block_on(state::get_recent_scans(pool, 10)).ok();
            let schedules = rt.block_on(state::list_schedules(pool)).ok();
            let status_json = serde_json::json!({
                "product": crate::about::PRODUCT,
                "version": crate::about::BUILD,
                "engine": crate::about::ENGINE,
                "uptime_secs": std::process::id(),
                "scans_count": scans.as_ref().map(|s| s.len()).unwrap_or(0),
                "schedules_count": schedules.as_ref().map(|s| s.len()).unwrap_or(0),
            });
            ("200 OK", "application/json", status_json.to_string())
        }
        "/metrics" => {
            let rt = match Runtime::new() {
                Ok(r) => r,
                Err(_) => {
                    return (
                        "500 Internal Server Error",
                        "text/plain",
                        "runtime error\n".into(),
                    )
                }
            };
            let scans = rt.block_on(state::get_recent_scans(pool, 100)).ok();
            let mut output = String::new();
            output.push_str("# HELP qxscan_uptime_seconds Daemon uptime\n");
            output.push_str("# TYPE qxscan_uptime_seconds gauge\n");
            output.push_str(&format!(
                "qxscan_uptime_seconds{{product=\"{product}\",version=\"{version}\"}} 1\n",
                product = crate::about::PRODUCT,
                version = crate::about::BUILD,
            ));
            output.push_str("# HELP qxscan_scans_total Total scans completed\n");
            output.push_str("# TYPE qxscan_scans_total counter\n");
            output.push_str(&format!(
                "qxscan_scans_total{{product=\"{product}\"}} {}\n",
                scans.as_ref().map(|s| s.len()).unwrap_or(0),
                product = crate::about::PRODUCT,
            ));
            ("200 OK", "text/plain; charset=utf-8", output)
        }
        _ => ("404 Not Found", "text/plain", "not found\n".into()),
    }
}
