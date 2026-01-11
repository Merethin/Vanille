use log::warn;
use serde::{Serialize, Deserialize};
use serenity::all::{ChannelId, Timestamp, UserId};
use sqlx::{FromRow, Row};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ReportEntry {
    pub name: String,
    pub event: String,
    pub origin: String,
    #[serde(skip_serializing)]
    #[sqlx(try_from = "i64")]
    pub queue: u64,
    pub queue_time: i64,
    #[sqlx(try_from = "i64")]
    pub recruiter: u64,
    pub sender: String,
    pub template: String,
    pub sent_time: i64,
    pub moved: bool,
    pub moved_time: Option<i64>,
}

impl ReportEntry {
    pub fn new(
        name: String,
        event: String,
        origin: String,
        queue: ChannelId,
        queue_time: Timestamp,
        recruiter: UserId,
        sender: String,
        template: String,
        sent_time: Timestamp,
    ) -> Self {
        Self {
            name,
            event,
            origin,
            queue: queue.get(),
            queue_time: queue_time.timestamp(),
            recruiter: recruiter.get(),
            sender,
            template,
            sent_time: sent_time.timestamp(),
            moved: false,
            moved_time: None
        }
    }

    pub async fn insert(&self, pool: &sqlx::PgPool) {
        let result = sqlx::query(
           "INSERT INTO delivery_reports (name, event, origin, queue, queue_time, recruiter, sender, template, sent_time)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
            ).bind(&self.name)
            .bind(&self.event)
            .bind(&self.origin)
            .bind(self.queue as i64)
            .bind(self.queue_time)
            .bind(self.recruiter as i64)
            .bind(&self.sender)
            .bind(&self.template)
            .bind(self.sent_time)
            .execute(pool).await;

        if result.is_err() {
            warn!("Failed to save delivery report '{:?}' to Postgres database - {:?}", self, result);
        }
    }

    pub async fn count(
        pool: &sqlx::PgPool,
        queue: ChannelId,
        range: Option<(u64, u64)>
    ) -> Result<Vec<(String, usize)>, sqlx::Error> {
        let rows = if let Some((start, end)) = range {
            sqlx::query(
            "SELECT sender, COUNT(*) AS sender_count FROM delivery_reports
                WHERE queue = $1 AND sent_time BETWEEN $2 AND $3
                GROUP BY sender ORDER BY sender_count DESC"
            )
            .bind(queue.get() as i64)
            .bind(start as i64)
            .bind(end as i64)   
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query(
        "SELECT sender, COUNT(*) AS sender_count FROM delivery_reports
            WHERE queue = $1 GROUP BY sender ORDER BY sender_count DESC"
            )
            .bind(queue.get() as i64)
            .fetch_all(pool)
            .await?
        };

        Ok(rows.iter().map(
            |row| (
                row.get::<String, &str>("sender"),
                row.get::<i64, &str>("sender_count") as usize,
            )
        ).collect())
    }

    pub async fn query(
        pool: &sqlx::PgPool,
        queue: ChannelId,
        range: Option<(u64, u64)>
    ) -> Result<Vec<ReportEntry>, sqlx::Error> {
        if let Some((start, end)) = range {
            sqlx::query_as(
        "SELECT name, event, origin, queue, queue_time, recruiter, sender, template, sent_time, moved, moved_time
                FROM delivery_reports WHERE queue = $1 AND sent_time BETWEEN $2 AND $3"
            ).bind(queue.get() as i64)
            .bind(start as i64)
            .bind(end as i64)
            .fetch_all(pool).await
        } else {
            sqlx::query_as(
        "SELECT name, event, origin, queue, queue_time, recruiter, sender, template, sent_time, moved, moved_time
                FROM delivery_reports WHERE queue = $1"
            ).bind(queue.get() as i64).fetch_all(pool).await
        }
    }
    
    pub async fn mark_move(
        pool: &sqlx::PgPool,
        queue: ChannelId,
        nation: &String,
        move_time: u64,
    ) {
        let result = sqlx::query(
        "WITH latest AS (
                SELECT id
                FROM delivery_reports
                WHERE queue = $1 AND name = $2
                ORDER BY sent_time DESC LIMIT 1
            ) UPDATE delivery_reports
            SET moved = TRUE, moved_time = $3 FROM latest 
            WHERE delivery_reports.id = latest.id 
            AND delivery_reports.moved = FALSE"
        )
        .bind(queue.get() as i64)
        .bind(nation)
        .bind(move_time as i64)   
        .execute(pool)
        .await;

        if result.is_err() {
            warn!("Failed to mark move for nation '{}', queue {} in Postgres database - {:?}", nation, queue.get(), result);
        }
    }
}