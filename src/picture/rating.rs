use std::fmt::Display;
use std::str::FromStr;

use adw::prelude::*;
use anyhow::{anyhow, Error, Result};
use glib::value::{
    FromValue, GenericValueTypeOrNoneChecker, ToValueOptional, ValueType, ValueTypeOptional,
};
use glib::Value;
use gtk::glib;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Rating {
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
            "one" | "One" => Ok(Rating::One),
            "two" | "Two" => Ok(Rating::Two),
            "three" | "Three" => Ok(Rating::Three),
            "four" | "Four" => Ok(Rating::Four),
            "five" | "Five" => Ok(Rating::Five),
            _ => Err(anyhow!("Invalid value for rating.")),
        }
    }
}

impl Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
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
    type Checker = GenericValueTypeOrNoneChecker<Self>;
    unsafe fn from_value(value: &'a Value) -> Self {
        Rating::from_str(<&str>::from_value(value)).unwrap()
    }
}
impl ValueTypeOptional for Rating {}
impl StaticType for Rating {
    fn static_type() -> glib::Type {
        String::static_type()
    }
}
impl ToValueOptional for Rating {
    fn to_value_optional(s: Option<&Self>) -> glib::Value {
        let value = s.map(Rating::to_string);
        <String>::to_value_optional(value.as_ref())
    }
}
