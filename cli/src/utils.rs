use crate::error::Error;
use crate::result::Result;
use celenium_consensus_core::constants::SOMPI_PER_CELENIUM;
use std::fmt::Display;

pub fn try_parse_required_nonzero_celenium_as_sompi_u64<S: ToString + Display>(celenium_amount: Option<S>) -> Result<u64> {
    if let Some(celenium_amount) = celenium_amount {
        let sompi_amount = celenium_amount
            .to_string()
            .parse::<f64>()
            .map_err(|_| Error::custom(format!("Supplied Celenium amount is not valid: '{celenium_amount}'")))?
            * SOMPI_PER_CELENIUM as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom("Supplied Celenium amount is not valid: '{celenium_amount}'"))
        } else {
            let sompi_amount = sompi_amount as u64;
            if sompi_amount == 0 {
                Err(Error::custom("Supplied required celenium amount must not be a zero: '{celenium_amount}'"))
            } else {
                Ok(sompi_amount)
            }
        }
    } else {
        Err(Error::custom("Missing Celenium amount"))
    }
}

pub fn try_parse_required_celenium_as_sompi_u64<S: ToString + Display>(celenium_amount: Option<S>) -> Result<u64> {
    if let Some(celenium_amount) = celenium_amount {
        let sompi_amount = celenium_amount
            .to_string()
            .parse::<f64>()
            .map_err(|_| Error::custom(format!("Supplied Kasapa amount is not valid: '{celenium_amount}'")))?
            * SOMPI_PER_CELENIUM as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom("Supplied Celenium amount is not valid: '{celenium_amount}'"))
        } else {
            Ok(sompi_amount as u64)
        }
    } else {
        Err(Error::custom("Missing Celenium amount"))
    }
}

pub fn try_parse_optional_celenium_as_sompi_i64<S: ToString + Display>(celenium_amount: Option<S>) -> Result<Option<i64>> {
    if let Some(celenium_amount) = celenium_amount {
        let sompi_amount = celenium_amount
            .to_string()
            .parse::<f64>()
            .map_err(|_e| Error::custom(format!("Supplied Kasapa amount is not valid: '{celenium_amount}'")))?
            * SOMPI_PER_CELENIUM as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom("Supplied Celenium amount is not valid: '{celenium_amount}'"))
        } else {
            Ok(Some(sompi_amount as i64))
        }
    } else {
        Ok(None)
    }
}
