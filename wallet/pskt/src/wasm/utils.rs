use celenium_consensus_core::constants::*;
use celenium_consensus_core::network::NetworkType;
use separator::{Separatable, separated_float, separated_int, separated_uint_with_output};

#[inline]
pub fn sompi_to_celenium(sompi: u64) -> f64 {
    sompi as f64 / SOMPI_PER_CELENIUM as f64
}

#[inline]
pub fn celenium_to_sompi(celenium: f64) -> u64 {
    (celenium * SOMPI_PER_CELENIUM as f64) as u64
}

#[inline]
pub fn sompi_to_celenium_string(sompi: u64) -> String {
    sompi_to_celenium(sompi).separated_string()
}

#[inline]
pub fn sompi_to_celenium_string_with_trailing_zeroes(sompi: u64) -> String {
    separated_float!(format!("{:.8}", sompi_to_celenium(sompi)))
}

pub fn celenium_suffix(network_type: &NetworkType) -> &'static str {
    match network_type {
        NetworkType::Mainnet => "KAS",
        NetworkType::Testnet => "TKAS",
        NetworkType::Simnet => "SKAS",
        NetworkType::Devnet => "DKAS",
    }
}

#[inline]
pub fn sompi_to_celenium_string_with_suffix(sompi: u64, network_type: &NetworkType) -> String {
    let kas = sompi_to_celenium_string(sompi);
    let suffix = celenium_suffix(network_type);
    format!("{kas} {suffix}")
}
