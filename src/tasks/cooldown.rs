use std::{collections::{HashSet, HashMap}, time::Duration};
use serenity::all::{CacheHttp, Context, ComponentInteraction, UserId, Timestamp};
use rand::seq::SliceRandom;
use log::warn;

use crate::bot::Data;

#[derive(PartialEq)]
enum SessionAction {
    SendTelegram,
    EnactPause,
    Close,
}

const SESSION_PAUSE_DELAY: i64 = 20 * 60; // Session is paused 10 min after the last activity check is passed
const INACTIVITY_CLOSE_DELAY: i64 = 5 * 60; // Session is closed 5 min after the last activity check is presented and not responded to

pub async fn cooldown_task(ctx: Context, data: Data) {
    let mut ticker = tokio::time::interval(Duration::from_secs(1));

    let mut extant_cooldowns: HashSet<UserId> = HashSet::new();
    let mut expired_cooldowns: HashMap<UserId, Option<ComponentInteraction>> = HashMap::new();

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
            let mut sessions = data.inner.sessions.lock().await;

            let mut result = sessions.values_mut().flat_map(|session| {
                if let Some(pause_time) = session.pause_time {
                    if (Timestamp::now().timestamp() - pause_time.timestamp()) > INACTIVITY_CLOSE_DELAY {
                        // Session activity check expired
                        Some((session.clone(), SessionAction::Close))
                    } else {
                        // Session is paused
                        None
                    }
                } else {
                    if extant_cooldowns.contains(&session.user) { 
                        // Cooldown still in progress
                        None
                    } else {
                        if (Timestamp::now().timestamp() - session.last_activity_check.timestamp()) > SESSION_PAUSE_DELAY {
                            // Time for a session activity check
                            session.pause_time = Some(Timestamp::now());
                            Some((session.clone(), SessionAction::EnactPause))
                        } else {
                            // Send batch
                            Some((session.clone(), SessionAction::SendTelegram))
                        }
                    }
                }
            }).collect::<Vec<_>>();

            for (session, action) in &result {
                if *action == SessionAction::Close {
                    sessions.remove(&session.user);
                }
            }

            let mut rng = rand::rng();
            result.shuffle(&mut rng);

            result
        };

        for (session, action) in sessions_to_update {
            match action {
                SessionAction::SendTelegram => session.try_send_new_telegram(&ctx, &data).await.unwrap_or_else(|err| {
                    warn!("Error triggering session update: {err}");
                }),
                SessionAction::EnactPause => session.inactivity_pause(&ctx).await.unwrap_or_else(|err| {
                    warn!("Error triggering session pause: {err}");
                }),
                SessionAction::Close => session.inactivity_close(&ctx).await.unwrap_or_else(|err| {
                    warn!("Error triggering session close: {err}");
                }),
            }
        }
    }
}