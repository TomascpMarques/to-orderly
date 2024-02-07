use entity::template;
use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveModelTrait, Set},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        template::ActiveModel {
            name: Set("First".into()),
            description: Set("First template".into()),
            ..Default::default()
        }
        .insert(db)
        .await?;
        template::ActiveModel {
            name: Set("Second".into()),
            description: Set("Second".into()),
            ..Default::default()
        }
        .insert(db)
        .await?;
        template::ActiveModel {
            name: Set("Third".into()),
            description: Set("Third Template".into()),
            ..Default::default()
        }
        .insert(db)
        .await?;
        template::ActiveModel {
            name: Set("Fourth".into()),
            description: Set("Fourth Template".into()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        Ok(())
    }
}
