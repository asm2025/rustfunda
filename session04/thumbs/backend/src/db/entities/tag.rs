use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::image_tag::Entity")]
    ImageTag,
}

impl Related<super::image::Entity> for Entity {
    fn to() -> RelationDef {
        super::image_tag::Relation::Image.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::image_tag::Relation::Tag.def().rev())
    }
}

impl Related<Entity> for super::image_tag::Entity {
    fn to() -> RelationDef {
        super::image_tag::Relation::Tag.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub use ActiveModel as TagModelDto;
pub use Column as TagColumn;
pub use Entity as Tag;
pub use Model as TagModel;
