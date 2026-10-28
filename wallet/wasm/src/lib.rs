// Wallet code is exempt from the arithmetic-side-effects lint.
#![allow(clippy::arithmetic_side_effects)]

use celenium_cli_lib::celenium_cli;
use wasm_bindgen::prelude::*;
use workflow_terminal::Options;
use workflow_terminal::Result;

#[wasm_bindgen]
pub async fn load_celenium_wallet_cli() -> Result<()> {
    let options = Options { ..Options::default() };
    celenium_cli(options, None).await?;
    Ok(())
}
