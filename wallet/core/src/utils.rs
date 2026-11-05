//!
//! Celenium value formatting and parsing utilities.
//!

use crate::result::Result;
use celenium_addresses::Address;
use celenium_consensus_core::constants::*;
use celenium_consensus_core::network::NetworkType;
use separator::{Separatable, separated_float, separated_int, separated_uint_with_output};
use workflow_log::style;

pub fn try_celenium_str_to_pradha<S: Into<String>>(s: S) -> Result<Option<u64>> {
    let s: String = s.into();
    let amount = s.trim();
    if amount.is_empty() {
        return Ok(None);
    }

    Ok(Some(str_to_pradha(amount)?))
}

pub fn try_celenium_str_to_pradha_i64<S: Into<String>>(s: S) -> Result<Option<i64>> {
    let s: String = s.into();
    let amount = s.trim();
    if amount.is_empty() {
        return Ok(None);
    }

    let amount = amount.parse::<f64>()? * PRADHA_PER_CELENIUM as f64;
    Ok(Some(amount as i64))
}

#[inline]
pub fn pradha_to_celenium(pradha: u64) -> f64 {
    pradha as f64 / PRADHA_PER_CELENIUM as f64
}

#[inline]
pub fn celenium_to_pradha(celenium: f64) -> u64 {
    (celenium * PRADHA_PER_CELENIUM as f64) as u64
}

#[inline]
pub fn pradha_to_celenium_string(pradha: u64) -> String {
    pradha_to_celenium(pradha).separated_string()
}

#[inline]
pub fn pradha_to_celenium_string_with_trailing_zeroes(pradha: u64) -> String {
    separated_float!(format!("{:.8}", pradha_to_celenium(pradha)))
}

pub fn celenium_suffix(network_type: &NetworkType) -> &'static str {
    match network_type {
        NetworkType::Mainnet => "CEN",
        NetworkType::Testnet => "TCEN",
        NetworkType::Simnet => "SCEN",
        NetworkType::Devnet => "DCEN",
    }
}

#[inline]
pub fn pradha_to_celenium_string_with_suffix(pradha: u64, network_type: &NetworkType) -> String {
    let kas = pradha_to_celenium_string(pradha);
    let suffix = celenium_suffix(network_type);
    format!("{kas} {suffix}")
}

#[inline]
pub fn pradha_to_celenium_string_with_trailing_zeroes_and_suffix(pradha: u64, network_type: &NetworkType) -> String {
    let kas = pradha_to_celenium_string_with_trailing_zeroes(pradha);
    let suffix = celenium_suffix(network_type);
    format!("{kas} {suffix}")
}

pub fn format_address_colors(address: &Address, range: Option<usize>) -> String {
    let address = address.to_string();

    let parts = address.split(':').collect::<Vec<&str>>();
    let prefix = style(parts[0]).dim();
    let payload = parts[1];
    let range = range.unwrap_or(6);
    let start = range;
    let finish = payload.len() - range;

    let left = &payload[0..start];
    let center = style(&payload[start..finish]).dim();
    let right = &payload[finish..];

    format!("{prefix}:{left}:{center}:{right}")
}

fn str_to_pradha(amount: &str) -> Result<u64> {
    let Some(dot_idx) = amount.find('.') else {
        return Ok(amount.parse::<u64>()? * PRADHA_PER_CELENIUM);
    };
    let integer = amount[..dot_idx].parse::<u64>()? * PRADHA_PER_CELENIUM;
    let decimal = &amount[dot_idx + 1..];
    let decimal_len = decimal.len();
    let decimal = if decimal_len == 0 {
        0
    } else if decimal_len <= 8 {
        decimal.parse::<u64>()? * 10u64.pow(8 - decimal_len as u32)
    } else {
        // TODO - discuss how to handle values longer than 8 decimal places
        // (reject, truncate, ceil(), etc.)
        decimal[..8].parse::<u64>()?
    };
    Ok(integer + decimal)
}
