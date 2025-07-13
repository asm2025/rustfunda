use anyhow::Result;
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DeleteResult, EntityTrait};

mod image_repository;
mod tag_repository;

pub use image_repository::*;
pub use tag_repository::*;

use crate::db::{FilterCondition, FilterRelatedCondition, Pagination, ResultSet};

#[async_trait]
pub trait IHasDatabase {
    fn database(&self) -> &DatabaseConnection;
}

#[async_trait]
pub trait IRepository: IHasDatabase {
    type Entity: EntityTrait;
    type Model: Send + Sync;
    type Key: Send + Sync;
    type CreateDto: Send + Sync;
    type UpdateDto: Send + Sync;

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
    async fn get(&self, id: Self::Key) -> Result<Option<Self::Model>>;
    async fn create(&self, model: Self::CreateDto) -> Result<Self::Model>;
    async fn update(&self, id: Self::Key, model: Self::UpdateDto) -> Result<Self::Model>;
    async fn delete(&self, id: Self::Key) -> Result<DeleteResult>;
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
