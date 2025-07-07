pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;

#[derive(DeriveIden)]
pub enum Images {
    Table,
    Id,
    Filename,
    FileSize,
    MimeType,
    Width,
    Height,
    AltText,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Tags {
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
pub enum ImageTags {
    Table,
    ImageId,
    TagId,
}

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220101_000001_create_table::Migration)]
    }
}
