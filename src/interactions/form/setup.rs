use serenity::all::{
    ActionRowComponent, CacheHttp, ComponentInteraction, Context, CreateActionRow, CreateInputText, CreateInteractionResponse, CreateModal, InputTextStyle, ModalInteraction
};

use caramel::ns::format::{canonicalize_name, prettify_name};

use crate::api::query_nation_data;
use crate::bot::{Data, Error, util::{self, Modal}};
use crate::models::user_data::UserData;

pub async fn spawn_setup_form(
    ctx: &Context, _: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    component.create_response(ctx.http(), CreateInteractionResponse::Modal(
        CreateModal::new("setup-modal", "Setup Telegram Templates").components(
            vec![CreateActionRow::InputText(
                CreateInputText::new(
                    InputTextStyle::Short, "Nation", "session-nation-input"
                ).placeholder("Enter your nation here...")
            ),
            CreateActionRow::InputText(
                CreateInputText::new(
                    InputTextStyle::Paragraph, "Newfound Templates", "session-newfound-input"
                ).placeholder("Enter one template per line")
            ),
            CreateActionRow::InputText(
                CreateInputText::new(
                    InputTextStyle::Paragraph, "Refound Templates", "session-refound-input"
                ).placeholder("Enter one template per line")
            )]
        )
    )).await?;

    Ok(())
}

pub async fn process_setup_form(
    ctx: &Context, data: &Data, modal: &ModalInteraction
) -> Result<(), Error> {
    let components = &modal.data.components;

    util::defer_ephemeral(ctx, Modal(modal)).await?;

    let mut nation = None;
    let mut newfound_templates = None;
    let mut refound_templates = None;

    for row in components {
        for component in &row.components {
            if let ActionRowComponent::InputText(input) = component {
                match input.custom_id.as_str() {
                    "session-nation-input" => nation = input.value.clone(),
                    "session-newfound-input" => newfound_templates = input.value.clone(),
                    "session-refound-input" => refound_templates = input.value.clone(),
                    _ => {}
                }
            }
        }
    }

    let nation = match nation {
        Some(v) => canonicalize_name(&v),
        None => {
            util::edit_reply(ctx, Modal(modal), "Error: Please enter a nation name!").await?;

            return Ok(());
        }
    };

    let Some(region) = data.inner.queues.lock().await.get(&modal.channel_id).and_then(
        |v| Some(v.region.clone())
    ) else {
        util::edit_reply(
            ctx, util::Modal(modal), 
            "Invalid interaction: no queue linked to channel"
        ).await?;

        return Ok(());
    };

    let founded = match query_nation_data(&data.inner.api_client, &nation).await {
        Ok(data) => {
            if canonicalize_name(&data.region) != region {
                util::edit_reply(
                    ctx, util::Modal(modal), 
                    &format!("Error: {} does not reside in {}!", nation, prettify_name(&region))
                ).await?;

                return Ok(());
            }

            data.foundedtime
        },
        Err(err) => {
            util::edit_reply(
                ctx, util::Modal(modal), 
                &format!("API error querying data for {}: {}", nation, err)
            ).await?;

            return Ok(());
        }
    };

    let newfound_templates = newfound_templates.and_then(|s| Some(s.lines().map(
        |s| s.trim().to_string()
    ).filter(
        |s| !s.is_empty()
    ).collect::<Vec<_>>())).unwrap_or(vec![]);

    let refound_templates = refound_templates.and_then(|s| Some(s.lines().map(
        |s| s.trim().to_string()
    ).filter(
        |s| !s.is_empty()
    ).collect::<Vec<_>>())).unwrap_or(vec![]);

    let user_data = UserData::new(
        modal.channel_id,
        modal.user.id,
        nation.clone(),
        founded,
        newfound_templates,
        refound_templates
    );

    user_data.insert(&data.inner.pool).await;
    data.inner.user_data.lock().await.insert((modal.channel_id, modal.user.id), user_data);

    util::edit_reply(
        ctx, util::Modal(modal), 
        &format!("Templates successfully registered for {}!", prettify_name(&nation))
    ).await?;

    Ok(())
}
