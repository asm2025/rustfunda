use anyhow::Result;
use async_trait::async_trait;
use sea_orm::{prelude::*, *};

use crate::data::entities::{
    Image, ImageModel, ImageTag, ImageTagColumn, ImageTagModel, ImageTagModelDto, Tag, TagColumn,
    TagModel, TagModelDto,
};

#[async_trait]
pub trait ITagRepository {
    async fn count(&self) -> Result<u64>;
    async fn list(&self) -> Result<Vec<TagModel>>;
    async fn get(&self, id: i64) -> Result<Option<TagModel>>;
    async fn create(&self, image: TagModel) -> Result<TagModel>;
    async fn update(&self, id: i64, tag: TagModel) -> Result<TagModel>;
    async fn delete(&self, id: i64) -> Result<DeleteResult>;
    async fn list_images(&self, id: i64) -> Result<Vec<ImageModel>>;
    async fn add_image(&self, tag_id: i64, image_id: i64) -> Result<ImageTagModel>;
    async fn remove_image(&self, tag_id: i64, image_id: i64) -> Result<DeleteResult>;
}

pub struct TagRepository {
    db: DatabaseConnection,
}

impl TagRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ITagRepository for TagRepository {
    async fn count(&self) -> Result<u64> {
        Tag::find().count(&self.db).await.map_err(Into::into)
    }

    async fn list(&self) -> Result<Vec<TagModel>> {
        Tag::find().all(&self.db).await.map_err(Into::into)
    }

    async fn get(&self, id: i64) -> Result<Option<TagModel>> {
        Tag::find_by_id(id).one(&self.db).await.map_err(Into::into)
    }

    async fn create(&self, tag: TagModel) -> Result<TagModel> {
        let active_model: TagModelDto = tag.into();
        active_model.insert(&self.db).await.map_err(Into::into)
    }

    async fn update(&self, id: i64, tag: TagModel) -> Result<TagModel> {
        let existing = Tag::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Tag not found"))?;
        let mut active_model: TagModelDto = existing.into();
        active_model.name = Set(tag.name);
        active_model.update(&self.db).await.map_err(Into::into)
    }

    async fn delete(&self, id: i64) -> Result<DeleteResult> {
        // First, delete the associations in ImageTag
        ImageTag::delete_many()
            .filter(ImageTagColumn::TagId.eq(id))
            .exec(&self.db)
            .await
            .map_err(anyhow::Error::from)?;
        Tag::delete_many()
            .filter(TagColumn::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(Into::into)
    }

    async fn list_images(&self, id: i64) -> Result<Vec<ImageModel>> {
        ImageTag::find()
            .filter(ImageTagColumn::TagId.eq(id))
            .find_with_related(Image)
            .all(&self.db)
            .await
            .map(|images| images.into_iter().flat_map(|(_, image)| image).collect())
            .map_err(Into::into)
    }

    async fn add_image(&self, tag_id: i64, image_id: i64) -> Result<ImageTagModel> {
        let image_tag = ImageTagModelDto {
            tag_id: Set(tag_id),
            image_id: Set(image_id),
        };
        image_tag.insert(&self.db).await.map_err(Into::into)
    }

    async fn remove_image(&self, tag_id: i64, image_id: i64) -> Result<DeleteResult> {
        ImageTag::delete_many()
            .filter(
                ImageTagColumn::TagId
                    .eq(tag_id)
                    .and(ImageTagColumn::ImageId.eq(image_id)),
            )
            .exec(&self.db)
            .await
            .map_err(Into::into)
    }
}
