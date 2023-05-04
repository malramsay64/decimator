use std::str::FromStr;

use anyhow::{Error, Result};
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumIter,
    DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum Rating {
    #[default]
    #[sea_orm(string_value = "Zero")]
    Zero,
    #[sea_orm(string_value = "One")]
    One,
    #[sea_orm(string_value = "Two")]
    Two,
    #[sea_orm(string_value = "Three")]
    Three,
    #[sea_orm(string_value = "Four")]
    Four,
    #[sea_orm(string_value = "Five")]
    Five,
}

impl FromStr for Rating {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "zero" | "Zero" => Ok(Rating::Zero),
            "one" | "One" => Ok(Rating::One),
            "two" | "Two" => Ok(Rating::Two),
            "three" | "Three" => Ok(Rating::Three),
            "four" | "Four" => Ok(Rating::Four),
            "five" | "Five" => Ok(Rating::Five),
            _ => Ok(Rating::default()),
        }
    }
}

impl TryFrom<&str> for Rating {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl From<Rating> for String {
    fn from(value: Rating) -> Self {
        match value {
            Rating::Zero => "Zero".into(),
            Rating::One => "One".into(),
            Rating::Two => "Two".into(),
            Rating::Three => "Three".into(),
            Rating::Four => "Four".into(),
            Rating::Five => "Five".into(),
        }
    }
}
