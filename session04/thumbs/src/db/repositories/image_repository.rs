use anyhow::Result;
use async_trait::async_trait;
use migration::OnConflict;
use sea_orm::{DeleteResult, JoinType, PaginatorTrait, QuerySelect, Set, prelude::*};

use crate::db::prelude::*;

#[async_trait]
pub trait IImageRepository: IRepositoryWithRelated<ImageEntity, UpdateImageDto, TagEntity> {
    async fn create_with_tags(&self, model: CreateImageDto) -> Result<ImageModel>;
    async fn list_tags(
        &self,
        id: i64,
        filter: Option<Box<dyn FilterCondition<TagEntity> + Send + Sync>>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<TagModel>>;
    async fn add_tag(&self, id: i64, related_id: i64) -> Result<()>;
    async fn remove_tag(&self, id: i64, related_id: i64) -> Result<DeleteResult>;
    async fn add_tags(&self, id: i64, tags: Vec<i64>) -> Result<u64>;
    async fn remove_tags(&self, id: i64, tags: Vec<i64>) -> Result<u64>;
    async fn add_tags_from_str(&self, id: i64, tags: &str) -> Result<u64>;
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
impl IRepository<ImageEntity, UpdateImageDto> for ImageRepository {
    async fn list(
        &self,
        filter: Option<Box<dyn FilterCondition<ImageEntity> + Send + Sync>>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<<ImageEntity as EntityTrait>::Model>> {
        let mut query = <ImageEntity as EntityTrait>::find();

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

    async fn count(
        &self,
        filter: Option<Box<dyn FilterCondition<ImageEntity> + Send + Sync>>,
    ) -> Result<u64> {
        let mut query = <ImageEntity as EntityTrait>::find();

        if let Some(f) = &filter {
            query = f.apply(query);
        }

        query.count(self.database()).await.map_err(Into::into)
    }

    async fn get(&self, id: i64) -> Result<Option<<ImageEntity as EntityTrait>::Model>> {
        ImageEntity::find_by_id(id)
            .one(self.database())
            .await
            .map_err(Into::into)
    }

    async fn create(
        &self,
        model: <ImageEntity as EntityTrait>::Model,
    ) -> Result<<ImageEntity as EntityTrait>::Model> {
        let active_model: <ImageEntity as EntityTrait>::ActiveModel = model.into();
        active_model
            .insert(self.database())
            .await
            .map_err(Into::into)
    }

    async fn update(&self, id: i64, model: UpdateImageDto) -> Result<ImageModel> {
        let existing = ImageEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Image not found".to_owned()))?;
        let mut active_model: ImageModelDto = existing.into();
        model.merge(&mut active_model);
        active_model
            .update(self.database())
            .await
            .map_err(Into::into)
    }

    async fn delete(&self, id: i64) -> Result<DeleteResult> {
        // First, delete the associations in ImageTag
        ImageTagEntity::delete_many()
            .filter(ImageTagColumn::ImageId.eq(id))
            .exec(&self.db)
            .await
            .map_err(anyhow::Error::from)?;
        ImageEntity::delete_by_id(id)
            .exec(self.database())
            .await
            .map_err(Into::into)
    }
}

#[async_trait]
impl IRepositoryWithRelated<ImageEntity, UpdateImageDto, TagEntity> for ImageRepository {
    async fn list_with_related(
        &self,
        filter: Option<Box<dyn FilterCondition<ImageEntity> + Send + Sync>>,
        filter_related: Option<
            Box<dyn FilterRelatedCondition<ImageEntity, TagEntity> + Send + Sync>,
        >,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<ModelWithRelated<ImageModel, TagModel>>> {
        let mut query = <ImageEntity as EntityTrait>::find();

        if let Some(f) = &filter {
            query = f.apply(query);
        }

        let count_query = query.clone();
        let total = count_query.count(self.database()).await?;
        let mut query = query.find_with_related(TagEntity);

        if let Some(l) = &filter_related {
            query = l.apply(query);
        }

        if let Some(p) = pagination {
            query = query.offset((p.page - 1) * p.page_size).limit(p.page_size);
        }

        let data = query
            .all(self.database())
            .await?
            .into_iter()
            .map(|e| ModelWithRelated {
                item: e.0,
                related: e.1,
            })
            .collect();

        Ok(ResultSet {
            data,
            total,
            pagination,
        })
    }
}

#[async_trait]
impl IImageRepository for ImageRepository {
    async fn create_with_tags(&self, model: CreateImageDto) -> Result<ImageModel> {
        let tags = model.tags.clone();
        let active_model: ImageModelDto = model.into();
        let result = active_model.insert(self.database()).await?;
        let Some(tags) = tags else {
            return Ok(result);
        };
        self.add_tags_from_str(result.id, &tags).await?;
        Ok(result)
    }

    async fn list_tags(
        &self,
        id: i64,
        filter: Option<Box<dyn FilterCondition<TagEntity> + Send + Sync>>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<TagModel>> {
        let mut query = <TagEntity as EntityTrait>::find()
            .join(
                JoinType::InnerJoin,
                ImageTagEntity::belongs_to(TagEntity)
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

    async fn add_tag(&self, id: i64, related_id: i64) -> Result<()> {
        let active_model = ImageTagModelDto {
            image_id: Set(id),
            tag_id: Set(related_id),
        };
        active_model.insert(self.database()).await?;
        Ok(())
    }

    async fn remove_tag(&self, id: i64, related_id: i64) -> Result<DeleteResult> {
        ImageTagEntity::delete_many()
            .filter(
                ImageTagColumn::ImageId
                    .eq(id)
                    .and(ImageTagColumn::TagId.eq(related_id)),
            )
            .exec(self.database())
            .await
            .map_err(Into::into)
    }

    async fn add_tags(&self, id: i64, tags: Vec<i64>) -> Result<u64> {
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

    async fn remove_tags(&self, id: i64, tags: Vec<i64>) -> Result<u64> {
        if tags.is_empty() {
            return Ok(0);
        }

        let result = ImageTagEntity::delete_many()
            .filter(
                ImageTagColumn::ImageId
                    .eq(id)
                    .and(ImageTagColumn::TagId.is_in(tags)),
            )
            .exec(self.database())
            .await?;

        Ok(result.rows_affected)
    }

    async fn add_tags_from_str(&self, id: i64, tags: &str) -> Result<u64> {
        if tags.is_empty() {
            return Ok(0);
        }

        let tags = tags
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        if tags.is_empty() {
            return Ok(0);
        }

        TagEntity::insert_many(tags.iter().map(|&tag| TagModelDto {
            name: Set(tag.to_string()),
            ..Default::default()
        }))
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .exec_without_returning(self.database())
        .await?;

        let tag_ids = TagEntity::find()
            .filter(TagColumn::Name.is_in(tags))
            .all(self.database())
            .await?
            .into_iter()
            .map(|tag| tag.id)
            .collect::<Vec<_>>();

        if tag_ids.is_empty() {
            return Ok(0);
        }

        let result = ImageTagEntity::insert_many(tag_ids.iter().map(|&tag_id| ImageTagModelDto {
            image_id: Set(id),
            tag_id: Set(tag_id),
        }))
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .exec_without_returning(self.database())
        .await?;

        Ok(result)
    }
}
