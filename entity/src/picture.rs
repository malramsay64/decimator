use camino::Utf8PathBuf;
use sea_orm::entity::prelude::*;

use super::{Flag, Rating, Selection};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "pictures")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub directory: String,
    pub filename: String,
    pub raw_extension: Option<String>,
    pub short_hash: Option<Vec<u8>>,
    pub full_hash: Option<Vec<u8>>,
    pub capture_time: Option<TimeDateTime>,
    pub rating: Option<Rating>,
    pub flag: Option<Flag>,
    pub hidden: bool,
    pub selection: Selection,
    pub thumbnail: Option<Vec<u8>>,
}

impl Model {
    pub fn filepath(&self) -> Utf8PathBuf {
        [self.directory.clone(), self.filename.clone()]
            .iter()
            .collect::<Utf8PathBuf>()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
