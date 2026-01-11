use serenity::all::{ComponentInteraction, Context};

use crate::bot::{Data, Error, util::{self, Component}};
use crate::interactions::check_interaction_authorization;

pub async fn handle_delete_queue_threshold(
    ctx: &Context, data: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    if let Some(message) = check_interaction_authorization(&component.member) {
        util::direct_reply(ctx, Component(component), message, true).await?;
        return Ok(());
    }
    
    let mut queues = data.inner.queues.lock().await;

    let Some(queue) = queues.get_mut(&component.channel_id) else {
        util::direct_reply(
            ctx, Component(component), "There is no queue set up in this channel!", true
        ).await?;
        
        return Ok(());
    };

    queue.thresholds = None;
    queue.insert(&data.inner.pool).await;

    util::direct_reply(
        ctx, Component(component), 
        "Threshold removed, use another button to refresh the edit page.", true
    ).await?;

    Ok(())
}

pub async fn handle_clear_queue_role_and_channel(
    ctx: &Context, data: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    if let Some(message) = check_interaction_authorization(&component.member) {
        util::direct_reply(ctx, Component(component), message, true).await?;
        return Ok(());
    }

    let mut queues = data.inner.queues.lock().await;

    let Some(queue) = queues.get_mut(&component.channel_id) else {
        util::direct_reply(
            ctx, Component(component), "There is no queue set up in this channel!", true
        ).await?;
        
        return Ok(());
    };

    queue.ping_role = None;
    queue.ping_channel = None;
    queue.insert(&data.inner.pool).await;

    util::direct_reply(
        ctx, Component(component), 
        "Role and channel cleared, use another button to refresh the edit page.", true
    ).await?;

    Ok(())
}