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

impl ToValue for Flag {
    fn to_value(&self) -> glib::Value {
        <str>::to_value(&self.to_string())
    }

    fn value_type(&self) -> glib::Type {
        String::static_type()
    }
}

impl ValueType for Flag {
    type Type = String;
}

unsafe impl<'a> FromValue<'a> for Flag {
    type Checker = GenericValueTypeOrNoneChecker<Self>;
    unsafe fn from_value(value: &'a Value) -> Self {
        Flag::from_str(<&str>::from_value(value)).unwrap()
    }
}

impl StaticType for Flag {
    fn static_type() -> glib::Type {
        String::static_type()
    }
}
