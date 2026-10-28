// Wallet code is exempt from the arithmetic-side-effects lint.
#![allow(clippy::arithmetic_side_effects)]

use celenium_cli_lib::{TerminalOptions, celenium_cli};

#[tokio::main]
async fn main() {
    let result = celenium_cli(TerminalOptions::new().with_prompt("$ "), None).await;
    if let Err(err) = result {
        println!("{err}");
    }
}
