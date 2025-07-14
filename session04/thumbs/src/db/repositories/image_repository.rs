use anyhow::Result;
use async_trait::async_trait;
use migration::OnConflict;
use sea_orm::{DeleteResult, JoinType, PaginatorTrait, QuerySelect, Set, prelude::*};

use crate::db::prelude::*;

#[async_trait]
pub trait IImageRepository: IRepositoryWithRelated {
    async fn create_with_tags(&self, model: CreateImageDto) -> Result<Self::Model>;
    async fn list_tags<F>(
        &self,
        id: i64,
        filter: Option<F>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<<Self::Related as EntityTrait>::Model>>
    where
        F: FilterCondition<Self::Related> + Send + Sync;
    async fn add_tag(&self, id: i64, tag_id: i64) -> Result<()>;
    async fn remove_tag(&self, id: i64, tag_id: i64) -> Result<DeleteResult>;
    async fn add_tags(&self, id: i64, tags: &str) -> Result<u64>;
    async fn add_many_tags(&self, id: i64, tags: &[i64]) -> Result<u64>;
    async fn remove_many_tags(&self, id: i64, tags: &[i64]) -> Result<u64>;
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
    type PrimaryKey = <<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType;
    type Model = <Self::Entity as EntityTrait>::Model;
    type ActiveModel = <Self::Entity as EntityTrait>::ActiveModel;
    type UpdateModel = UpdateImageDto;

    async fn list<F>(
        &self,
        filter: Option<F>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<Self::Model>>
    where
        F: FilterCondition<Self::Entity> + Send + Sync,
    {
        let mut query = <Self::Entity as EntityTrait>::find();

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
        let mut query = <Self::Entity as EntityTrait>::find();

        if let Some(f) = &filter {
            query = f.apply(query);
        }

        query.count(self.database()).await.map_err(Into::into)
    }

    async fn get(&self, id: Self::PrimaryKey) -> Result<Option<Self::Model>> {
        Self::Entity::find_by_id(id)
            .one(self.database())
            .await
            .map_err(Into::into)
    }

    async fn create(&self, model: Self::Model) -> Result<Self::Model> {
        let active_model: Self::ActiveModel = model.into();
        active_model
            .insert(self.database())
            .await
            .map_err(Into::into)
    }

    async fn update(&self, id: Self::PrimaryKey, model: Self::UpdateModel) -> Result<Self::Model> {
        let existing = Self::Entity::find_by_id(id)
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

    async fn delete(&self, id: Self::PrimaryKey) -> Result<DeleteResult> {
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
        let mut query = <Self::Entity as EntityTrait>::find();

        if let Some(f) = &filter {
            query = f.apply(query);
        }

        let count_query = query.clone();
        let total = count_query.count(self.database()).await?;
        let mut query = query.find_with_related(TagEntity);

        if let Some(r) = &filter_related {
            query = r.apply(query);
        }

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
    async fn create_with_tags(&self, model: CreateImageDto) -> Result<Self::Model> {
        let tags = model.tags.clone();
        let active_model: Self::ActiveModel = model.into();
        let result = active_model.insert(self.database()).await?;
        let Some(tags) = tags else {
            return Ok(result);
        };
        self.add_tags(result.id, &tags).await?;
        Ok(result)
    }

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

    async fn add_tags(&self, id: i64, tags: &str) -> Result<u64> {
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

    async fn add_many_tags(&self, id: i64, tags: &[i64]) -> Result<u64> {
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

    async fn remove_many_tags(&self, id: i64, tags: &[i64]) -> Result<u64> {
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
}
