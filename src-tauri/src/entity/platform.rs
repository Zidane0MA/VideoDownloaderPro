use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "platforms")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub icon_path: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::creator::Entity")]
    Creators,
    #[sea_orm(has_many = "super::source::Entity")]
    Sources,
    #[sea_orm(has_one = "super::platform_session::Entity")]
    PlatformSession,
}

impl Related<super::creator::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Creators.def()
    }
}

impl Related<super::source::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sources.def()
    }
}

impl Related<super::platform_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlatformSession.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
