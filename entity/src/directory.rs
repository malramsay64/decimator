use sea_orm::entity::prelude::*;

// use crate::picture::Entity;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "directories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub directory: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // Picture,
    #[sea_orm(belongs_to = "Entity", from = "Column::ParentId", to = "Column::Id")]
    SelfReferencing,
    // #[sea_orm(has_many = "Entity")]
    // SubDirectory,
    // #[sea_orm(belongs_to = "Entity", from = "Column::ParentId", to = "Column::Id")]
    // ParentDirectory,
}

impl Related<Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SelfReferencing.def()
    }
}

// impl Related<super::picture::Entity> for Entity {
//     fn to() -> RelationDef {
//         Relation::Picture.def()
//     }
// }

pub struct SelfReferencingLink;

impl Linked for SelfReferencingLink {
    type FromEntity = Entity;
    type ToEntity = Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::SelfReferencing.def().rev()]
    }
}

impl ActiveModelBehavior for ActiveModel {}
