use serenity::all::{ComponentInteraction, ComponentInteractionDataKind, Context};

use crate::bot::{Data, Error, util::{self, Component}};
use crate::interactions::check_interaction_authorization;

pub async fn handle_edit_queue_role(
    ctx: &Context, data: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    if let Some(message) = check_interaction_authorization(&component.member) {
        util::direct_reply(ctx, Component(component), message, true).await?;
        return Ok(());
    }

    let role = {
        let ComponentInteractionDataKind::RoleSelect { values, .. } = &component.data.kind else {
            util::direct_reply(
                ctx, Component(component), "Error: invalid interaction", true
            ).await?;
        
            return Ok(());
        };

        values.get(0).cloned()
    };

    let mut queues = data.inner.queues.lock().await;

    let Some(queue) = queues.get_mut(&component.channel_id) else {
        util::direct_reply(
            ctx, Component(component), "There is no queue set up in this channel!", true
        ).await?;
        
        return Ok(());
    };

    queue.ping_role = role;
    queue.insert(&data.inner.pool).await;

    util::direct_reply(
        ctx, Component(component), 
        "Role edited, use another button to refresh the edit page.", true
    ).await?;

    Ok(())
}

pub async fn handle_edit_queue_channel(
    ctx: &Context, data: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    if let Some(message) = check_interaction_authorization(&component.member) {
        util::direct_reply(ctx, Component(component), message, true).await?;
        return Ok(());
    }
    
    let channel = {
        let ComponentInteractionDataKind::ChannelSelect { values, .. } = &component.data.kind else {
            util::direct_reply(
                ctx, Component(component), "Error: invalid interaction", true
            ).await?;
        
            return Ok(());
        };

        values.get(0).cloned()
    };

    let mut queues = data.inner.queues.lock().await;

    let Some(queue) = queues.get_mut(&component.channel_id) else {
        util::direct_reply(
            ctx, Component(component), "There is no queue set up in this channel!", true
        ).await?;
        
        return Ok(());
    };

    queue.ping_channel = channel;
    queue.insert(&data.inner.pool).await;

    util::direct_reply(
        ctx, Component(component), 
        "Role edited, use another button to refresh the edit page.", true
    ).await?;

    Ok(())
}