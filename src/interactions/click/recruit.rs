use rand::seq::IndexedRandom;
use serenity::all::{
    CacheHttp, ComponentInteraction, Context, EditInteractionResponse, UserId, Timestamp
};

use crate::api::calculate_telegram_delay;
use crate::bot::{Data, Error, util::{self, Component}};
use crate::embeds::create_telegram_embed;
use crate::models::queue::QUEUE_TELEGRAM_BUFFER;
use crate::models::report::ReportEntry;

pub async fn handle_recruit_oneshot(
    ctx: &Context, data: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    component.defer_ephemeral(ctx.http()).await?;

    let user_data = {
        match data.inner.user_data.lock().await.get(&(component.channel_id, component.user.id)) {
            Some(v) => v.clone(),
            None => {
                util::edit_reply(
                    ctx, Component(component), 
                    "No user data linked to this queue! Please click 'Setup Templates' first."
                ).await?;

                return Ok(());
            },
        }
    };

    if data.inner.cooldowns.lock().await.contains_key(
        &UserId::new(user_data.user_id)
    ) {
        util::edit_reply(ctx, Component(component), "Error: cooldown still in progress.").await?;

        return Ok(());
    }

    let ((nations, templates, update), channel) = {
        let mut queues = data.inner.queues.lock().await;
        let queue = match queues.get_mut(&component.channel_id) {
            Some(v) => v,
            None => {
                drop(queues);

                util::edit_reply(
                    ctx, Component(component), 
                    "Invalid interaction: no queue linked to channel"
                ).await?;

                return Ok(());
            },
        };

        (queue.pull(&user_data, 8), queue.channel)
    };

    if nations.is_empty() || templates.is_empty() {
        util::edit_reply(
            ctx, Component(component), 
            "No eligible nations to telegram! Please try again later."
        ).await?;

        return Ok(());
    }
    
    let template = {
        let mut rng = rand::rng();
        templates.choose(&mut rng).expect(
            "Template list should not be empty"
        )
    };

    let cooldown = Timestamp::now().timestamp() 
        + calculate_telegram_delay(user_data.founded) * nations.len() as i64 
        + QUEUE_TELEGRAM_BUFFER;

    let (embed, components) = create_telegram_embed(
        &nations, template, &user_data.nation, cooldown, &data.inner.user_agent, false
    );

    data.inner.cooldowns.lock().await.insert(
        UserId::new(user_data.user_id), 
        (cooldown, Some(component.clone()))
    );

    component.edit_response(
        ctx.http(), EditInteractionResponse::new().embed(embed).components(components)
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
            UserId::new(user_data.user_id), 
            user_data.nation.clone(),
            template.clone(),
            Timestamp::now()
        ).insert(&data.inner.pool).await;
    }

    Ok(())
}