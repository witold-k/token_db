// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! A compact token database with stable numeric IDs and frequency counts.
//!
//! `TokenDb` keeps tokens in insertion order. A token receives an ID the first
//! time it is inserted, and that ID remains stable for the lifetime of the
//! database and across save/load round trips.

mod db;
mod error;
mod storage;

pub use db::{TokenDb, TokenEntry, TokenId};
pub use error::{Error, Result};
