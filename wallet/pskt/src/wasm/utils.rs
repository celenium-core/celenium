use celenium_consensus_core::constants::*;
use celenium_consensus_core::network::NetworkType;
use separator::{Separatable, separated_float, separated_int, separated_uint_with_output};

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
