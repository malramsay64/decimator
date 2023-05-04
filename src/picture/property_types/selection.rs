//! Modelling the picking of a picture

use std::str::FromStr;

use anyhow::{anyhow, Error, Result};
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};

// Collection of options for picking an image,
// it can either be Rejected, Hidden or Selected.
#[derive(
    Copy, Clone, Serialize, Deserialize, Debug, Default, PartialEq, Eq, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum Selection {
    #[sea_orm(string_value = "Ignore")]
    Ignore,
    #[default]
    #[sea_orm(string_value = "Ordinary")]
    Ordinary,
    #[sea_orm(string_value = "Pick")]
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

impl From<Selection> for String {
    fn from(value: Selection) -> Self {
        match value {
            Selection::Ignore => "Ignore".into(),
            Selection::Ordinary => "Ordinary".into(),
            Selection::Pick => "Pick".into(),
        }
    }
}
