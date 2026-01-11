use poise::CreateReply;
use serenity::all::{CreateEmbed, CreateMessage, EditMessage};
use caramel::ns::format::canonicalize_name;

use crate::bot::{Context, Error};
use crate::models::queue::{Queue, Filter};
use crate::embeds::create_queue_embed;
use crate::commands::check_command_authorization;

#[poise::command(slash_command)]
pub async fn create_queue(
    ctx: Context<'_>,
    region: String,
    size: usize,
) -> Result<(), Error> {
    if !check_command_authorization(&ctx).await? {
        return Ok(());
    }

    let mut queues = ctx.data().inner.queues.lock().await;

    if queues.contains_key(&ctx.channel_id()) {
        ctx.send(
            CreateReply::default().content("There is already a queue set up in this channel!").ephemeral(true)
        ).await?;

        return Ok(());
    }

    let mut message = ctx.channel_id().send_message(ctx.http(), CreateMessage::new().add_embed(
        CreateEmbed::new().description("Setting up queue...")
    )).await?;

    let queue = Queue::new(
        ctx.channel_id(),
        message.id, 
        canonicalize_name(&region), 
        Filter::default(),
        size
    );

    let (embed, components) = create_queue_embed(&queue);

    message.edit(ctx.http(), EditMessage::new().embed(embed).components(components)).await?;

    queue.insert(&ctx.data().inner.pool).await;
    queues.insert(ctx.channel_id(), queue);

    ctx.send(
        CreateReply::default().content("Queue created successfully.").ephemeral(true)
    ).await?;

    Ok(())
}