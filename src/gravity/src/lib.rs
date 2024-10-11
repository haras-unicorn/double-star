#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::todo)]
#![deny(clippy::unreachable)]
#![deny(clippy::allow_attributes_without_reason)]

pub mod config;
pub mod log;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DoubleStarMessage {
  Generated(String),
  Break,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrbitusMessage {
  Submit(String),
  Exited,
}
