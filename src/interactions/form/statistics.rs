use std::time::Duration;

use itertools::Itertools;
use serenity::{all::{
    ActionRowComponent, CacheHttp, ComponentInteraction, Context, CreateActionRow, CreateAttachment, CreateInputText, CreateInteractionResponse, CreateModal, EditInteractionResponse, InputTextStyle, ModalInteraction
}, builder::CreateInteractionResponseMessage, collector::ComponentInteractionCollector, model::application::ComponentInteractionDataKind};

use crate::{bot::{Data, Error, util::{self, Modal}}, embeds::{create_ranged_statistics_embed, create_template_embed}};
use vanille_models::report::ReportEntry;
use vanille_graph::*;

pub async fn spawn_stat_time_form(
    ctx: &Context, _: &Data, component: &ComponentInteraction, custom_id: &str
) -> Result<(), Error> {
    component.create_response(ctx.http(), CreateInteractionResponse::Modal(
        CreateModal::new(custom_id, "Report Timeframe").components(
            vec![CreateActionRow::InputText(
                CreateInputText::new(
                    InputTextStyle::Short, "Start of Report", "report-start"
                ).placeholder("Time is in UTC, defaults to the current time")
            ),CreateActionRow::InputText(
                CreateInputText::new(
                    InputTextStyle::Short, "End of Report", "report-end"
                ).placeholder("Time is in UTC, defaults to the current time")
            )]
        )
    )).await?;

    Ok(())
}

pub async fn spawn_template_id_form(
    ctx: &Context, _: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    component.create_response(ctx.http(), CreateInteractionResponse::Modal(
        CreateModal::new("stat-template-check-report", "Select Template").components(
            vec![CreateActionRow::InputText(
                CreateInputText::new(
                    InputTextStyle::Short, "Template ID", "template-id"
                ).placeholder("Enter template ID to analyze")
            )]
        )
    )).await?;

    Ok(())
}

async fn handle_ranged_leaders(
    ctx: &Context, data: &Data, modal: &ModalInteraction, range: (u64, u64)
) -> Result<(), Error> {
    let leaders = ReportEntry::count_by_nation(
        &data.inner.pool, &[modal.channel_id.get() as i64], Some(range)
    ).await?;

    if leaders.is_empty() {
        util::edit_reply(
            ctx, Modal(modal), "Error: no results recorded for this time period!"
        ).await?;
    } else {
        util::edit_reply(
            ctx, Modal(modal), 
            &format!(
                "Leaderboard from <t:{}:f> to <t:{}:f>:\n```\n{}\n```", 
                range.0, range.1, leaders.iter().map(|(nation, count)| format!("{nation}: {count}")).join("\n")
            )
        ).await?;
    }

    Ok(())
}

async fn handle_ranged_csv(
    ctx: &Context, data: &Data, modal: &ModalInteraction, range: (u64, u64)
) -> Result<(), Error> {
    let entries = ReportEntry::query(
        &data.inner.pool, modal.channel_id, Some(range)
    ).await?;

    if entries.is_empty() {
        util::edit_reply(
            ctx, Modal(modal), "Error: no results recorded for this time period!"
        ).await?;
    } else {
        let mut output: Vec<u8> = Vec::new();
        let mut writer = csv::WriterBuilder::new().has_headers(false).from_writer(&mut output);

        writer.write_record(&[
            "Nation Name", "Event Type", "Event Source", "Queued at Time", 
            "Recruiter Discord ID", "Sender Nation", "Telegram Template",
            "Sent at Time", "Moved to Region?", "Moved at Time"
        ])?;

        for entry in entries {
            writer.serialize(entry)?;
        }

        drop(writer);

        modal.edit_response(
ctx.http(), EditInteractionResponse::new().content(format!(
                "Telegram data from <t:{}:f> to <t:{}:f>:", range.0, range.1
            )).new_attachment(
                CreateAttachment::bytes(output, "vanille-report.csv")
            )
        ).await?;
    }

    Ok(())
}

async fn handle_ranged_graph(
    ctx: &Context, data: &Data, 
    modal: &ModalInteraction, range: (u64, u64),
    region: &str, kind: &str
) -> Result<(), Error> {
    let entries = ReportEntry::query(
        &data.inner.pool, modal.channel_id, Some(range)
    ).await?;

    if entries.is_empty() {
        util::edit_reply(
            ctx, Modal(modal), "Error: no results recorded for this time period!"
        ).await?;
    } else {
        let output: Vec<u8> = match kind {
            "delivery_time" => generate_delivery_time_plot(region, &entries).await?,
            "queue_activity" => generate_queue_activity_plot(region, &entries).await?,
            "sender_activity" => generate_sender_activity_plot(region, &entries).await?,
            "sender_category" => generate_sender_category_plot(region, &entries).await?,
            "sender_delivery_time" => generate_sender_delivery_time_plot(region, &entries).await?,
            "successful_recruits" => generate_successful_recruits_plot(region, &entries).await?,
            _ => {
                util::edit_reply(
                    ctx, Modal(modal), "Error: no graph type selected!"
                ).await?;
                return Ok(());
            }
        };

        modal.edit_response(
ctx.http(), EditInteractionResponse::new().content(format!(
                "Graph from <t:{}:f> to <t:{}:f>:", range.0, range.1
            )).new_attachment(
                CreateAttachment::bytes(output, "vanille-graph.png")
            )
        ).await?;
    }

    Ok(())
}

pub async fn process_stat_ranged_form(
    ctx: &Context, data: &Data, modal: &ModalInteraction
) -> Result<(), Error> {
    let region = data.inner.queues.lock().await.get(&modal.channel_id).map(|q| q.region.clone());

    if region.is_none() {
        util::direct_reply(
            ctx, Modal(modal),
            "Invalid interaction: no queue linked to channel", true
        ).await?;

        return Ok(());
    }

    modal.defer_ephemeral(ctx.http()).await?;

    let Some(range) = extract_time_range_from_modal(ctx, modal).await? else {
        return Ok(());
    };

    let (embed, components) = create_ranged_statistics_embed(range);
    modal.edit_response(ctx.http(), EditInteractionResponse::new().embed(embed).components(components)).await?;

    let message = modal.get_response(ctx.http()).await?;

    let mut kind: Option<String> = None;

    while let Some(interaction) = ComponentInteractionCollector::new(ctx).message_id(message.id).timeout(Duration::from_secs(120)).await {
        match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => {
                kind = values.get(0).cloned();
                interaction.create_response(ctx.http(), CreateInteractionResponse::Acknowledge).await?;
                continue;
            },
            ComponentInteractionDataKind::Button => {
                match interaction.data.custom_id.as_str() {
                    "stat-leaders-custom" => {
                        interaction.create_response(ctx.http(), CreateInteractionResponse::Acknowledge).await?;
                        return handle_ranged_leaders(ctx, data, modal, range).await;
                    },
                    "stat-csv-custom" => {
                        interaction.create_response(ctx.http(), CreateInteractionResponse::Acknowledge).await?;
                        return handle_ranged_csv(ctx, data, modal, range).await;
                    },
                    "stat-graph-custom" => {
                        match kind {
                            Some(kind) => {
                                interaction.create_response(ctx.http(), CreateInteractionResponse::Acknowledge).await?;
                                return handle_ranged_graph(ctx, data, modal, range, &region.unwrap(), &kind).await;
                            },
                            None => {
                                interaction.create_response(ctx.http(), CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new().content("Error: no graph kind selected!").ephemeral(true)
                                )).await?;
                            }
                        }
                    }
                    _ => {},
                }
            },
            _ => {},
        }
    }

    modal.edit_response(ctx.http(), EditInteractionResponse::default().components(vec![])).await?;
    return Ok(());
}

pub async fn extract_time_range_from_modal(
    ctx: &Context, modal: &ModalInteraction
) -> Result<Option<(u64, u64)>, Error> {
    let components = &modal.data.components;

    let mut start = None;
    let mut end = None;

    for row in components {
        for component in &row.components {
            if let ActionRowComponent::InputText(input) = component {
                match input.custom_id.as_str() {
                    "report-start" => start = input.value.clone(),
                    "report-end" => end = input.value.clone(),
                    _ => {}
                }
            }
        }
    }

    let Some(start) = start.and_then(|s| dateparser::parse_with_timezone(
        &s, &chrono::offset::Utc
    ).ok()) else {
        util::edit_reply(ctx, Modal(modal), "Error: invalid or empty start time!").await?;
        return Ok(None);
    };

    let Some(end) = end.and_then(|s| dateparser::parse_with_timezone(
        &s, &chrono::offset::Utc
    ).ok()) else {
        util::edit_reply(ctx, Modal(modal), "Error: invalid or empty end time!").await?;
        return Ok(None);
    };

    if start.timestamp() >= end.timestamp() {
        util::edit_reply(ctx, Modal(modal), "Error: start time must be before end time!").await?;
        return Ok(None);
    }

    Ok(Some((start.timestamp() as u64, end.timestamp() as u64)))
}

pub async fn process_stat_template_check_form(
    ctx: &Context, data: &Data, modal: &ModalInteraction
) -> Result<(), Error> {
    if !data.inner.queues.lock().await.contains_key(&modal.channel_id) {
        util::direct_reply(
            ctx, Modal(modal),
            "Invalid interaction: no queue linked to channel", true
        ).await?;

        return Ok(());
    }

    modal.defer_ephemeral(ctx.http()).await?;

    let components = &modal.data.components;

    let mut template = None;

    for row in components {
        for component in &row.components {
            if let ActionRowComponent::InputText(input) = component {
                match input.custom_id.as_str() {
                    "template-id" => template = input.value.clone(),
                    _ => {}
                }
            }
        }
    }

    let Some(template) = template else {
        return Ok(());
    };

    let entries = ReportEntry::query_template_stats(
        &data.inner.pool, modal.channel_id, Some(template)
    ).await?;

    if entries.is_empty() {
        util::edit_reply(ctx, Modal(modal), "Error: could not find a template matching that ID!").await?;
    } else {
        modal.edit_response(
            ctx.http(), EditInteractionResponse::new().embed(
                create_template_embed(&entries[0])
            )
        ).await?;
    }

    Ok(())
}
