use poise::CreateReply;
use log::warn;

use crate::bot::{Context, Error};
use crate::models::user_data::UserData;
use crate::commands::check_command_authorization;

#[poise::command(slash_command)]
pub async fn delete_queue(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !check_command_authorization(&ctx).await? {
        return Ok(());
    }

    let mut queues = ctx.data().inner.queues.lock().await;

    let Some(queue) = queues.remove(&ctx.channel_id()) else {
        ctx.send(
            CreateReply::default().content("There is no queue set up in this channel!").ephemeral(true)
        ).await?;

        return Ok(());
    };

    drop(queues);

    let mut sessions = ctx.data().inner.sessions.lock().await;

    let sessions_to_remove = sessions.iter().flat_map(|(key, session)| {
        if session.queue == queue.channel {
            Some(key.clone())
        } else { None }
    }).collect::<Vec<_>>();

    for session in sessions_to_remove {
        sessions.remove(&session);
    }

    drop(sessions);

    let mut user_data = ctx.data().inner.user_data.lock().await;

    let data_to_remove = user_data.keys().flat_map(|key| {
        if key.0 == queue.channel {
            Some(key.clone())
        } else { None }
    }).collect::<Vec<_>>();

    for data in data_to_remove {
        user_data.remove(&data);
    }

    drop(user_data);

    queue.remove(&ctx.data().inner.pool).await;
    UserData::remove_matching(queue.channel.get() as i64, &ctx.data().inner.pool).await;

    ctx.http().delete_message(queue.channel, queue.message, None).await.unwrap_or_else(|err| {
        warn!("Failed to delete queue message: {}", err);
    });

    ctx.send(
        CreateReply::default().content("Queue deleted successfully.").ephemeral(true)
    ).await?;

    Ok(())
}