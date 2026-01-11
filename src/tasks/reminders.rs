use std::time::Duration;
use serenity::all::{CacheHttp, Context, CreateMessage};
use log::warn;

use crate::bot::Data;

pub async fn reminders_task(ctx: Context, data: Data) {
    let mut ticker = tokio::time::interval(Duration::from_secs(60));

    loop {
        ticker.tick().await;

        let reminders = {
            let mut queues = data.inner.queues.lock().await;

            queues.values_mut().flat_map(|queue| {
                queue.make_reminder_if_needed()
            }).collect::<Vec<_>>()
        };

        for (channel, text) in reminders {
            if let Err(err) = channel.send_message(
                ctx.http(), CreateMessage::new().content(&text)
            ).await {
                warn!("Error sending reminder for channel {channel}: {err} ({text})");
            }
        }
    }
}