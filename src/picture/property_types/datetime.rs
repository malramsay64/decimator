use std::fmt::Display;
use std::str::FromStr;

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use time::format_description::FormatItem;
use time::macros::format_description;
use time::PrimitiveDateTime;

// Define the format to send to the fontend. This is also used to update
// the time from the frontend.
const DISPLAY_FORMAT: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

// Define a datetime format to be used internally within the application
//
// This allows for wrapping all the required traits, ensuring the easy
// conversion between all the nessecary types and representations. The main
// one of concern here is to and from strings for display along with the
// representation within the database.
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

impl PartialOrd for DateTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.datetime.cmp(&other.datetime))
    }
}

impl Ord for DateTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.datetime.cmp(&other.datetime)
    }
}

impl Eq for DateTime {}

impl PartialEq for DateTime {
    fn eq(&self, other: &Self) -> bool {
        self.datetime.eq(&other.datetime)
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
