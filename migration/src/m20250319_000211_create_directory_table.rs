use entity::prelude::*;
use sea_orm::Schema;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        let schema = Schema::new(backend);
        let table = Table::alter()
            .table(Picture)
            .add_column_if_not_exists(
                &mut schema.get_column_def::<Picture>(picture::Column::DirectoryId),
            )
            .take();
        manager.alter_table(table).await?;

        for mut index in schema.create_index_from_entity(Picture) {
            manager.create_index(index.if_not_exists().take()).await?
        }
        for mut index in schema.create_index_from_entity(Directory) {
            manager.create_index(index.if_not_exists().take()).await?
        }

        manager
            .create_table(
                schema
                    .create_table_from_entity(Directory)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        let schema = Schema::new(backend);
        let table = Table::alter()
            .table(Picture)
            .drop_column(Alias::new("directory_id"))
            .take();
        manager.alter_table(table).await?;

        manager
            .drop_table(Table::drop().table(Directory).to_owned())
            .await?;

        Ok(())
    }
}
