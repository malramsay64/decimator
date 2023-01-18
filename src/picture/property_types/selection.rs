//! Modelling the picking of a picture

use std::fmt::Display;
use std::str::FromStr;

use adw::prelude::*;
use anyhow::{anyhow, Error, Result};
use glib::value::{FromValue, ValueType};
use glib::Value;
use gtk::glib;
use gtk::glib::value::GenericValueTypeChecker;
use serde::{Deserialize, Serialize};

// Collection of options for picking an image,
// it can either be Rejected, Hidden or Selected.
#[derive(Copy, Clone, Serialize, Deserialize, Debug, Default)]
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
            "Ignore" => Ok(Selection::Ignore),
            "Ordinary" => Ok(Selection::Ordinary),
            "Pick" => Ok(Selection::Pick),
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

impl ToValue for Selection {
    fn to_value(&self) -> glib::Value {
        <str>::to_value(&self.to_string())
    }

    fn value_type(&self) -> glib::Type {
        String::static_type()
    }
}

impl ValueType for Selection {
    type Type = String;
}

unsafe impl<'a> FromValue<'a> for Selection {
    type Checker = GenericValueTypeChecker<Self>;
    unsafe fn from_value(value: &'a Value) -> Self {
        Selection::from_str(<&str>::from_value(value)).unwrap()
    }
}

impl StaticType for Selection {
    fn static_type() -> glib::Type {
        String::static_type()
    }
}
