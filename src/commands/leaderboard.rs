use std::{time::Duration, fmt::Write};

use poise::CreateReply;
use serenity::all::{ButtonStyle, ChannelId, ComponentInteractionCollector, ComponentInteractionDataKind, CreateActionRow, CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, EditMessage, ModalInteractionCollector};

use crate::{bot::{Context, Error}, interactions::form::{extract_time_range_from_modal, spawn_stat_time_form}, models::report::ReportEntry};

#[poise::command(slash_command)]
pub async fn leaderboard(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let mut queues: Vec<(ChannelId, String, String)> = ctx.data().inner.queues.lock().await.values().map(
        |v| (v.channel, v.region.clone(), "".into())
    ).collect();

    {
        let Some(guild) = ctx.guild() else {
            ctx.send(
                CreateReply::default().content("This command cannot be run in DMs!").ephemeral(true)
            ).await?;

            return Ok(());
        };

        queues.retain(|(channel, _, _)| guild.channels.contains_key(channel));
    }

    for (channel, _, name) in &mut queues {
        *name = channel.name(ctx.http()).await?;
    }

    let reply = ctx.send(
        CreateReply::default().content("Select the queues you want to query a leaderboard for:").ephemeral(true).components(
            vec![CreateActionRow::SelectMenu(
                CreateSelectMenu::new("leaderboard-queues", CreateSelectMenuKind::String { options: queues.iter().map(
                    |(channel, region, name)| {
                        CreateSelectMenuOption::new(name, channel.get().to_string()).description(format!("region: {region}"))
                    }
                ).collect() }).min_values(1).max_values(queues.len() as u8)
            ),
            CreateActionRow::Buttons(vec![
                CreateButton::new("leaderboard-all").label("All Time").style(ButtonStyle::Primary),
                CreateButton::new("leaderboard-custom").label("Custom Range").style(ButtonStyle::Secondary),
            ])]
        ).ephemeral(true)
    ).await?;

    let message = reply.message().await?;

    let mut selected_queues: Vec<i64> = vec![];
    let mut range: Option<(u64, u64)> = None;
    let mut selected: bool = false;

    while let Some(interaction) = ComponentInteractionCollector::new(ctx).message_id(message.id).timeout(Duration::from_secs(120)).await {
        match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => {
                selected_queues = values.iter().filter_map(|v| v.parse().ok()).collect();
                interaction.create_response(ctx.http(), CreateInteractionResponse::Acknowledge).await?;
                continue;
            },
            ComponentInteractionDataKind::Button => {
                if selected_queues.is_empty() {
                    interaction.create_response(ctx.http(), CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content("Please select queues!")
                    )).await?;

                    continue;
                }

                match interaction.data.custom_id.as_str() {
                    "leaderboard-all" => {
                        range = None;
                        selected = true;
                        interaction.create_response(ctx.http(), CreateInteractionResponse::Acknowledge).await?;
                        break;
                    },
                    "leaderboard-custom" => {
                        spawn_stat_time_form(ctx.serenity_context(), ctx.data(), &interaction, "leaderboard-custom-report").await?;

                        if let Some(modal) = ModalInteractionCollector::new(ctx).message_id(message.id).timeout(Duration::from_secs(60)).await {
                            range = extract_time_range_from_modal(ctx.serenity_context(), &modal).await?;
                            
                            if range.is_none() {
                                continue;
                            };

                            selected = true;
                            modal.create_response(ctx.http(), CreateInteractionResponse::Acknowledge).await?;
                            break;
                        }
                    },
                    _ => {},
                }
            },
            _ => {},
        }
    }

    if !selected {
        reply.edit(ctx, CreateReply::default().content("Select the queues you want to query a leaderboard for:").components(vec![])).await?;
        return Ok(());
    }

    let entries = ReportEntry::count_by_user(&ctx.data().inner.pool, &selected_queues, range).await?;
    let total: usize = entries.iter().map(|v| v.1).sum();

    if entries.is_empty() {
        reply.edit(
            ctx, CreateReply::default().content("Error: no results recorded!").components(vec![])
        ).await?;

        return Ok(());
    }

    let mut output = String::new();

    for (user, count) in entries {
        let name = user.to_user(ctx.http()).await?.name;
        writeln!(output, "{}: {}", name, count)?;
    }

    write!(output, "\ntotal: {}", total)?;
    
    if let Some(range) = range {
        reply.edit(
            ctx, CreateReply::default().content(format!(
                "Leaderboard from <t:{}:f> to <t:{}:f>:\n```\n{}\n```", 
                range.0, range.1, output
            )).components(vec![])
        ).await?;
    } else {
        reply.edit(
            ctx, CreateReply::default().content(format!(
                "All-time leaderboard:\n```\n{}\n```", 
                output
            )).components(vec![])
        ).await?;
    }

    Ok(())
}