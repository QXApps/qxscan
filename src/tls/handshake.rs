//! ############################################################################
//! @file       handshake.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      TCP connect and TLS handshake — negotiates TLS session with target.
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
use std::net::ToSocketAddrs;
use std::time::Duration;

use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode, SslVersion};

use crate::qem::observation::TlsInfo;

#[derive(Debug, Clone)]
pub struct HandshakeOutcome {
    pub ip: Option<String>,
    pub tls: TlsInfo,
}

#[derive(Debug)]
pub enum ProbeError {
    ResolutionFailed(String),
    ConnectionFailed(std::io::Error),
    TlsHandshakeFailed(String),
    /// TCP connection succeeded, but the server did not respond with TLS
    /// for any of the supported protocol versions.
    NotTls(String),
    NoSupportedTls,
}

impl std::fmt::Display for ProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProbeError::ResolutionFailed(host) => write!(f, "DNS resolution failed for {host}"),
            ProbeError::ConnectionFailed(e) => write!(f, "TCP connection failed: {e}"),
            ProbeError::TlsHandshakeFailed(msg) => write!(f, "TLS handshake failed: {msg}"),
            ProbeError::NotTls(msg) => write!(f, "Target does not appear to use TLS: {msg}"),
            ProbeError::NoSupportedTls => write!(
                f,
                "No supported TLS protocol found (requires TLS 1.2 or 1.3)"
            ),
        }
    }
}

impl std::error::Error for ProbeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProbeError::ConnectionFailed(e) => Some(e),
            _ => None,
        }
    }
}

const SUPPORTED_TLS_VERSIONS: [SslVersion; 2] = [SslVersion::TLS1_3, SslVersion::TLS1_2];

pub fn probe(
    host: &str,
    port: u16,
    timeout: Duration,
    no_verify: bool,
) -> Result<HandshakeOutcome, ProbeError> {
    let addr = format!("{host}:{port}");
    let socks = match addr.to_socket_addrs() {
        Ok(s) => s.collect::<Vec<_>>(),
        Err(_) => return Err(ProbeError::ResolutionFailed(host.to_string())),
    };

    if socks.is_empty() {
        return Err(ProbeError::ResolutionFailed(host.to_string()));
    }

    // When --no-verify is set (common for private-network self-signed certs),
    // skip chain validation. Certificate metadata (expiry, SANs, subject) is
    // still collected and evaluated by the compliance layer regardless.
    let verification_mode = if no_verify {
        SslVerifyMode::NONE
    } else {
        SslVerifyMode::PEER
    };

    let mut last_conn_err = None;
    let mut last_tls_err = None;

    for &tls_version in &SUPPORTED_TLS_VERSIONS {
        for &sock in &socks {
            match try_connect_tls(host, sock, timeout, verification_mode, tls_version) {
                Ok(tls) => {
                    return Ok(HandshakeOutcome {
                        ip: Some(sock.ip().to_string()),
                        tls,
                    });
                }
                Err(e) => {
                    // anyhow wraps the original error; io::Error is a root
                    // cause (source() returns None). Try downcast_ref directly
                    // first, then fall back to source() for nested errors
                    // (e.g. openssl wrapping io::Error).
                    if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                        last_conn_err = Some(io_err.kind());
                    } else if let Some(io_err) =
                        e.source().and_then(|s| s.downcast_ref::<std::io::Error>())
                    {
                        last_conn_err = Some(io_err.kind());
                    } else if last_tls_err.is_none() {
                        last_tls_err = Some(e.to_string());
                    }
                }
            }
        }
    }

    if let Some(err_kind) = last_conn_err {
        Err(ProbeError::ConnectionFailed(std::io::Error::from(err_kind)))
    } else if let Some(msg) = last_tls_err {
        // TCP connected but TLS failed — determine whether the server
        // responded with non-TLS data (plain HTTP, etc.) vs. a genuine
        // TLS handshake error (cert validation, cipher mismatch, …).
        if is_not_tls_error(&msg) {
            Err(ProbeError::NotTls(msg))
        } else {
            Err(ProbeError::TlsHandshakeFailed(msg))
        }
    } else {
        Err(ProbeError::NoSupportedTls)
    }
}

/// Heuristic to detect whether a TLS handshake error indicates the
/// server does not speak TLS at all (plain HTTP, SSH, etc.) rather than
/// a genuine TLS protocol or certificate issue.
///
/// OpenSSL produces specific error messages when it receives plaintext
/// (e.g. an HTTP response) instead of a TLS record:
/// - "wrong version number" — HTTP/1.1 "HTTP/1.1" isn't a valid TLS record
/// - "unexpected EOF" — server closes after receiving ClientHello
/// - "http request" — rare; some servers return HTTP 400
fn is_not_tls_error(msg: &str) -> bool {
    let keywords = [
        "wrong version number",
        "unexpected EOF",
        "http request",
        "no protocols available",
    ];
    keywords.iter().any(|kw| msg.contains(kw))
}

fn try_connect_tls(
    host: &str,
    sock: std::net::SocketAddr,
    timeout: Duration,
    verify_mode: SslVerifyMode,
    tls_version: SslVersion,
) -> anyhow::Result<TlsInfo> {
    let mut builder = SslConnector::builder(SslMethod::tls())?;
    builder.set_verify(verify_mode);
    builder.set_min_proto_version(Some(tls_version))?;
    builder.set_max_proto_version(Some(tls_version))?;
    let connector = builder.build();

    let stream = std::net::TcpStream::connect_timeout(&sock, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;

    let tls_stream = connector.connect(host, stream)?;
    let ssl = tls_stream.ssl();

    let negotiated_version = ssl.version_str().to_string();
    let cipher = ssl
        .current_cipher()
        .map(|c| c.name().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let forward_secrecy = if crate::tls::cipher::is_tls13(&negotiated_version) {
        true
    } else {
        crate::tls::cipher::is_forward_secrecy_cipher(&cipher)
    };
    let pqc_hybrid = crate::tls::pqc::is_pqc_hybrid_kex(&cipher);

    let cert = ssl
        .peer_certificate()
        .as_ref()
        .map(|cert| crate::tls::cert::from_x509(cert.as_ref()))
        .transpose()?;

    Ok(TlsInfo {
        negotiated_version,
        cipher,
        forward_secrecy,
        pqc_hybrid,
        cert,
    })
}
