use std::fmt;
use log::warn;
use rand::seq::IndexedRandom;
use serenity::all::{CacheHttp, CreateMessage, Context, UserId, ChannelId, Timestamp};

use crate::api::calculate_telegram_delay;
use crate::{embeds::create_telegram_embed, models::report::ReportEntry};
use crate::bot::{Data, Error};

#[derive(Debug, Clone)]
pub enum RecruitDelay {
    Fixed(u64),
    Automatic,
}

#[derive(Clone)]
pub struct Session {
    pub user: UserId,
    pub queue: ChannelId,
    pub nation: String,
    pub delay: RecruitDelay,
}

pub const SESSION_TELEGRAM_BUFFER: i64 = 10; // 10 seconds past normal telegram cooldown

impl fmt::Display for RecruitDelay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixed(delay) => write!(f, "{} seconds", delay),
            Self::Automatic => write!(f, "automatic")
        }
    }
}

impl Session {
    pub async fn try_send_new_telegram(
        &self, ctx: &Context, data: &Data
    ) -> Result<(), Error> {
        let user_data = {
            match data.inner.user_data.lock().await.get(&(self.queue, self.user)) {
                Some(v) => v.clone(),
                None => {
                    warn!("Error in session: No user data linked to queue");

                    return Ok(());
                },
            }
        };

        let ((nations, templates, update), channel) = {
            let mut queues = data.inner.queues.lock().await;
            let queue = match queues.get_mut(&self.queue) {
                Some(v) => v,
                None => {
                    drop(queues);

                    warn!("Error in session: No queue linked to channel");

                    return Ok(());
                },
            };

            (queue.pull(&user_data, 8), queue.channel)
        };

        if nations.is_empty() || templates.is_empty() {
            // No nations to telegram
            return Ok(());
        }

        let template = {
            let mut rng = rand::rng();
            templates.choose(&mut rng).expect(
                "Template list should not be empty"
            )
        };

        let cooldown = match self.delay {
            RecruitDelay::Fixed(time) => {
                Timestamp::now().timestamp() + time as i64
            },
            RecruitDelay::Automatic => {
                Timestamp::now().timestamp() 
                    + calculate_telegram_delay(user_data.founded) * nations.len() as i64
                    + SESSION_TELEGRAM_BUFFER
            },
        };

        let (embed, components) = create_telegram_embed(
            &nations, template, &user_data.nation, cooldown, &data.inner.user_agent, true
        );

        data.inner.cooldowns.lock().await.insert(
            (UserId::new(user_data.user_id), user_data.nation.clone()), 
            (cooldown, None)
        );

        self.user.direct_message(
            ctx.http(), CreateMessage::new().embed(embed).components(components)
        ).await?;

        if let Some(update) = update {
            update.execute(ctx.clone()).await;
        }

        for nation in &nations {
            ReportEntry::new(
                nation.name.clone(), 
                nation.event.clone(), 
                nation.region.clone(),
                channel,
                nation.queue_time, 
                self.user,
                user_data.nation.clone(),
                template.clone(),
                Timestamp::now()
            ).insert(&data.inner.pool).await;
        }

        Ok(())
    }
}