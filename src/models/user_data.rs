use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serenity::all::{ChannelId, UserId};
use sqlx::FromRow;
use log::warn;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserData {
    #[sqlx(try_from = "i64")]
    pub queue: u64,
    #[sqlx(try_from = "i64")]
    pub user_id: u64,
    pub nation: String,
    pub founded: i64,
    pub newfounds: Vec<String>,
    pub refounds: Vec<String>,
}

impl UserData {
    pub fn new(
        queue: ChannelId,
        user: UserId, 
        nation: String,
        founded: i64,
        newfounds: Vec<String>,
        refounds: Vec<String>,
    ) -> Self {
        Self {
            queue: queue.get(),
            user_id: user.get(),
            nation,
            founded,
            newfounds,
            refounds
        }
    }

    pub async fn insert(&self, pool: &sqlx::PgPool) {
        let result = sqlx::query(
           "INSERT INTO user_data (queue, user_id, nation, founded, newfounds, refounds)
                VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (queue, user_id) DO UPDATE
                SET nation = EXCLUDED.nation,
                founded = EXCLUDED.founded,
                newfounds = EXCLUDED.newfounds,
                refounds = EXCLUDED.refounds"
            ).bind(self.queue as i64)
            .bind(self.user_id as i64)
            .bind(&self.nation)
            .bind(&self.founded)
            .bind(&self.newfounds)
            .bind(&self.refounds)
            .execute(pool).await;

        if result.is_err() {
            warn!("Failed to save user data '{:?}' to Postgres database - {:?}", self, result);
        }
    }

    pub async fn remove_matching(
        queue: i64,
        pool: &sqlx::PgPool
    ) {
        let result = sqlx::query(
           "DELETE FROM user_data WHERE queue = $1"
            ).bind(queue as i64)
            .execute(pool).await;

        if result.is_err() {
            warn!("Failed to delete user data for queue '{:?}' from Postgres database - {:?}", queue, result);
        }
    }

    pub async fn query(
        pool: &sqlx::PgPool,
    ) -> Result<HashMap<(ChannelId, UserId), UserData>, sqlx::Error> {
        let vec = sqlx::query_as::<_, UserData>(
            "SELECT queue, user_id, nation, founded, newfounds, refounds FROM user_data"
        ).fetch_all(pool).await?;

        let mut map = HashMap::new();
        for value in vec {
            map.insert((ChannelId::new(value.queue), UserId::new(value.user_id)), value);
        }

        Ok(map)
    }
}