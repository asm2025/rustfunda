use anyhow::Result;
use async_trait::async_trait;
use sea_orm::{prelude::*, *};

use crate::db::entities::{
    Image, ImageModel, ImageModelDto, ImageTag, ImageTagColumn, ImageTagModel, ImageTagModelDto,
    Tag, TagModel,
};

#[async_trait]
pub trait IImageRepository {
    async fn count(&self) -> Result<u64>;
    async fn list(&self) -> Result<Vec<ImageModel>>;
    async fn get(&self, id: i64) -> Result<Option<ImageModel>>;
    async fn create(&self, image: ImageModel) -> Result<ImageModel>;
    async fn update(&self, id: i64, image: ImageModel) -> Result<ImageModel>;
    async fn delete(&self, id: i64) -> Result<DeleteResult>;
    async fn list_tags(&self, id: i64) -> Result<Vec<TagModel>>;
    async fn add_tag(&self, image_id: i64, tag_id: i64) -> Result<ImageTagModel>;
    async fn remove_tag(&self, image_id: i64, tag_id: i64) -> Result<DeleteResult>;
}

pub struct ImageRepository {
    db: DatabaseConnection,
}

impl ImageRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IImageRepository for ImageRepository {
    async fn count(&self) -> Result<u64> {
        Image::find().count(&self.db).await.map_err(Into::into)
    }

    async fn list(&self) -> Result<Vec<ImageModel>> {
        Image::find().all(&self.db).await.map_err(Into::into)
    }

    async fn get(&self, id: i64) -> Result<Option<ImageModel>> {
        Image::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(Into::into)
    }

    async fn create(&self, image: ImageModel) -> Result<ImageModel> {
        let active_model: ImageModelDto = image.into();
        active_model.insert(&self.db).await.map_err(Into::into)
    }

    async fn update(&self, id: i64, image: ImageModel) -> Result<ImageModel> {
        // Fetch the existing model by ID
        let existing = Image::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Image not found"))?;
        // Convert to ActiveModel and update fields
        let mut active_model: ImageModelDto = existing.into();

        // Update fields if they are set in the provided image model
        if !image.filename.is_empty() {
            active_model.filename = Set(image.filename);
        }

        if image.file_size > 0 {
            active_model.file_size = Set(image.file_size);
        }

        if !image.mime_type.is_empty() {
            active_model.mime_type = Set(image.mime_type);
        }

        if let Some(width) = image.width {
            if width > 0 {
                active_model.width = Set(Some(width));
            }
        }

        if let Some(height) = image.height {
            if height > 0 {
                active_model.height = Set(Some(height));
            }
        }

        if let Some(alt_text) = image.alt_text {
            if !alt_text.is_empty() {
                active_model.alt_text = Set(Some(alt_text));
            }
        }

        let updated = active_model.update(&self.db).await?;
        Ok(updated)
    }

    async fn delete(&self, id: i64) -> Result<DeleteResult> {
        // First, delete all tags associated with the image
        ImageTag::delete_many()
            .filter(ImageTagColumn::ImageId.eq(id))
            .exec(&self.db)
            .await
            .map_err(anyhow::Error::from)?;
        // Then, delete the image itself
        Image::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(Into::into)
    }

    async fn list_tags(&self, id: i64) -> Result<Vec<TagModel>> {
        ImageTag::find()
            .filter(ImageTagColumn::ImageId.eq(id))
            .find_with_related(Tag)
            .all(&self.db)
            .await
            .map(|tags| tags.into_iter().flat_map(|(_, tag)| tag).collect())
            .map_err(Into::into)
    }

    async fn add_tag(&self, image_id: i64, tag_id: i64) -> Result<ImageTagModel> {
        let image_tag = ImageTagModelDto {
            image_id: Set(image_id),
            tag_id: Set(tag_id),
        };
        image_tag.insert(&self.db).await.map_err(Into::into)
    }

    async fn remove_tag(&self, image_id: i64, tag_id: i64) -> Result<DeleteResult> {
        ImageTag::delete_many()
            .filter(ImageTagColumn::ImageId.eq(image_id))
            .filter(ImageTagColumn::TagId.eq(tag_id))
            .exec(&self.db)
            .await
            .map_err(Into::into)
    }
}
