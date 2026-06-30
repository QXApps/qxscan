//! ############################################################################
//! @file       pqc.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      Post-quantum cryptography detection — PQC hybrid key exchange detection.
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
pub fn is_pqc_hybrid_kex(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    upper.contains("KYBER") || upper.contains("MLKEM") || upper.contains("HYBRID")
}
