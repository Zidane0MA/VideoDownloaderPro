use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub creator_id: String,
    pub source_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub original_url: String,
    pub status: String,
    pub posted_at: Option<DateTimeUtc>,
    pub downloaded_at: Option<DateTimeUtc>,
    pub deleted_at: Option<DateTimeUtc>,
    #[sea_orm(column_type = "Text")]
    pub raw_json: Option<String>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::creator::Entity",
        from = "Column::CreatorId",
        to = "super::creator::Column::Id"
    )]
    Creator,
    #[sea_orm(
        belongs_to = "super::source::Entity",
        from = "Column::SourceId",
        to = "super::source::Column::Id"
    )]
    Source,
    #[sea_orm(has_many = "super::media::Entity")]
    Media,
    #[sea_orm(has_one = "super::download_task::Entity")]
    DownloadTask,
}

impl Related<super::creator::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Creator.def()
    }
}

impl Related<super::source::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Source.def()
    }
}

impl Related<super::media::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Media.def()
    }
}

impl Related<super::download_task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DownloadTask.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
