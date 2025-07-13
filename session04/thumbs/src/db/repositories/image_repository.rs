use anyhow::Result;
use async_trait::async_trait;
use migration::OnConflict;
use sea_orm::{DeleteResult, JoinType, PaginatorTrait, QuerySelect, Set, prelude::*};

use crate::db::prelude::*;

#[async_trait]
pub trait IImageRepository: IRepository + IRepositoryWithRelated {
    async fn list_tags<F>(
        &self,
        id: Self::Key,
        filter: Option<F>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<<Self::Related as EntityTrait>::Model>>
    where
        F: FilterCondition<Self::Related> + Send + Sync;
    async fn add_tag(&self, id: i64, tag_id: i64) -> Result<()>;
    async fn remove_tag(&self, id: i64, tag_id: i64) -> Result<DeleteResult>;
    async fn add_tags(&self, id: i64, tags: &[i64]) -> Result<u64>;
    async fn remove_tags(&self, id: i64, tags: &[i64]) -> Result<u64>;
    async fn add_tag_str(&self, id: i64, tag: &str) -> Result<()>;
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
impl IHasDatabase for ImageRepository {
    fn database(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl IRepository for ImageRepository {
    type Entity = ImageEntity;
    type Model = ImageModel;
    type Key = i64;
    type CreateDto = CreateImageDto;
    type UpdateDto = UpdateImageDto;

    async fn list<F>(
        &self,
        filter: Option<F>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<Self::Model>>
    where
        F: FilterCondition<Self::Entity> + Send + Sync,
    {
        let mut query = Self::Entity::find();

        if let Some(f) = &filter {
            query = f.apply(query);
        }

        let total = query.clone().count(self.database()).await?;

        if let Some(p) = pagination {
            query = query.offset((p.page - 1) * p.page_size).limit(p.page_size);
        }

        let data = query.all(self.database()).await?;

        Ok(ResultSet {
            data,
            total,
            pagination,
        })
    }

    async fn count<F>(&self, filter: Option<F>) -> Result<u64>
    where
        F: FilterCondition<Self::Entity> + Send + Sync,
    {
        let mut query = Self::Entity::find();

        if let Some(f) = &filter {
            query = f.apply(query);
        }

        query.count(self.database()).await.map_err(Into::into)
    }

    async fn get(&self, id: Self::Key) -> Result<Option<Self::Model>> {
        Self::Entity::find_by_id(id)
            .one(self.database())
            .await
            .map_err(Into::into)
    }

    async fn create(&self, model: Self::CreateDto) -> Result<Self::Model> {
        let tags = model.tags.clone();
        let active_model: ImageModelDto = model.into();
        let result = active_model.insert(self.database()).await?;
        let Some(tags) = tags else {
            return Ok(result);
        };

        if tags.is_empty() {
            return Ok(result);
        }

        let tags = tags
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        if tags.is_empty() {
            return Ok(result);
        }

        Self::Related::insert_many(tags.iter().map(|&tag| TagModelDto {
            name: Set(tag.to_string()),
            ..Default::default()
        }))
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .exec_without_returning(self.database())
        .await?;

        let tag_ids = Self::Related::find()
            .filter(TagColumn::Name.is_in(tags))
            .all(self.database())
            .await?
            .into_iter()
            .map(|tag| tag.id)
            .collect::<Vec<_>>();

        if tag_ids.is_empty() {
            return Ok(result);
        }

        ImageTagEntity::insert_many(tag_ids.iter().map(|&tag_id| ImageTagModelDto {
            image_id: Set(result.id),
            tag_id: Set(tag_id),
        }))
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .exec_without_returning(self.database())
        .await?;

        Ok(result)
    }

    async fn update(&self, id: Self::Key, model: Self::UpdateDto) -> Result<Self::Model> {
        let existing = Self::Entity::find_by_id(id)
            .one(self.database())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Image not found"))?;
        let mut active_model: ImageModelDto = existing.into();

        if let Some(title) = model.title {
            active_model.title = Set(title);
        }

        if let Some(description) = model.description {
            active_model.description = Set(Some(description));
        }

        if let Some(filename) = model.filename {
            active_model.filename = Set(filename);
        }

        if let Some(file_size) = model.file_size {
            active_model.file_size = Set(file_size);
        }

        if let Some(mime_type) = model.mime_type {
            active_model.mime_type = Set(mime_type);
        }

        if let Some(width) = model.width {
            active_model.width = Set(Some(width));
        }

        if let Some(height) = model.height {
            active_model.height = Set(Some(height));
        }

        if let Some(alt_text) = model.alt_text {
            active_model.alt_text = Set(Some(alt_text));
        }

        active_model
            .update(self.database())
            .await
            .map_err(Into::into)
    }

    async fn delete(&self, id: Self::Key) -> Result<DeleteResult> {
        Self::Entity::delete_by_id(id)
            .exec(self.database())
            .await
            .map_err(Into::into)
    }
}

#[async_trait]
impl IRepositoryWithRelated for ImageRepository {
    type Related = TagEntity;

    async fn list_with_related<F, R>(
        &self,
        filter: Option<F>,
        filter_related: Option<R>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<(Self::Model, Vec<<Self::Related as EntityTrait>::Model>)>>
    where
        F: FilterCondition<Self::Entity> + Send + Sync,
        R: FilterRelatedCondition<Self::Entity, Self::Related> + Send + Sync,
    {
        let mut query = Self::Entity::find();

        if let Some(f) = &filter {
            query = f.apply(query);
        }

        let mut query = query.find_with_related(TagEntity);

        if let Some(r) = &filter_related {
            query = r.apply(query);
        }

        let total = query.clone().count(self.database()).await?;

        if let Some(p) = pagination {
            query = query.offset((p.page - 1) * p.page_size).limit(p.page_size);
        }

        let data = query.all(self.database()).await?;

        Ok(ResultSet {
            data,
            total,
            pagination,
        })
    }
}

#[async_trait]
impl IImageRepository for ImageRepository {
    async fn list_tags<F>(
        &self,
        id: i64,
        filter: Option<F>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<<Self::Related as EntityTrait>::Model>>
    where
        F: FilterCondition<Self::Related> + Send + Sync,
    {
        let mut query = <Self::Related as EntityTrait>::find()
            .join(
                JoinType::InnerJoin,
                ImageTagEntity::belongs_to(Self::Related)
                    .from(ImageTagColumn::TagId)
                    .to(TagColumn::Id)
                    .into(),
            )
            .filter(ImageTagColumn::ImageId.eq(id));

        if let Some(f) = &filter {
            query = f.apply(query);
        }

        let total = query.clone().count(self.database()).await?;

        if let Some(p) = pagination {
            query = query.offset((p.page - 1) * p.page_size).limit(p.page_size);
        }

        let data = query.all(self.database()).await?;
        Ok(ResultSet {
            data,
            total,
            pagination,
        })
    }

    async fn add_tag(&self, id: i64, tag_id: i64) -> Result<()> {
        let image_tag = ImageTagModelDto {
            image_id: Set(id),
            tag_id: Set(tag_id),
        };
        image_tag.insert(self.database()).await?;
        Ok(())
    }

    async fn remove_tag(&self, id: i64, tag_id: i64) -> Result<DeleteResult> {
        ImageTagEntity::delete_many()
            .filter(ImageTagColumn::ImageId.eq(id))
            .filter(ImageTagColumn::TagId.eq(tag_id))
            .exec(self.database())
            .await
            .map_err(Into::into)
    }

    async fn add_tags(&self, id: i64, tags: &[i64]) -> Result<u64> {
        if tags.is_empty() {
            return Ok(0);
        }

        let image_tags = tags.iter().map(|&tag_id| ImageTagModelDto {
            image_id: Set(id),
            tag_id: Set(tag_id),
        });

        let result = ImageTagEntity::insert_many(image_tags)
            .on_conflict(OnConflict::new().do_nothing().to_owned())
            .exec_without_returning(self.database())
            .await?;

        Ok(result)
    }

    async fn remove_tags(&self, id: i64, tags: &[i64]) -> Result<u64> {
        if tags.is_empty() {
            return Ok(0);
        }

        let result = ImageTagEntity::delete_many()
            .filter(ImageTagColumn::ImageId.eq(id))
            .filter(ImageTagColumn::TagId.is_in(tags.to_vec()))
            .exec(self.database())
            .await?;

        Ok(result.rows_affected)
    }

    async fn add_tag_str(&self, id: i64, tag: &str) -> Result<()> {
        let existing_tag = Self::Related::find()
            .filter(TagColumn::Name.eq(tag))
            .one(self.database())
            .await?;
        let tag_id = match existing_tag {
            Some(t) => t.id,
            None => {
                let new_tag = TagModelDto {
                    name: Set(tag.to_string()),
                    ..Default::default()
                };
                let inserted = Self::Related::insert(new_tag)
                    .on_conflict(OnConflict::new().do_nothing().to_owned())
                    .exec(self.database())
                    .await?;
                inserted.last_insert_id
            }
        };
        let image_tag = ImageTagModelDto {
            image_id: Set(id),
            tag_id: Set(tag_id),
        };
        image_tag.insert(self.database()).await?;
        Ok(())
    }
}
