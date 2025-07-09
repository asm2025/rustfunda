use async_trait::async_trait;
use sea_orm::ConnectionTrait;
use sea_orm_migration::prelude::*;
use sea_query::{OnConflict, Query};

use crate::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create images table
        manager
            .create_table(
                Table::create()
                    .table(Images::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Images::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Images::Filename).string().not_null())
                    .col(ColumnDef::new(Images::FileSize).big_integer().not_null())
                    .col(ColumnDef::new(Images::MimeType).string().not_null())
                    .col(ColumnDef::new(Images::Width).integer())
                    .col(ColumnDef::new(Images::Height).integer())
                    .col(ColumnDef::new(Images::AltText).string())
                    .col(ColumnDef::new(Images::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Images::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;
        // Create indexes for images table
        manager
            .create_index(
                Index::create()
                    .name("idx-images-filename")
                    .if_not_exists()
                    .table(Images::Table)
                    .col(Images::Filename)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx-images-mime_type")
                    .if_not_exists()
                    .table(Images::Table)
                    .col(Images::MimeType)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx-images-created_at")
                    .if_not_exists()
                    .table(Images::Table)
                    .col(Images::CreatedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx-images-updated_at")
                    .if_not_exists()
                    .table(Images::Table)
                    .col(Images::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        // Create tags table
        manager
            .create_table(
                Table::create()
                    .table(Tags::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tags::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tags::Name).string().not_null().unique_key())
                    .to_owned(),
            )
            .await?;

        // Create image_tags junction table
        manager
            .create_table(
                Table::create()
                    .table(ImageTags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ImageTags::ImageId).big_integer().not_null())
                    .col(ColumnDef::new(ImageTags::TagId).big_integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(ImageTags::ImageId)
                            .col(ImageTags::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-image_tags-image_id")
                            .from(ImageTags::Table, ImageTags::ImageId)
                            .to(Images::Table, Images::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-image_tags-tag_id")
                            .from(ImageTags::Table, ImageTags::TagId)
                            .to(Tags::Table, Tags::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Insert default tags
        let default_tags = [
            "landscape",
            "portrait",
            "nature",
            "urban",
            "black-and-white",
            "color",
            "macro",
            "street",
            "architecture",
            "people",
        ];
        let db = manager.get_connection();
        let backend = db.get_database_backend();
        let mut insert_stmt = Query::insert()
            .into_table(Tags::Table)
            .columns([Tags::Name])
            .to_owned();

        for tag in default_tags {
            insert_stmt.values([tag.into()]).unwrap();
        }

        let insert_stmt =
            insert_stmt.on_conflict(OnConflict::column(Tags::Name).do_nothing().to_owned());
        db.execute(backend.build(insert_stmt)).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ImageTags::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Tags::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Images::Table).to_owned())
            .await?;

        Ok(())
    }
}
