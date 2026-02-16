use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sources")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub platform_id: String,
    pub creator_id: Option<String>,
    #[sea_orm(column_name = "type")]
    pub source_type: String,
    pub name: String,
    pub url: String,
    pub sync_mode: String,
    pub date_start: Option<DateTimeUtc>,
    pub date_end: Option<DateTimeUtc>,
    pub max_items: Option<i32>,
    pub last_checked: Option<DateTimeUtc>,
    pub is_active: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::platform::Entity",
        from = "Column::PlatformId",
        to = "super::platform::Column::Id"
    )]
    Platform,
    #[sea_orm(
        belongs_to = "super::creator::Entity",
        from = "Column::CreatorId",
        to = "super::creator::Column::Id"
    )]
    Creator,
    #[sea_orm(has_many = "super::post::Entity")]
    Posts,
}

impl Related<super::platform::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Platform.def()
    }
}

impl Related<super::creator::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Creator.def()
    }
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Posts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
