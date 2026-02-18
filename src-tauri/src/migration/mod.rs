use sea_orm_migration::prelude::*;

mod m20260217_000001_initial_schema;
mod m20260217_000002_add_download_stats;
mod m20260219_000001_add_username_to_sessions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260217_000001_initial_schema::Migration),
            Box::new(m20260217_000002_add_download_stats::Migration),
            Box::new(m20260219_000001_add_username_to_sessions::Migration),
        ]
    }
}
