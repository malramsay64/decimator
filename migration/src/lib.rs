pub use sea_orm_migration::prelude::*;

mod m20230802_113601_create_pictures_table;
mod m20250319_000211_create_directory_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230802_113601_create_pictures_table::Migration),
            Box::new(m20250319_000211_create_directory_table::Migration),
        ]
    }
}
