use serenity::all::{
    CacheHttp, ComponentInteraction, Context, CreateInteractionResponse, 
    CreateInteractionResponseMessage, EditInteractionResponse, ModalInteraction
};

pub enum Interaction<'a> {
    Component(&'a ComponentInteraction),
    Modal(&'a ModalInteraction)
}

pub use Interaction::{Component, Modal};

pub async fn defer_ephemeral(ctx: &Context, interaction: Interaction<'_>) -> Result<(), crate::bot::Error> {
    match interaction {
        Component(c) => c.defer_ephemeral(ctx.http()).await,
        Modal(c) => c.defer_ephemeral(ctx.http()).await,
    }?;

    Ok(())
}

pub async fn direct_reply(
    ctx: &Context, interaction: Interaction<'_>, content: &str, ephemeral: bool
) -> Result<(), crate::bot::Error> {
    let message = CreateInteractionResponseMessage::new().content(content).ephemeral(ephemeral);

    let response = CreateInteractionResponse::Message(message);
    match interaction {
        Component(c) => c.create_response(ctx.http(), response).await,
        Modal(c) => c.create_response(ctx.http(), response).await,
    }?;

    Ok(())
}

pub async fn edit_reply(
    ctx: &Context, interaction: Interaction<'_>, content: &str
) -> Result<(), crate::bot::Error> {
    let message = EditInteractionResponse::new().content(content);

    match interaction {
        Component(c) => c.edit_response(ctx.http(), message).await,
        Modal(c) => c.edit_response(ctx.http(), message).await,
    }?;

    Ok(())
}