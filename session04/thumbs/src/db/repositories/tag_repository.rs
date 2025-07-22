use anyhow::{Result, anyhow};
use async_trait::async_trait;
use migration::OnConflict;
use sea_orm::{
    DatabaseTransaction, DeleteResult, JoinType, PaginatorTrait, QuerySelect, Set,
    TransactionTrait, prelude::*,
};

use crate::db::prelude::*;

#[async_trait]
pub trait ITagRepository: IRepositoryWithRelated<TagEntity, UpdateTagDto, ImageEntity> {
    async fn list_images(
        &self,
        id: i64,
        filter: Option<Box<dyn FilterCondition<ImageEntity> + Send + Sync>>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<ImageModel>>;
    async fn add_image(&self, id: i64, related_id: i64) -> Result<ImageTagModel>;
    async fn remove_image(&self, id: i64, related_id: i64) -> Result<DeleteResult>;
    async fn add_images(&self, id: i64, images: Vec<i64>) -> Result<u64>;
    async fn remove_images(&self, id: i64, images: Vec<i64>) -> Result<u64>;
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
impl IHasDatabase for TagRepository {
    fn database(&self) -> &DatabaseConnection {
        &self.db
    }

    async fn begin_transaction(&self) -> Result<DatabaseTransaction> {
        self.db.begin().await.map_err(anyhow::Error::from)
    }
}

#[async_trait]
impl IRepository<TagEntity, UpdateTagDto> for TagRepository {
    async fn list(
        &self,
        filter: Option<Box<dyn FilterCondition<TagEntity> + Send + Sync>>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<TagModel>> {
        let mut query = <TagEntity as EntityTrait>::find();

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
        filter: Option<Box<dyn FilterCondition<TagEntity> + Send + Sync>>,
    ) -> Result<u64> {
        let mut query = <TagEntity as EntityTrait>::find();

        if let Some(f) = &filter {
            query = f.apply(query);
        }

        query.count(self.database()).await.map_err(Into::into)
    }

    async fn get(&self, id: i64) -> Result<Option<TagModel>> {
        TagEntity::find_by_id(id)
            .one(self.database())
            .await
            .map_err(Into::into)
    }

    async fn create(&self, model: TagModel) -> Result<TagModel> {
        let active_model: TagModelDto = model.into();
        active_model
            .insert(self.database())
            .await
            .map_err(Into::into)
    }

    async fn update(&self, id: i64, model: UpdateTagDto) -> Result<TagModel> {
        let existing = TagEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Tag not found".to_owned()))?;
        let mut active_model: TagModelDto = existing.into();
        model.merge(&mut active_model);
        active_model
            .update(self.database())
            .await
            .map_err(Into::into)
    }

    async fn delete(&self, id: i64) -> Result<Option<TagModel>> {
        let model = TagEntity::find_by_id(id)
            .one(self.database())
            .await
            .map_err(anyhow::Error::from)?;
        let Some(model) = model else {
            return Err(anyhow!("Tag not found."));
        };

        // First, delete the associations in ImageTag
        ImageTagEntity::delete_many()
            .filter(ImageTagColumn::ImageId.eq(id))
            .exec(&self.db)
            .await
            .map_err(anyhow::Error::from)?;
        TagEntity::delete_by_id(id)
            .exec(self.database())
            .await
            .map_err(anyhow::Error::from)?;

        Ok(Some(model))
    }
}

#[async_trait]
impl IRepositoryWithRelated<TagEntity, UpdateTagDto, ImageEntity> for TagRepository {
    async fn list_with_related(
        &self,
        filter: Option<Box<dyn FilterCondition<TagEntity> + Send + Sync>>,
        filter_related: Option<
            Box<dyn FilterRelatedCondition<TagEntity, ImageEntity> + Send + Sync>,
        >,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<ModelWithRelated<TagModel, ImageModel>>> {
        let mut query = <TagEntity as EntityTrait>::find();

        if let Some(f) = &filter {
            query = f.apply(query);
        }

        let count_query = query.clone();
        let total = count_query.count(self.database()).await?;
        let mut query = query.find_with_related(ImageEntity);

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

    async fn get_with_related(
        &self,
        id: i64,
    ) -> Result<Option<ModelWithRelated<TagModel, ImageModel>>> {
        let tag = <TagEntity as EntityTrait>::find_by_id(id)
            .one(self.database())
            .await?;
        let Some(tag) = tag else { return Ok(None) };
        let images = tag.find_related(ImageEntity).all(self.database()).await?;

        Ok(Some(ModelWithRelated {
            item: tag,
            related: images,
        }))
    }
}

#[async_trait]
impl ITagRepository for TagRepository {
    async fn list_images(
        &self,
        id: i64,
        filter: Option<Box<dyn FilterCondition<ImageEntity> + Send + Sync>>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<ImageModel>> {
        let mut query = <ImageEntity as EntityTrait>::find()
            .join(
                JoinType::InnerJoin,
                ImageTagEntity::belongs_to(ImageEntity)
                    .from(ImageTagColumn::ImageId)
                    .to(ImageColumn::Id)
                    .into(),
            )
            .filter(ImageTagColumn::TagId.eq(id));

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

    async fn add_image(&self, id: i64, related_id: i64) -> Result<ImageTagModel> {
        let active_model = ImageTagModelDto {
            tag_id: Set(id),
            image_id: Set(related_id),
        };
        active_model
            .insert(self.database())
            .await
            .map_err(Into::into)
    }

    async fn remove_image(&self, id: i64, related_id: i64) -> Result<DeleteResult> {
        ImageTagEntity::delete_many()
            .filter(
                ImageTagColumn::TagId
                    .eq(id)
                    .and(ImageTagColumn::ImageId.eq(related_id)),
            )
            .exec(self.database())
            .await
            .map_err(Into::into)
    }

    async fn add_images(&self, id: i64, images: Vec<i64>) -> Result<u64> {
        if images.is_empty() {
            return Ok(0);
        }

        let image_tags = images.iter().map(|&image_id| ImageTagModelDto {
            tag_id: Set(id),
            image_id: Set(image_id),
        });

        let result = ImageTagEntity::insert_many(image_tags)
            .on_conflict(OnConflict::new().do_nothing().to_owned())
            .exec_without_returning(self.database())
            .await?;

        Ok(result)
    }

    async fn remove_images(&self, id: i64, images: Vec<i64>) -> Result<u64> {
        if images.is_empty() {
            return Ok(0);
        }

        let result = ImageTagEntity::delete_many()
            .filter(
                ImageTagColumn::TagId
                    .eq(id)
                    .and(ImageTagColumn::ImageId.is_in(images)),
            )
            .exec(self.database())
            .await?;

        Ok(result.rows_affected)
    }
}
