use sea_orm::{NotSet, Set, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTagDto {
    pub name: String,
}

impl From<CreateTagDto> for ActiveModel {
    fn from(req: CreateTagDto) -> Self {
        Self {
            id: NotSet,
            name: Set(req.name),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateTagDto {
    pub name: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::image_tag::Entity")]
    ImageTag,
}

impl Related<super::image::Entity> for Entity {
    fn to() -> RelationDef {
        super::image_tag::Relation::ImageEntity.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::image_tag::Relation::TagEntity.def().rev())
    }
}

impl Related<Entity> for super::image_tag::Entity {
    fn to() -> RelationDef {
        super::image_tag::Relation::TagEntity.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub use ActiveModel as TagModelDto;
pub use Column as TagColumn;
pub use Entity as TagEntity;
pub use Model as TagModel;
