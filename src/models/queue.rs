use std::collections::{HashMap, HashSet, VecDeque};
use caramel::ns::format::prettify_name;
use itertools::Itertools;
use log::warn;
use lazy_static::lazy_static;
use regex::Regex;

use serenity::all::{
    CacheHttp, ChannelId, Context, EditMessage, Mentionable, 
    MessageId, RoleId, Timestamp, CreateEmbed, CreateActionRow
};

use sqlx::{prelude::FromRow, Row};

use crate::{embeds::create_queue_embed, models::user_data::UserData};

#[derive(Debug, Default)]
pub struct Filter {
    pub regions: Vec<String>,
}

impl Filter {
    pub fn matches(&self, region: &str) -> bool {
        !self.regions.iter().any(|v| v == region)
    }
}

#[derive(Debug)]
pub struct Nation {
    pub name: String,
    pub region: String,
    pub event: String,
    pub queue_time: Timestamp,
}

#[derive(Debug, Default)]
struct QueueImpl {
    nations: VecDeque<Nation>,
    dedup: HashSet<String>,
}

#[derive(Debug)]
pub struct QueueMessageUpdate {
    embed: CreateEmbed,
    components: Vec<CreateActionRow>,
    channel: ChannelId,
    message: MessageId,
}

impl QueueMessageUpdate {
    pub async fn execute(self, ctx: Context) {
        match ctx.http().get_message(self.channel, self.message).await {
            Ok(mut message) => { 
                if let Err(err) = message.edit(
        ctx.http(), EditMessage::new().embed(self.embed).components(self.components)
                ).await {
                    warn!("Failed to update queue message: {}", err);
                }
            },
            Err(err) => {
                warn!("Failed to fetch queue message: {}", err);
            }
        }
    }
}

pub const QUEUE_TELEGRAM_BUFFER: i64 = 5; // 5 seconds past normal telegram cooldown
const REMINDER_COOLDOWN: i64 = 6 * 3600; // Four hours at least between each reminder ping.

#[derive(Debug, FromRow)]
pub struct Queue {
    pub channel: ChannelId,
    pub message: MessageId,
    pub region: String,
    pub filter: Filter,
    pub size: usize,
    pub thresholds: Option<(u64, u64)>,
    pub ping_channel: Option<ChannelId>,
    pub ping_role: Option<RoleId>,
    #[sqlx(skip)]
    queue: QueueImpl,
    #[sqlx(skip)]
    last_update: Timestamp,
    #[sqlx(skip)]
    last_telegram: Timestamp,
    #[sqlx(skip)]
    last_reminder: Timestamp,
}

lazy_static! {
    static ref NUMBER_RE: Regex = Regex::new(r#"^[0-9a-z_-]+[0-9]+$"#).unwrap();
    static ref ROMAN_RE: Regex = Regex::new(r#"^[0-9a-z_-]+_m{0,4}(cm|cd|d?c{0,3})(xc|xl|l?x{0,3})(ix|iv|v?i{0,3})$"#).unwrap();
}

impl Queue {
    pub fn new(
        channel: ChannelId,
        message: MessageId,
        region: String,
        filter: Filter,
        size: usize,
    ) -> Self {
        Self {
            channel,
            message,
            region,
            filter,
            size,
            thresholds: None,
            ping_channel: None,
            ping_role: None,
            queue: QueueImpl::default(),
            last_update: Timestamp::now(),
            last_telegram: Timestamp::now(),
            last_reminder: Timestamp::now(),
        }
    }

    pub fn amount_in_queue(&self) -> usize {
        self.queue.nations.len()
    }

    pub fn last_updated(&self) -> Timestamp {
        self.last_update
    }

    pub fn last_telegram_sent(&self) -> Timestamp {
        self.last_telegram
    }

    pub fn add(&mut self, nation: Nation) -> bool {
        if self.queue.dedup.insert(nation.name.clone()) {
            self.queue.nations.push_back(nation);

            if self.queue.nations.len() > self.size {
                if let Some(old_nation) = self.queue.nations.pop_front() {
                    self.queue.dedup.remove(&old_nation.name);
                }
            }

            self.last_update = Timestamp::now();

            true
        } else {
            false
        }
    }

    pub fn pull(
        &mut self, data: &UserData, mut limit: usize
    ) -> (Vec<Nation>, Vec<String>, Option<QueueMessageUpdate>) {
        if self.queue.nations.is_empty() { return (vec![], vec![], None); }

        let mut indexes: Vec<usize> = Vec::new();
        let mut pos: usize = self.queue.nations.len() - 1;

        let mut eligible_templates: Option<Vec<String>> = None;

        while limit > 0 {
            if let Some(nation) = self.queue.nations.get(pos) {
                if let Some(el_templates) = &mut eligible_templates {
                    let intersection: Vec<String> = match nation.event.as_str() {
                        "newfound" => Some(&data.newfounds),
                        "refound" => Some(&data.refounds),
                        _ => None,
                    }.and_then(
                        |v| Some(v.iter().filter(
                            |v| el_templates.contains(v)
                        ).cloned().collect())
                    ).unwrap_or(vec![]);

                    if !intersection.is_empty() {
                        eligible_templates = Some(intersection);
                        limit -= 1;
                        indexes.push(pos);
                    }
                } else {
                    eligible_templates = match nation.event.as_str() {
                        "newfound" => Some(&data.newfounds),
                        "refound" => Some(&data.refounds),
                        _ => None,
                    }.cloned();

                    if !eligible_templates.as_ref().and_then(|v| Some(v.is_empty())).unwrap_or(true) {
                        limit -= 1;
                        indexes.push(pos);
                    }
                }
            }

            if pos == 0 { break; }
            pos -= 1;
        }

        let (nations, templates) = (indexes.into_iter().sorted().rev().flat_map(|index| {
            if let Some(nation) = self.queue.nations.remove(index) {
                self.queue.dedup.remove(&nation.name);
                Some(nation)
            } else {
                None
            }
        }).collect::<Vec<_>>(), eligible_templates.unwrap_or(vec![]));

        let update = if !nations.is_empty() {
            self.last_telegram = Timestamp::now();

            Some(self.generate_queue_update())
        } else {
            None
        };

        (nations, templates, update)
    }

    pub async fn query(
        pool: &sqlx::PgPool,
    ) -> Result<HashMap<ChannelId, Queue>, sqlx::Error> {
        let vec = sqlx::query(
       "SELECT channel_id, message_id, region, size, excluded_regions, 
            fill_threshold, time_threshold, ping_channel, ping_role FROM queues"
        ).fetch_all(pool).await?;

        let mut map = HashMap::new();
        for value in vec {
            let channel = ChannelId::new(value.get::<i64, &str>("channel_id") as u64);
            let fill_threshold = value.get::<Option<i64>, &str>("fill_threshold").and_then(|v| Some(v as u64));
            let time_threshold = value.get::<Option<i64>, &str>("time_threshold").and_then(|v| Some(v as u64));
            map.insert(
                channel,
                Queue {
                    channel,
                    message: MessageId::new(value.get::<i64, &str>("message_id") as u64),
                    region: value.get::<String, &str>("region"),
                    filter: Filter { 
                        regions: value.get::<Vec<String>, &str>("excluded_regions"),
                    },
                    size: value.get::<i64, &str>("size") as usize,
                    thresholds: fill_threshold.zip(time_threshold),
                    ping_channel: value.get::<Option<i64>, &str>("ping_channel").and_then(|v| Some(ChannelId::new(v as u64))),
                    ping_role: value.get::<Option<i64>, &str>("ping_role").and_then(|v| Some(RoleId::new(v as u64))),
                    queue: QueueImpl::default(),
                    last_update: Timestamp::now(),
                    last_telegram: Timestamp::now(),
                    last_reminder: Timestamp::now(),
                }
            );
        }

        Ok(map)
    }

    pub async fn insert(
        &self,
        pool: &sqlx::PgPool
    ) {
        let result = sqlx::query(
           "INSERT INTO queues (channel_id, message_id, region, size, excluded_regions, 
                fill_threshold, time_threshold, ping_channel, ping_role)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (channel_id) DO UPDATE
                SET message_id = EXCLUDED.message_id,
                region = EXCLUDED.region,
                size = EXCLUDED.size,
                excluded_regions = EXCLUDED.excluded_regions,
                fill_threshold = EXCLUDED.fill_threshold,
                time_threshold = EXCLUDED.time_threshold,
                ping_channel = EXCLUDED.ping_channel,
                ping_role = EXCLUDED.ping_role"
            ).bind(self.channel.get() as i64)
            .bind(self.message.get() as i64)
            .bind(&self.region)
            .bind(self.size as i64)
            .bind(&self.filter.regions)
            .bind(self.thresholds.and_then(|v| Some(v.0 as i64)))
            .bind(self.thresholds.and_then(|v| Some(v.1 as i64)))
            .bind(self.ping_channel.and_then(|v| Some(v.get() as i64)))
            .bind(self.ping_role.and_then(|v| Some(v.get() as i64)))
            .execute(pool).await;

        if result.is_err() {
            warn!("Failed to save queue '{:?}' to Postgres database - {:?}", self, result);
        }
    }

    pub async fn remove(
        &self,
        pool: &sqlx::PgPool
    ) {
        let result = sqlx::query(
           "DELETE FROM queues WHERE channel_id = $1"
            ).bind(self.channel.get() as i64)
            .execute(pool).await;

        if result.is_err() {
            warn!("Failed to delete queue '{:?}' from Postgres database - {:?}", self, result);
        }
    }

    pub fn generate_queue_update(&self) -> QueueMessageUpdate {
        let (embed, components) = create_queue_embed(self);
        QueueMessageUpdate { embed, components, channel: self.channel, message: self.message }
    }

    pub fn add_to_queue(
        &mut self,
        nation: &str,
        event: &str,
        region: &str,
    ) -> Option<QueueMessageUpdate> {
        if region == self.region 
        || !self.filter.matches(region) {
            return None;
        }

        if NUMBER_RE.is_match(nation)
        || ROMAN_RE.is_match(nation) {
            return None;
        }

        if self.add(
            Nation { 
                name: nation.to_owned(), region: region.to_owned(), 
                event: event.to_owned(), queue_time: Timestamp::now() 
            }
        ) {
            Some(self.generate_queue_update())
        } else {
            None
        }
    }

    pub fn make_reminder_if_needed(&mut self) -> Option<(ChannelId, String)> {
        let Some((fill_threshold, time_threshold)) = self.thresholds else {
            return None;
        };

        let Some(channel) = self.ping_channel else {
            return None;
        };

        let Some(role) = self.ping_role else {
            return None;
        };

        let now = Timestamp::now();
        let time_since_last_reminder = now.timestamp() - self.last_reminder.timestamp();
        let time_since_last_telegram = now.timestamp() - self.last_telegram.timestamp();

        if time_since_last_reminder < REMINDER_COOLDOWN
        || time_since_last_telegram < (time_threshold * 60) as i64
        || self.queue.nations.len() < fill_threshold as usize {
            return None;
        }

        self.last_reminder = now;

        Some((
            channel,
            format!(
                "{} - it's been over {} minutes since someone sent a telegram and the {} queue is over {} nations, time to recruit!",
                role.mention(),
                time_threshold,
                prettify_name(&self.region),
                fill_threshold
            )
        ))
    }
}