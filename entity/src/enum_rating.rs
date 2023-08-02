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
