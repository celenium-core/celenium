#![allow(clippy::arithmetic_side_effects)]
extern crate self as celenium_cli;

mod cli;
pub mod error;
pub mod extensions;
mod helpers;
mod imports;
mod matchers;
pub mod modules;
mod notifier;
pub mod result;
pub mod utils;
mod wizards;

pub use cli::{CeleniumCli, Options, TerminalOptions, TerminalTarget, celenium_cli};
pub use workflow_terminal::Terminal;
