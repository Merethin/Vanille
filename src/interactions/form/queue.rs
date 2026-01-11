use log::warn;
use uuid::Uuid;
use serenity::all::{
    CacheHttp, ComponentInteraction, Context, CreateActionRow, CreateInputText, CreateInteractionResponse, 
    CreateModal, InputTextStyle, ModalInteraction, ActionRowComponent, EditInteractionResponse
};

use caramel::ns::format::canonicalize_name;

use crate::{bot::{Data, Error, util::{self, Modal}}, embeds::create_edit_queue_embed};
use crate::interactions::check_interaction_authorization;

pub async fn spawn_queue_size_form(
    ctx: &Context, data: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    let key = Uuid::new_v4().to_string();
    data.inner.interaction_tokens.lock().await.insert(key.clone(), component.token.clone());

    component.create_response(ctx.http(), CreateInteractionResponse::Modal(
        CreateModal::new(format!("queue-size-modal:{}", key), "Edit Queue Size").components(
            vec![CreateActionRow::InputText(
                CreateInputText::new(
                    InputTextStyle::Short, "Queue Size", "queue-size"
                ).placeholder("Size must be between 50 and 500")
            )]
        )
    )).await?;

    Ok(())
}

pub async fn spawn_queue_regions_form(
    ctx: &Context, data: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    let key = Uuid::new_v4().to_string();
    data.inner.interaction_tokens.lock().await.insert(key.clone(), component.token.clone());

    component.create_response(ctx.http(), CreateInteractionResponse::Modal(
        CreateModal::new(format!("queue-regions-modal:{}", key), "Edit Excluded Regions").components(
            vec![CreateActionRow::InputText(
                CreateInputText::new(
                    InputTextStyle::Paragraph, "Regions to Exclude", "regions"
                ).placeholder("One region per line, leave empty to clear the list").required(false)
            )]
        )
    )).await?;

    Ok(())
}

pub async fn spawn_queue_threshold_form(
    ctx: &Context, data: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    let key = Uuid::new_v4().to_string();
    data.inner.interaction_tokens.lock().await.insert(key.clone(), component.token.clone());

    component.create_response(ctx.http(), CreateInteractionResponse::Modal(
        CreateModal::new(format!("queue-threshold-modal:{}", key), "Edit/Set Reminder Threshold").components(
            vec![CreateActionRow::InputText(
                CreateInputText::new(
                    InputTextStyle::Short, "Fill Threshold", "fill-threshold"
                ).placeholder("Minimum amount of nations to trigger a reminder at")
            ), CreateActionRow::InputText(
                CreateInputText::new(
                    InputTextStyle::Short, "Time Threshold (in minutes)", "time-threshold"
                ).placeholder("Minimum time since a telegram was sent to trigger at")
            )]
        )
    )).await?;

    Ok(())
}

pub async fn process_queue_size_form(
     ctx: &Context, data: &Data, modal: &ModalInteraction, key: &str
) -> Result<(), Error> {
    if let Some(message) = check_interaction_authorization(&modal.member) {
        util::direct_reply(ctx, Modal(modal), message, true).await?;
        return Ok(());
    }

    let components = &modal.data.components;
    util::defer_ephemeral(ctx, Modal(modal)).await?;

    let Some(token) = data.inner.interaction_tokens.lock().await.remove(key) else {
        util::edit_reply(
            ctx, Modal(modal), "Error: invalid interaction"
        ).await?;

        return Ok(());
    };

    let mut size = None;

    for row in components {
        for component in &row.components {
            if let ActionRowComponent::InputText(input) = component {
                match input.custom_id.as_str() {
                    "queue-size" => size = input.value.clone(),
                    _ => {}
                }
            }
        }
    }

    let size = size.and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);

    if size < 50 || size > 500 {
        util::edit_reply(
            ctx, Modal(modal), "Error: size is not a number or doesn't fit in range"
        ).await?;
        
        return Ok(());
    }

    let mut queues = data.inner.queues.lock().await;

    let Some(queue) = queues.get_mut(&modal.channel_id) else {
        util::edit_reply(
            ctx, Modal(modal), "There is no queue set up in this channel!"
        ).await?;
        
        return Ok(());
    };

    queue.size = size as usize;
    queue.insert(&data.inner.pool).await;

    let (embed, components) = create_edit_queue_embed(
        &queue.region, queue.size, &queue.filter.regions, &queue.thresholds, &queue.ping_channel, &queue.ping_role
    );

    if let Err(err) = ctx.http().edit_original_interaction_response(
        &token, 
        &EditInteractionResponse::new().embed(embed).components(components), 
        vec![]
    ).await {
        warn!("Error while editing interaction message: {err}");
    }

    modal.delete_response(ctx.http()).await?;

    Ok(())
}

pub async fn process_queue_regions_form(
     ctx: &Context, data: &Data, modal: &ModalInteraction, key: &str
) -> Result<(), Error> {
    if let Some(message) = check_interaction_authorization(&modal.member) {
        util::direct_reply(ctx, Modal(modal), message, true).await?;
        return Ok(());
    }

    let components = &modal.data.components;
    util::defer_ephemeral(ctx, Modal(modal)).await?;

    let Some(token) = data.inner.interaction_tokens.lock().await.remove(key) else {
        util::edit_reply(
            ctx, Modal(modal), "Error: invalid interaction"
        ).await?;

        return Ok(());
    };

    let mut regions = None;

    for row in components {
        for component in &row.components {
            if let ActionRowComponent::InputText(input) = component {
                match input.custom_id.as_str() {
                    "regions" => regions = input.value.clone(),
                    _ => {}
                }
            }
        }
    }

    let mut queues = data.inner.queues.lock().await;

    let Some(queue) = queues.get_mut(&modal.channel_id) else {
        util::edit_reply(
            ctx, Modal(modal), "There is no queue set up in this channel!"
        ).await?;
        
        return Ok(());
    };

    queue.filter.regions = regions.and_then(|v| 
        Some(v.split("\n").map(|s| canonicalize_name(&s.trim())).filter(|s| !s.is_empty()).collect())
    ).unwrap_or(vec![]);

    queue.insert(&data.inner.pool).await;

    let (embed, components) = create_edit_queue_embed(
        &queue.region, queue.size, &queue.filter.regions, &queue.thresholds, &queue.ping_channel, &queue.ping_role
    );

    if let Err(err) = ctx.http().edit_original_interaction_response(
        &token, 
        &EditInteractionResponse::new().embed(embed).components(components), 
        vec![]
    ).await {
        warn!("Error while editing interaction message: {err}");
    }

    modal.delete_response(ctx.http()).await?;

    Ok(())
}

pub async fn process_queue_threshold_form(
     ctx: &Context, data: &Data, modal: &ModalInteraction, key: &str
) -> Result<(), Error> {
    if let Some(message) = check_interaction_authorization(&modal.member) {
        util::direct_reply(ctx, Modal(modal), message, true).await?;
        return Ok(());
    }
    
    let components = &modal.data.components;
    util::defer_ephemeral(ctx, Modal(modal)).await?;

    let Some(token) = data.inner.interaction_tokens.lock().await.remove(key) else {
        util::edit_reply(
            ctx, Modal(modal), "Error: invalid interaction"
        ).await?;

        return Ok(());
    };

    let mut fill_threshold = None;
    let mut time_threshold = None;

    for row in components {
        for component in &row.components {
            if let ActionRowComponent::InputText(input) = component {
                match input.custom_id.as_str() {
                    "fill-threshold" => fill_threshold = input.value.clone(),
                    "time-threshold" => time_threshold = input.value.clone(),
                    _ => {}
                }
            }
        }
    }

    let mut queues = data.inner.queues.lock().await;

    let Some(queue) = queues.get_mut(&modal.channel_id) else {
        util::edit_reply(
            ctx, Modal(modal), "There is no queue set up in this channel!"
        ).await?;
        
        return Ok(());
    };

    let fill_threshold = fill_threshold.and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    let time_threshold = time_threshold.and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);

    if fill_threshold < 30 || fill_threshold > 500 {
        util::edit_reply(
            ctx, Modal(modal), "Error: fill threshold is not a number or doesn't fit in range 30-500"
        ).await?;
        
        return Ok(());
    }

    if time_threshold < 30 || time_threshold > 360 {
        util::edit_reply(
            ctx, Modal(modal), "Error: time threshold is not a number or doesn't fit in range 30-360"
        ).await?;
        
        return Ok(());
    }

    queue.thresholds = Some((fill_threshold, time_threshold));

    queue.insert(&data.inner.pool).await;

    let (embed, components) = create_edit_queue_embed(
        &queue.region, queue.size, &queue.filter.regions, &queue.thresholds, &queue.ping_channel, &queue.ping_role
    );

    if let Err(err) = ctx.http().edit_original_interaction_response(
        &token, 
        &EditInteractionResponse::new().embed(embed).components(components), 
        vec![]
    ).await {
        warn!("Error while editing interaction message: {err}");
    }

    modal.delete_response(ctx.http()).await?;

    Ok(())
}