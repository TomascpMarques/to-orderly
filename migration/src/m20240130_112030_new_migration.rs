use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Template::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Template::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Template::Name).string().not_null())
                    .col(ColumnDef::new(Template::Description).string().not_null())
                    .col(ColumnDef::new(Template::Available).boolean().default(true))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-template_name")
                    .table(Template::Table)
                    .col(Template::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(Template::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx-template_name").to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Template {
    Table,
    Id,
    Name,
    Description,
    Available,
}
