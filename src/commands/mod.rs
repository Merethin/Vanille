mod create_queue;
mod edit_queue;
mod delete_queue;

use poise::{CreateReply, Command};
use crate::bot::{Context, Error, Data};

use create_queue::create_queue;
use edit_queue::edit_queue;
use delete_queue::delete_queue;

pub fn create_command_list() -> Vec<Command<Data, Error>> {
    vec![
        create_queue(),
        edit_queue(),
        delete_queue()
    ]
}

pub async fn check_command_authorization(ctx: &Context<'_>) -> Result<bool, Error> {
    match ctx.author_member().await {
        None => {
            ctx.send(
                CreateReply::default().content("This command cannot be run in DMs!").ephemeral(true)
            ).await?;

            return Ok(false);
        },
        Some(member) => {
            if !member.permissions.map_or(false, |p| p.manage_guild()) {
                ctx.send(
                    CreateReply::default().content("You are not allowed to run this command!").ephemeral(true)
                ).await?;

                return Ok(false);
            }

            return Ok(true);
        }
    }
}