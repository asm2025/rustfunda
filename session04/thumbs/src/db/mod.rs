use sea_orm::{Condition, EntityTrait, QueryFilter, Select, SelectTwoMany};
use serde::{Deserialize, Serialize};

pub mod entities;
pub mod repositories;
pub mod prelude {
    pub use super::entities::*;
    pub use super::repositories::*;
    pub use super::*;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u64,
    pub page_size: u64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSet<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub pagination: Option<Pagination>,
}

impl Default for ResultSet<()> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            total: 0,
            pagination: None,
        }
    }
}

pub struct ClosureFilter<F>
where
    F: Fn() -> Condition,
{
    condition: F,
}

impl<F> ClosureFilter<F>
where
    F: Fn() -> Condition,
{
    pub fn new(condition: F) -> Self {
        Self { condition }
    }
}

pub struct DirectCondition(pub Condition);

pub trait FilterCondition<E: EntityTrait> {
    fn apply(&self, query: Select<E>) -> Select<E>;
}

impl<E: EntityTrait> FilterCondition<E> for Condition {
    fn apply(&self, query: Select<E>) -> Select<E> {
        query.filter(self.clone())
    }
}

impl<E: EntityTrait, F> FilterCondition<E> for F
where
    F: Fn(Select<E>) -> Select<E>,
{
    fn apply(&self, query: Select<E>) -> Select<E> {
        self(query)
    }
}

impl<E: EntityTrait, F> FilterCondition<E> for ClosureFilter<F>
where
    F: Fn() -> Condition,
{
    fn apply(&self, query: Select<E>) -> Select<E> {
        query.filter((self.condition)())
    }
}

impl<E: EntityTrait> FilterCondition<E> for DirectCondition {
    fn apply(&self, query: Select<E>) -> Select<E> {
        query.filter(self.0.clone())
    }
}

pub trait FilterRelatedCondition<E: EntityTrait, R: EntityTrait> {
    fn apply(&self, query: SelectTwoMany<E, R>) -> SelectTwoMany<E, R>;
}

impl<E: EntityTrait, R: EntityTrait> FilterRelatedCondition<E, R> for Condition {
    fn apply(&self, query: SelectTwoMany<E, R>) -> SelectTwoMany<E, R> {
        query.filter(self.clone())
    }
}

impl<E: EntityTrait, R: EntityTrait, F> FilterRelatedCondition<E, R> for F
where
    F: Fn(SelectTwoMany<E, R>) -> SelectTwoMany<E, R>,
{
    fn apply(&self, query: SelectTwoMany<E, R>) -> SelectTwoMany<E, R> {
        self(query)
    }
}

impl<E: EntityTrait, R: EntityTrait, F> FilterRelatedCondition<E, R> for ClosureFilter<F>
where
    F: Fn() -> Condition,
{
    fn apply(&self, query: SelectTwoMany<E, R>) -> SelectTwoMany<E, R> {
        query.filter((self.condition)())
    }
}

impl<E: EntityTrait, R: EntityTrait> FilterRelatedCondition<E, R> for DirectCondition {
    fn apply(&self, query: SelectTwoMany<E, R>) -> SelectTwoMany<E, R> {
        query.filter(self.0.clone())
    }
}

pub trait Merge<T> {
    fn merge(&self, model: &mut T);
}
