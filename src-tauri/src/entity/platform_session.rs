use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "platform_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub platform_id: String,
    pub status: String,
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub encrypted_cookies: Option<Vec<u8>>,
    pub cookie_method: String,
    pub expires_at: Option<DateTimeUtc>,
    pub last_verified: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::platform::Entity",
        from = "Column::PlatformId",
        to = "super::platform::Column::Id"
    )]
    Platform,
}

impl Related<super::platform::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Platform.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
