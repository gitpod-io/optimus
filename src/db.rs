use anyhow::{Context, Result};
use serenity::{
    async_trait, client,
    model::id::{RoleId, UserId},
    prelude::TypeMapKey,
};
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

// pub struct User {
//     userid: UserId,
//     roles: Vec<RoleId>,
// }

impl Db {
    pub async fn set_user_roles(&self, user_id: UserId, roles: Vec<RoleId>) -> Result<()> {
        let user_id = user_id.0 as i64;
        let roles = serde_json::to_string(&roles)?;
        sqlx::query!("insert into user_profile (user_id, roles) values (?1, ?2) on conflict(user_id) do update set roles=?2", user_id, roles).execute(&self.sqlitedb).await?;
        Ok(())
    }
    // pub async fn get_user_roles(&self, user_id: UserId) -> Result<Option<User>> {
    //     let user_id = user_id.0 as i64;
    //     let result = sqlx::query!("select * from user_profile where user_id=?", user_id)
    //         .fetch_optional(&self.sqlitedb)
    //         .await?;
    //     if let Some(x) = result {
    //         Ok(Some(User {
    //             userid: UserId(x.user_id as u64),
    //             roles: serde_json::from_str(&x.roles).context("Failed to get roles")?,
    //         }))
    //     } else {
    //         Ok(None)
    //     }
    // }
}
