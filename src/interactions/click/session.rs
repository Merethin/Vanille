use serenity::all::{ComponentInteraction, Context};

use crate::bot::{Data, Error, util::{self, Component}};

pub async fn handle_stream_end(
    ctx: &Context, data: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    util::defer_ephemeral(ctx, Component(component)).await?;
    let success = data.inner.sessions.lock().await.remove(&component.user.id).is_some();
    
    if success {
        util::edit_reply(
            ctx, Component(component), "Your current session has been stopped!"
        ).await?;
    } else {
        util::edit_reply(
            ctx, Component(component), "Error: You do not currently have a session in progress."
        ).await?;
    }

    Ok(())
}
