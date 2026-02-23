use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PlatformSessions::Table)
                    .add_column(ColumnDef::new(PlatformSessions::AvatarUrl).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PlatformSessions::Table)
                    .add_column(
                        ColumnDef::new(PlatformSessions::ErrorMessage)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PlatformSessions::Table)
                    .drop_column(PlatformSessions::AvatarUrl)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PlatformSessions::Table)
                    .drop_column(PlatformSessions::ErrorMessage)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum PlatformSessions {
    Table,
    AvatarUrl,
    ErrorMessage,
}
