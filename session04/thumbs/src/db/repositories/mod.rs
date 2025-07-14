use anyhow::Result;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DeleteResult, EntityTrait, ModelTrait};

mod image_repository;
mod tag_repository;

pub use image_repository::*;
pub use tag_repository::*;

use crate::db::{FilterCondition, FilterRelatedCondition, Merge, Pagination, ResultSet};

#[async_trait]
pub trait IHasDatabase {
    fn database(&self) -> &DatabaseConnection;
}

#[async_trait]
pub trait IRepository: IHasDatabase {
    type Entity: EntityTrait + Send + Sync;
    type PrimaryKey: Send + Sync;
    type Model: ModelTrait + Send + Sync;
    type ActiveModel: ActiveModelTrait + Send + Sync;
    type UpdateModel: Merge<Self::ActiveModel> + Send + Sync;

    async fn list<F>(
        &self,
        filter: Option<F>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<Self::Model>>
    where
        F: FilterCondition<Self::Entity> + Send + Sync;
    async fn count<F>(&self, filter: Option<F>) -> Result<u64>
    where
        F: FilterCondition<Self::Entity> + Send + Sync;
    async fn get(&self, id: Self::PrimaryKey) -> Result<Option<Self::Model>>;
    async fn create(&self, model: Self::Model) -> Result<Self::Model>;
    async fn update(&self, id: Self::PrimaryKey, model: Self::UpdateModel) -> Result<Self::Model>;
    async fn delete(&self, id: Self::PrimaryKey) -> Result<DeleteResult>;
}

#[async_trait]
pub trait IRepositoryWithRelated: IRepository {
    type Related: EntityTrait;

    async fn list_with_related<F, R>(
        &self,
        filter: Option<F>,
        filter_related: Option<R>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<(Self::Model, Vec<<Self::Related as EntityTrait>::Model>)>>
    where
        F: FilterCondition<Self::Entity> + Send + Sync,
        R: FilterRelatedCondition<Self::Entity, Self::Related> + Send + Sync;
}
