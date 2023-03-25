//! Modelling the picking of a picture

use std::fmt::Display;
use std::str::FromStr;

use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};

// Collection of options for picking an image,
// it can either be Rejected, Hidden or Selected.
#[derive(Copy, Clone, Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub enum Selection {
    Ignore,
    #[default]
    Ordinary,
    Pick,
}

impl FromStr for Selection {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            // We need to match the string with quotes to allow for converting
            // from the variant representation.
            // TODO: Consolidate all handling of Selection to a variant
            "Ignore" | "'Ignore'" => Ok(Selection::Ignore),
            "Ordinary" | "'Ordinary'" => Ok(Selection::Ordinary),
            "Pick" | "'Pick'" => Ok(Selection::Pick),
            _ => Err(anyhow!("Invalid value for Selection.")),
        }
    }
}

impl TryFrom<&str> for Selection {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Selection::Ignore => "Ignore",
            Selection::Ordinary => "Ordinary",
            Selection::Pick => "Pick",
        };
        write!(f, "{text}")
    }
}
