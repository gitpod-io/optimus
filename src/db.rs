use anyhow::{Context, Result};
use serenity::{async_trait, client, prelude::TypeMapKey};
use std::sync::Arc;

pub struct Db {
    pub sqlitedb: sqlx::sqlite::SqlitePool,
}

impl TypeMapKey for Db {
    type Value = Arc<Db>;
}

impl Db {
    pub async fn new() -> Result<Self> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename("bot.sqlite")
                    .create_if_missing(true),
            )
            .await?;
        Ok(Self { sqlitedb: pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.sqlitedb)
            .await
            .context("Failed to run database migrations")?;
        Ok(())
    }
}

#[async_trait]
pub trait ClientContextExt {
    async fn get_db(&self) -> Arc<Db>;
}

#[async_trait]
impl ClientContextExt for client::Context {
    async fn get_db(&self) -> Arc<Db> {
        self.data.read().await.get::<Db>().unwrap().clone()
    }
}
