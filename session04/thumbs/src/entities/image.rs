use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "images")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub filename: String,
    pub file_size: i64,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::image_tag::Entity")]
    ImageTag,
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        super::image_tag::Relation::Tag.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::image_tag::Relation::Image.def().rev())
    }
}

impl Related<Entity> for super::image_tag::Entity {
    fn to() -> RelationDef {
        super::image_tag::Relation::Image.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
