use sea_orm::{EntityTrait, NotSet, Set, prelude::*};
use serde::{Deserialize, Serialize};

use crate::db::Merge;

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

#[derive(Debug, Deserialize)]
pub struct CreateTagDto {
    pub name: String,
}

impl From<CreateTagDto> for Model {
    fn from(req: CreateTagDto) -> Self {
        Self {
            id: 0,
            name: req.name,
        }
    }
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

impl Merge<ActiveModel> for UpdateTagDto {
    fn merge(&self, model: &mut ActiveModel) {
        if let Some(name) = self.name.as_ref() {
            model.name = Set(name.clone());
        }
    }
}

pub use ActiveModel as TagModelDto;
pub use Column as TagColumn;
pub use Entity as TagEntity;
pub use Model as TagModel;
