//! ############################################################################
//! @file       cipher.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      Cipher suite inspection — identifies negotiated cipher and properties.
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
pub fn is_forward_secrecy_cipher(cipher: &str) -> bool {
    let upper = normalize(cipher);
    upper.contains("ECDHE") || upper.contains("DHE")
}

pub fn is_tls13(version: &str) -> bool {
    matches!(version, "TLSv1.3" | "TLS1.3" | "TLS 1.3")
}

pub fn is_strong_cipher(cipher: &str) -> bool {
    let upper = normalize(cipher);
    upper.contains("AES_256_GCM")
        || upper.contains("AES_128_GCM")
        || upper.contains("CHACHA20_POLY1305")
}

fn normalize(value: &str) -> String {
    value.to_ascii_uppercase().replace('-', "_")
}
