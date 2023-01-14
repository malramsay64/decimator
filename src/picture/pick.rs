//! Modelling the picking of a picture

use std::fmt::Display;

use adw::prelude::*;
use anyhow::{anyhow, Error, Result};
use glib::value::{
    FromValue, GenericValueTypeOrNoneChecker, ToValueOptional, ValueType, ValueTypeOptional,
};
use glib::Value;
use gtk::glib;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

// Collection of options for picking an image,
// it can either be Rejected, Hidden or Selected.
#[derive(Copy, Clone, Serialize, Deserialize, Debug, Default)]
pub enum Selection {
    Rejected,
    #[default]
    None,
    Picked,
}

impl FromStr for Selection {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "rejected" | "Rejected" => Ok(Selection::Rejected),
            "none" | "None" => Ok(Selection::None),
            "selected" | "Selected" => Ok(Selection::Picked),
            _ => Err(anyhow!("Invalid value for Picked.")),
        }
    }
}

impl Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Selection::Rejected => "Rejected",
            Selection::None => "None",
            Selection::Picked => "Selected",
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
    type Checker = GenericValueTypeOrNoneChecker<Self>;
    unsafe fn from_value(value: &'a Value) -> Self {
        Selection::from_str(<&str>::from_value(value)).unwrap()
    }
}

impl StaticType for Selection {
    fn static_type() -> glib::Type {
        String::static_type()
    }
}
