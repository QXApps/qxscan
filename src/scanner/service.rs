//! ############################################################################
//! @file       service.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      Service type definitions — ServiceType enum and default port mappings.
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
//! Typed service enum.
//! Each variant maps to a default port and protocol behaviour.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceType {
    Https,
    Smtp,
    Imap,
    Pop3,
    Postgres,
    Mysql,
    Ldap,
    Ftp,
}

impl ServiceType {
    #[allow(dead_code)]
    pub fn default_port(&self) -> u16 {
        match self {
            Self::Https => 443,
            Self::Smtp => 587,
            Self::Imap => 993,
            Self::Pop3 => 995,
            Self::Postgres => 5432,
            Self::Mysql => 3306,
            Self::Ldap => 636,
            Self::Ftp => 990,
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Self::Https => "https",
            Self::Smtp => "smtp",
            Self::Imap => "imap",
            Self::Pop3 => "pop3",
            Self::Postgres => "postgres",
            Self::Mysql => "mysql",
            Self::Ldap => "ldap",
            Self::Ftp => "ftp",
        }
    }
}
