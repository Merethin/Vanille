use poise::CreateReply;

use crate::bot::{Context, Error};
use crate::embeds::create_edit_queue_embed;
use crate::commands::check_command_authorization;

#[poise::command(slash_command)]
pub async fn edit_queue(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !check_command_authorization(&ctx).await? {
        return Ok(());
    }

    let queues = ctx.data().inner.queues.lock().await;

    let Some(queue) = queues.get(&ctx.channel_id()) else {
        ctx.send(
            CreateReply::default().content("There is no queue set up in this channel!").ephemeral(true)
        ).await?;

        return Ok(());
    };

    let (embed, components) = create_edit_queue_embed(
        &queue.region, queue.size, &queue.filter.regions, &queue.thresholds, &queue.ping_channel, &queue.ping_role
    );

    drop(queues);

    ctx.send(
        CreateReply::default().embed(embed).components(components).ephemeral(true)
    ).await?;

    Ok(())
}