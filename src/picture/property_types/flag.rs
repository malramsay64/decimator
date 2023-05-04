
use std::str::FromStr;

use anyhow::{anyhow, Error, Result};
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum Flag {
    #[default]
    #[sea_orm(string_value = "None")]
    None,
    #[sea_orm(string_value = "Red")]
    Red,
    #[sea_orm(string_value = "Green")]
    Green,
    #[sea_orm(string_value = "Blue")]
    Blue,
    #[sea_orm(string_value = "Yellow")]
    Yellow,
    #[sea_orm(string_value = "Purple")]
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

impl From<Flag> for String {
    fn from(value: Flag) -> Self {
        match value {
            Flag::None => "None".into(),
            Flag::Red => "Red".into(),
            Flag::Green => "Green".into(),
            Flag::Blue => "Blue".into(),
            Flag::Yellow => "Yellow".into(),
            Flag::Purple => "Purple".into(),
        }
    }
}
