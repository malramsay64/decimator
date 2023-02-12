use std::fmt::Display;
use std::str::FromStr;

use adw::prelude::*;
use anyhow::{anyhow, Error, Result};
use glib::value::{FromValue, GenericValueTypeChecker, ValueType};
use glib::Value;
use gtk::glib;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Rating {
    #[default]
    Zero,
    One,
    Two,
    Three,
    Four,
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

impl Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Rating::Zero => "Zero",
            Rating::One => "One",
            Rating::Two => "Two",
            Rating::Three => "Three",
            Rating::Four => "Four",
            Rating::Five => "Five",
        };
        write!(f, "{text}")
    }
}

impl ToValue for Rating {
    fn to_value(&self) -> glib::Value {
        <str>::to_value(&self.to_string())
    }

    fn value_type(&self) -> glib::Type {
        String::static_type()
    }
}

impl ValueType for Rating {
    type Type = String;
}
unsafe impl<'a> FromValue<'a> for Rating {
    type Checker = GenericValueTypeChecker<Self>;
    unsafe fn from_value(value: &'a Value) -> Self {
        Rating::from_str(<&str>::from_value(value)).unwrap()
    }
}
impl StaticType for Rating {
    fn static_type() -> glib::Type {
        String::static_type()
    }
}
