use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum Flag {
    #[sea_orm(string_value = "Red")]
    Red,
    #[sea_orm(string_value = "Green")]
    Green,
    #[sea_orm(string_value = "Blue")]
    Blue,
    #[sea_orm(string_value = "Yellow")]
    Yellow,
    #[sea_orm(string_value = "Purple")]
    Purple,
}
