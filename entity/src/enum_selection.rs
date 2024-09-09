use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Copy, Clone, Serialize, Deserialize, Debug, Default, PartialEq, Eq, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum Selection {
    #[sea_orm(string_value = "Ignore")]
    Ignore,
    #[default]
    #[sea_orm(string_value = "Ordinary")]
    Ordinary,
    #[sea_orm(string_value = "Pick")]
    Pick,
}
