//! ############################################################################
//! @file       mod.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      TLS probe module — handshake, certificate, cipher, and PQC detection.
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
pub mod cert;
pub mod cipher;
pub mod handshake;
pub mod pqc;
