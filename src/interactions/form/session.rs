use std::collections::hash_map::Entry;
use serenity::all::{
    CacheHttp, ComponentInteraction, Context, CreateActionRow, CreateInputText, CreateInteractionResponse, 
    CreateModal, InputTextStyle, ModalInteraction, ActionRowComponent, CreateMessage
};

use crate::api::calculate_telegram_delay;
use crate::bot::{Data, Error, util::{self, Modal}};
use crate::models::session::{RecruitDelay, Session, SESSION_TELEGRAM_BUFFER};
use crate::embeds::create_session_start_embed;

pub async fn spawn_session_form(
    ctx: &Context, _: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    component.create_response(ctx.http(), CreateInteractionResponse::Modal(
        CreateModal::new("stream-start-modal", "Start Recruitment Session").components(
            vec![CreateActionRow::InputText(
                CreateInputText::new(
                    InputTextStyle::Short, "Delay (time between telegrams)", "session-delay"
                ).placeholder("Leave empty for automatic delay...").required(false)
            )]
        )
    )).await?;

    Ok(())
}

const MAX_ACCEPTABLE_DELAY: i64 = 180; // 3 minutes is long enough

pub async fn process_session_form(
     ctx: &Context, data: &Data, modal: &ModalInteraction
) -> Result<(), crate::bot::Error> {
    let components = &modal.data.components;
    util::defer_ephemeral(ctx, Modal(modal)).await?;

    let mut delay = None;

    for row in components {
        for component in &row.components {
            if let ActionRowComponent::InputText(input) = component {
                match input.custom_id.as_str() {
                    "session-delay" => delay = input.value.clone(),
                    _ => {}
                }
            }
        }
    }

    let user_data = {
        match data.inner.user_data.lock().await.get(&(modal.channel_id, modal.user.id)) {
            Some(v) => v.clone(),
            None => {
                util::edit_reply(
                    ctx, util::Modal(modal), 
                    "No user data linked to this queue! Please click 'Setup Templates' first."
                ).await?;

                return Ok(());
            },
        }
    };

    let delay = match delay.and_then(|v| v.parse::<u64>().ok()) {
        Some(delay) => {
            let min_acceptable_delay = calculate_telegram_delay(user_data.founded) * 8 + SESSION_TELEGRAM_BUFFER;

            if (delay as i64) < min_acceptable_delay {
                util::edit_reply(
                    ctx, util::Modal(modal), 
                    &format!("Due to your nation's age, telegram delay cannot be lower than {} seconds.", min_acceptable_delay)
                ).await?;

                return Ok(());
            }

            if (delay as i64) > MAX_ACCEPTABLE_DELAY {
                util::edit_reply(
                    ctx, util::Modal(modal), 
                    "Telegram delay cannot be more than 3 minutes (180 seconds)!"
                ).await?;

                return Ok(());
            }

            RecruitDelay::Fixed(delay)
        },
        None => RecruitDelay::Automatic
    };

    match data.inner.sessions.lock().await.entry(modal.user.id) {
        Entry::Occupied(_) => {
            util::edit_reply(
                ctx, util::Modal(modal), 
                "You already have a session in progress! Please stop the current session before starting a new one."
            ).await?;

            return Ok(());
        },
        Entry::Vacant(v) => {
            v.insert(Session { 
                user: modal.user.id, queue: modal.channel_id, nation: user_data.nation.clone(), delay: delay.clone() 
            });
        },
    }

    let (embed, components) = create_session_start_embed(
        &user_data.nation,
        &delay
    );

    modal.user.direct_message(
        ctx.http(), CreateMessage::new().embed(embed).components(components)
    ).await?;

    util::edit_reply(
        ctx, Modal(modal), &format!("Session started! (delay: {})\nCheck your DMs!", delay)
    ).await?;

    Ok(())
}