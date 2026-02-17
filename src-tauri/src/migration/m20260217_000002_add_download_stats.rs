use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DownloadTasks::Table)
                    .add_column(
                        ColumnDef::new(DownloadTasks::DownloadedBytes)
                            .big_integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(DownloadTasks::Table)
                    .add_column(
                        ColumnDef::new(DownloadTasks::TotalBytes)
                            .big_integer()
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
                    .table(DownloadTasks::Table)
                    .drop_column(DownloadTasks::DownloadedBytes)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(DownloadTasks::Table)
                    .drop_column(DownloadTasks::TotalBytes)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum DownloadTasks {
    Table,
    DownloadedBytes,
    TotalBytes,
}
