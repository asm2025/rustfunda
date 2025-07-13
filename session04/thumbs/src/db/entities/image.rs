use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{EntityTrait, NotSet, Set, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "images")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
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
        super::image_tag::Relation::TagEntity.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::image_tag::Relation::ImageEntity.def().rev())
    }
}

impl Related<Entity> for super::image_tag::Entity {
    fn to() -> RelationDef {
        super::image_tag::Relation::ImageEntity.def()
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            created_at: Set(now),
            updated_at: Set(now),
            ..ActiveModelTrait::default()
        }
    }

    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert {
            self.updated_at = Set(Utc::now());
        }
        Ok(self)
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateImageDto {
    pub title: String,
    pub description: Option<String>,
    pub filename: String,
    pub file_size: i64,
    pub mime_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt_text: Option<String>,
    pub tags: Option<String>,
}

impl From<CreateImageDto> for ImageModelDto {
    fn from(req: CreateImageDto) -> Self {
        let now = Utc::now();
        Self {
            id: NotSet,
            title: Set(req.title),
            description: Set(req.description),
            filename: Set(req.filename),
            file_size: Set(req.file_size),
            mime_type: Set(req.mime_type),
            width: Set(req.width),
            height: Set(req.height),
            alt_text: Set(req.alt_text),
            created_at: Set(now),
            updated_at: Set(now),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateImageDto {
    pub title: Option<String>,
    pub description: Option<String>,
    pub filename: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt_text: Option<String>,
}

pub use ActiveModel as ImageModelDto;
pub use Column as ImageColumn;
pub use Entity as ImageEntity;
pub use Model as ImageModel;
