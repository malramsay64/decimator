use std::fmt::Display;
use std::str::FromStr;

use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Flag {
    #[default]
    None,
    Red,
    Green,
    Blue,
    Yellow,
    Purple,
}

impl FromStr for Flag {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "none" | "None" => Ok(Flag::None),
            "red" | "Red" => Ok(Flag::Red),
            "green" | "Green" => Ok(Flag::Green),
            "blue" | "Blue" => Ok(Flag::Blue),
            "yellow" | "Yellow" => Ok(Flag::Yellow),
            "purple" | "Purple" => Ok(Flag::Purple),
            _ => Err(anyhow!("Invalid value for Flags.")),
        }
    }
}

impl TryFrom<&str> for Flag {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Flag::None => "None",
            Flag::Red => "Red",
            Flag::Green => "Green",
            Flag::Blue => "Blue",
            Flag::Yellow => "Yellow",
            Flag::Purple => "Purple",
        };
        write!(f, "{text}")
    }
}
