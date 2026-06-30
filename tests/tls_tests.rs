//! Integration tests for the TLS module — cipher detection and PQC.
//!
//! These tests cover the pure functions in:
//!   src/tls/cipher.rs
//!   src/tls/pqc.rs

use qxscan::tls::cipher;
use qxscan::tls::pqc;

// ── TLS version detection ────────────────────────────────────

#[test]
fn tls13_detected() {
    assert!(cipher::is_tls13("TLSv1.3"));
    assert!(cipher::is_tls13("TLS1.3"));
    assert!(cipher::is_tls13("TLS 1.3"));
}

#[test]
fn tls12_not_tls13() {
    assert!(!cipher::is_tls13("TLSv1.2"));
    assert!(!cipher::is_tls13("TLSv1.1"));
    assert!(!cipher::is_tls13("TLSv1.0"));
    assert!(!cipher::is_tls13("SSLv3"));
}

#[test]
fn empty_and_unknown_not_tls13() {
    assert!(!cipher::is_tls13(""));
    assert!(!cipher::is_tls13("bogus"));
}

// ── Strong cipher detection ──────────────────────────────────

#[test]
fn strong_ciphers_pass() {
    assert!(cipher::is_strong_cipher("TLS_AES_256_GCM_SHA384"));
    assert!(cipher::is_strong_cipher("TLS_AES_128_GCM_SHA256"));
    assert!(cipher::is_strong_cipher("TLS_CHACHA20_POLY1305_SHA256"));
    // Hyphenated variant (some OpenSSL versions)
    assert!(cipher::is_strong_cipher("TLS-AES-256-GCM-SHA384"));
}

#[test]
fn weak_ciphers_fail() {
    assert!(!cipher::is_strong_cipher("TLS_RSA_WITH_AES_128_CBC_SHA"));
    assert!(!cipher::is_strong_cipher("TLS_RSA_WITH_RC4_128_SHA"));
    assert!(!cipher::is_strong_cipher(
        "TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA256"
    ));
    assert!(!cipher::is_strong_cipher(""));
}

// ── Forward secrecy detection ────────────────────────────────

#[test]
fn forward_secrecy_ecdhe() {
    assert!(cipher::is_forward_secrecy_cipher(
        "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384"
    ));
    assert!(cipher::is_forward_secrecy_cipher(
        "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256"
    ));
}

#[test]
fn forward_secrecy_dhe() {
    assert!(cipher::is_forward_secrecy_cipher(
        "TLS_DHE_RSA_WITH_AES_256_GCM_SHA384"
    ));
}

#[test]
fn no_forward_secrecy() {
    assert!(!cipher::is_forward_secrecy_cipher(
        "TLS_RSA_WITH_AES_256_GCM_SHA384"
    ));
    assert!(!cipher::is_forward_secrecy_cipher("TLS_AES_256_GCM_SHA384"));
    assert!(!cipher::is_forward_secrecy_cipher(""));
}

// ── PQC hybrid key exchange detection ────────────────────────

#[test]
fn pqc_hybrid_detected() {
    assert!(pqc::is_pqc_hybrid_kex("TLS_KYBER_MLKEM_HYBRID"));
    assert!(pqc::is_pqc_hybrid_kex("KYBER1024"));
    assert!(pqc::is_pqc_hybrid_kex("MLKEM-1024"));
    assert!(pqc::is_pqc_hybrid_kex("HYBRID-KEX"));
}

#[test]
fn pqc_hybrid_not_detected() {
    assert!(!pqc::is_pqc_hybrid_kex("TLS_AES_256_GCM_SHA384"));
    assert!(!pqc::is_pqc_hybrid_kex("TLS_CHACHA20_POLY1305_SHA256"));
    assert!(!pqc::is_pqc_hybrid_kex(""));
}
