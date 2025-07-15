use anyhow::Result;
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DeleteResult, EntityTrait, PrimaryKeyTrait};

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
pub trait IRepository<E, U>: IHasDatabase
where
    E: EntityTrait + Send + Sync,
    U: Merge<<E as EntityTrait>::ActiveModel> + Send + Sync,
{
    async fn list(
        &self,
        filter: Option<Box<dyn FilterCondition<E> + Send + Sync>>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<<E as EntityTrait>::Model>>;
    async fn count(&self, filter: Option<Box<dyn FilterCondition<E> + Send + Sync>>)
    -> Result<u64>;
    async fn get(
        &self,
        id: <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Result<Option<<E as EntityTrait>::Model>>;
    async fn create(&self, model: <E as EntityTrait>::Model) -> Result<<E as EntityTrait>::Model>;
    async fn update(
        &self,
        id: <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
        model: U,
    ) -> Result<<E as EntityTrait>::Model>;
    async fn delete(
        &self,
        id: <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Result<DeleteResult>;
}

#[async_trait]
pub trait IRepositoryWithRelated<E, U, R>: IRepository<E, U>
where
    E: EntityTrait + Send + Sync,
    U: Merge<<E as EntityTrait>::ActiveModel> + Send + Sync,
    R: EntityTrait + Send + Sync,
{
    async fn list_with_related(
        &self,
        filter: Option<Box<dyn FilterCondition<E> + Send + Sync>>,
        filter_related: Option<Box<dyn FilterRelatedCondition<E, R> + Send + Sync>>,
        pagination: Option<Pagination>,
    ) -> Result<ResultSet<(<E as EntityTrait>::Model, Vec<<R as EntityTrait>::Model>)>>;
}
