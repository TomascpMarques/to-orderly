pub use sea_orm_migration::prelude::*;

mod m20240130_112030_new_migration;
mod m20240130_215925_seed_data;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240130_112030_new_migration::Migration),
            Box::new(m20240130_215925_seed_data::Migration),
        ]
    }
}
