use std::{time::Duration, fmt::Write};

use itertools::Itertools;
use poise::CreateReply;
use serenity::all::{ButtonStyle, ChannelId, ComponentInteractionCollector, ComponentInteractionDataKind, CreateActionRow, CreateAttachment, CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, ModalInteractionCollector};
use thousands::Separable;

use crate::{bot::{Context, Error}, interactions::form::{extract_time_range_from_modal, spawn_stat_time_form}, models::{report::ReportEntry, user_data::UserData}};

fn create_leaderboard_menu(
    queues: &Vec<(ChannelId, String, String)>,
    bbcode: bool,
) -> Vec<CreateActionRow> {
    vec![
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("leaderboard-queues", CreateSelectMenuKind::String { options: queues.iter().map(
                |(channel, region, name)| {
                    CreateSelectMenuOption::new(name, channel.get().to_string()).description(format!("region: {region}"))
                }
            ).collect() }).min_values(1).max_values(queues.len() as u8)
        ),
        CreateActionRow::Buttons(vec![
            CreateButton::new("leaderboard-all").label("All Time").style(ButtonStyle::Primary),
            CreateButton::new("leaderboard-custom").label("Custom Range").style(ButtonStyle::Secondary),
            CreateButton::new("leaderboard-bbcode").label(
                if bbcode { "BBCode: On" } else { "BBCode: Off" }
            ).style(
                if bbcode { ButtonStyle::Success } else { ButtonStyle::Danger }
            ),
        ])
    ]
}

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
            create_leaderboard_menu(&queues, false)
        ).ephemeral(true)
    ).await?;

    let message = reply.message().await?;

    let mut selected_queues: Vec<i64> = vec![];
    let mut range: Option<(u64, u64)> = None;
    let mut selected: bool = false;
    let mut bbcode: bool = false;

    while let Some(interaction) = ComponentInteractionCollector::new(ctx).message_id(message.id).timeout(Duration::from_secs(120)).await {
        match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => {
                selected_queues = values.iter().filter_map(|v| v.parse().ok()).collect();
                interaction.create_response(ctx.http(), CreateInteractionResponse::Acknowledge).await?;
                continue;
            },
            ComponentInteractionDataKind::Button => {
                if interaction.data.custom_id == "leaderboard-bbcode" {
                    bbcode = !bbcode;
                    interaction.create_response(ctx.http(), CreateInteractionResponse::Acknowledge).await?;
                    reply.edit(ctx, CreateReply::default().components(create_leaderboard_menu(&queues, bbcode))).await?;
                    continue;
                }

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
        reply.edit(ctx, CreateReply::default().components(vec![])).await?;
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

    if !bbcode {
        for (user, count) in entries {
            let name = user.to_user(ctx.http()).await?.name;
            writeln!(output, "{}: {}", name, count)?;
        }
    } else {
        write!(output, "[table][tr][td][b]Rank[/b][/td][td][b]Recruiter[/b][/td][td][b]Total Telegrams[/b][/td][/tr]")?;

        for (index, (user, count)) in entries.into_iter().enumerate() {
            let nations = UserData::find_nations(&ctx.data().inner.pool, user, &selected_queues).await?;
            writeln!(
                output, 
                "[tr][td]{}[/td][td]{}[/td][td]{}[/td][/tr]", 
                index + 1, 
                nations.iter().map(|n| format!("[nation]{n}[/nation]")).join(" / "), 
                count.separate_with_commas()
            )?;
        }

        write!(output, "[/table]")?;
    }

    let mut attached: Option<String> = None;
    if output.len() > 1850 {
        attached = Some(output);
        output = "Output generated as attachment due to length - check your DMs".into();
    }
    
    let builder = if let Some(range) = range {
        CreateReply::default().content(format!(
            "Leaderboard from <t:{}:f> to <t:{}:f>:\n```\n{}\n```\nTotal: {}", 
            range.0, range.1, output, total
        )).components(vec![])
    } else {
        CreateReply::default().content(format!(
            "All-time leaderboard:\n```\n{}\n```\nTotal: {}", 
            output, total
        )).components(vec![])
    };

    reply.edit(ctx, builder).await?;

    if let Some(content) = attached {
        ctx.author().direct_message(
            ctx.http(), 
            CreateMessage::new().content("Generated leaderboard:").add_file(
                CreateAttachment::bytes(content, "leaderboard.txt")
            )
        ).await?;
    }

    Ok(())
}