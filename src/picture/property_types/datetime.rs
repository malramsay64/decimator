use std::fmt::Display;
use std::str::FromStr;

use adw::prelude::*;
use anyhow::{Error, Result};
use glib::value::{FromValue, GenericValueTypeOrNoneChecker, ValueType};
use glib::Value;
use gtk::glib;
use gtk::glib::value::{ToValueOptional, ValueTypeOptional};
use serde::{Deserialize, Serialize};
use time::format_description::FormatItem;
use time::macros::format_description;
use time::{PrimitiveDateTime};

// Define the format to send to the fontend. This is also used to update
// the time from the frontend.
const DISPLAY_FORMAT: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DateTime {
    datetime: PrimitiveDateTime,
}

impl DateTime {
    pub fn year(&self) -> i32 {
        self.datetime.year()
    }

    pub fn month(&self) -> u8 {
        self.datetime.month().into()
    }

    pub fn day(&self) -> u8 {
        self.datetime.day()
    }

    pub fn datetime(&self) -> PrimitiveDateTime {
        self.datetime
    }
}

impl From<PrimitiveDateTime> for DateTime {
    fn from(value: PrimitiveDateTime) -> Self {
        Self { datetime: value }
    }
}

impl TryFrom<String> for DateTime {
    type Error = Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl FromStr for DateTime {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            datetime: PrimitiveDateTime::parse(input, DISPLAY_FORMAT)?,
        })
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.datetime.format(DISPLAY_FORMAT).unwrap())
    }
}

impl ToValue for DateTime {
    fn to_value(&self) -> glib::Value {
        <str>::to_value(&self.to_string())
    }

    fn value_type(&self) -> glib::Type {
        String::static_type()
    }
}

impl ValueType for DateTime {
    type Type = String;
}
impl ValueTypeOptional for DateTime {}
impl ToValueOptional for DateTime {
    fn to_value_optional(s: Option<&Self>) -> glib::Value {
        let value = s.map(Self::to_string);
        <String>::to_value_optional(value.as_ref())
    }
}

unsafe impl<'a> FromValue<'a> for DateTime {
    type Checker = GenericValueTypeOrNoneChecker<Self>;
    unsafe fn from_value(value: &'a Value) -> Self {
        DateTime::from_str(<&str>::from_value(value)).expect("Unable to parse datetime string")
    }
}

impl StaticType for DateTime {
    fn static_type() -> glib::Type {
        String::static_type()
    }
}
