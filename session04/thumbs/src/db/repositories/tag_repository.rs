use anyhow::Result;
use async_trait::async_trait;
use sea_orm::{DeleteResult, prelude::*};

use crate::db::prelude::*;

#[async_trait]
pub trait ITagRepository: IRepository + IRepositoryWithRelated {
    async fn list(&self) -> Result<ResultSet<TagModel>>;
    async fn count(&self) -> Result<u64>;
    async fn get(&self, id: i64) -> Result<Option<TagModel>>;
    async fn create(&self, tag: CreateTagDto) -> Result<TagModel>;
    async fn update(&self, id: i64, tag: UpdateTagDto) -> Result<TagModel>;
    async fn delete(&self, id: i64) -> Result<DeleteResult>;
    async fn list_images(&self, id: i64) -> Result<Vec<ImageModel>>;
    async fn add_image(&self, tag_id: i64, image_id: i64) -> Result<ImageTagModel>;
    async fn remove_image(&self, tag_id: i64, image_id: i64) -> Result<DeleteResult>;
    async fn add_images(&self, tag_id: i64, image_ids: Vec<i64>) -> Result<Vec<ImageTagModel>>;
    async fn remove_images(&self, tag_id: i64, image_ids: Vec<i64>) -> Result<DeleteResult>;
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
    async fn list(&self) -> Result<Vec<TagModel>> {
        TagEntity::find().all(&self.db).await.map_err(Into::into)
    }

    async fn count(&self) -> Result<u64> {
        TagEntity::find().count(&self.db).await.map_err(Into::into)
    }

    async fn get(&self, id: i64) -> Result<Option<TagModel>> {
        TagEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(Into::into)
    }

    async fn create(&self, tag: CreateTagDto) -> Result<TagModel> {
        let active_model: TagModelDto = tag.into();
        active_model.insert(&self.db).await.map_err(Into::into)
    }

    async fn update(&self, id: i64, tag: UpdateTagDto) -> Result<TagModel> {
        let existing = TagEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Tag not found"))?;
        let mut active_model: TagModelDto = existing.into();

        if let Some(tag) = tag.name {
            active_model.name = Set(tag);
        }

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
            .find_with_related(ImageEntity)
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

    async fn add_images(&self, tag_id: i64, image_ids: Vec<i64>) -> Result<Vec<ImageTagModel>> {
        let mut image_tags = Vec::new();

        for image_id in image_ids {
            let image_tag = ImageTagModelDto {
                tag_id: Set(tag_id),
                image_id: Set(image_id),
            };
            image_tags.push(image_tag.insert(&self.db).await?);
        }

        Ok(image_tags)
    }

    async fn remove_images(&self, tag_id: i64, image_ids: Vec<i64>) -> Result<DeleteResult> {
        ImageTag::delete_many()
            .filter(
                ImageTagColumn::TagId
                    .eq(tag_id)
                    .and(ImageTagColumn::ImageId.is_in(image_ids)),
            )
            .exec(&self.db)
            .await
            .map_err(Into::into)
    }
}
