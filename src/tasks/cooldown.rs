use std::{collections::{HashSet, HashMap}, time::Duration};
use serenity::all::{CacheHttp, Context, ComponentInteraction, UserId, Timestamp};
use log::warn;

use crate::bot::Data;

pub async fn cooldown_task(ctx: Context, data: Data) {
    let mut ticker = tokio::time::interval(Duration::from_secs(1));

    let mut extant_cooldowns: HashSet<(UserId, String)> = HashSet::new();
    let mut expired_cooldowns: HashMap<(UserId, String), Option<ComponentInteraction>> = HashMap::new();

    loop {
        ticker.tick().await;

        extant_cooldowns.clear();
        expired_cooldowns.clear();

        {
            let mut cooldowns = data.inner.cooldowns.lock().await;
            let now = Timestamp::now().timestamp();

            for (key, value) in cooldowns.iter() {
                if value.0 <= now {
                    expired_cooldowns.insert(key.clone(), value.1.clone());
                } else {
                    extant_cooldowns.insert(key.clone());
                }
            }

            for key in expired_cooldowns.keys() {
                cooldowns.remove(key);
            }
        }

        for interaction in expired_cooldowns.values().flatten() {
            interaction.delete_response(ctx.http()).await.unwrap_or_else(|err| {
                warn!("Error deleting expired cooldown message: {err}");
            });
        }

        let sessions_to_update = {
            let sessions = data.inner.sessions.lock().await;

            sessions.values().flat_map(|session| {
                if extant_cooldowns.contains(
                    &(session.user, session.nation.clone())
                ) { None }
                else { Some(session.clone()) }
            }).collect::<Vec<_>>()
        };

        for session in sessions_to_update {
            session.try_send_new_telegram(&ctx, &data).await.unwrap_or_else(|err| {
                warn!("Error triggering session update: {err}");
            });

            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }
}