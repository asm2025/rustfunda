pub mod image;
pub mod image_tag;
pub mod tag;

pub mod prelude {
    pub use super::image::ActiveModel as ImageModelDto;
    pub use super::image::Column as ImageColumn;
    pub use super::image::Entity as Image;
    pub use super::image::Model as ImageModel;

    pub use super::tag::ActiveModel as TagModelDto;
    pub use super::tag::Column as TagColumn;
    pub use super::tag::Entity as Tag;
    pub use super::tag::Model as TagModel;

    pub use super::image_tag::ActiveModel as ImageTagModelDto;
    pub use super::image_tag::Column as ImageTagColumn;
    pub use super::image_tag::Entity as ImageTag;
    pub use super::image_tag::Model as ImageTagModel;

    pub use sea_orm::{ActiveModelTrait, EntityTrait, QueryFilter, Set};
}
