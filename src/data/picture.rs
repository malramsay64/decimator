use camino::Utf8PathBuf;
use sea_orm::entity::prelude::*;
use sea_orm::prelude::*;
use time::PrimitiveDateTime;

use crate::picture::{Flag, Rating, Selection};

#[derive(Debug, Default, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "picture")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Binary(BlobSize::Blob(Some(16)))"
    )]
    pub id: Uuid,
    pub directory: String,
    pub filename: String,
    pub raw_extension: Option<String>,
    pub capture_time: Option<PrimitiveDateTime>,
    pub selection: Selection,
    pub rating: Rating,
    pub flag: Flag,
    pub hidden: bool,
    pub thumbnail: Option<Vec<u8>>,
}

impl Model {
    pub fn filepath(&self) -> Utf8PathBuf {
        [self.directory.clone(), self.filename.clone()]
            .iter()
            .collect::<Utf8PathBuf>()
    }
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
