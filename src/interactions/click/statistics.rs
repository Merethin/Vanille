use itertools::Itertools;
use serenity::all::{
    CacheHttp, ComponentInteraction, Context, CreateAttachment, CreateInteractionResponse, 
    CreateInteractionResponseMessage, EditInteractionResponse
};

use crate::bot::{Data, Error, util::{self, Component}};
use crate::embeds::create_statistics_embed;
use crate::models::report::ReportEntry;

pub async fn create_statistics_menu(
    ctx: &Context, _: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    let (embed, components) = create_statistics_embed();

    Ok(component.create_response(ctx.http(), CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new().embed(embed).components(components).ephemeral(true)
    )).await?)
}

pub async fn handle_stat_leaders_all(
    ctx: &Context, data: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    if !data.inner.queues.lock().await.contains_key(&component.channel_id) {
        util::direct_reply(
            ctx, Component(&component), 
            "Invalid interaction: no queue linked to channel", 
            true
        ).await?;

        return Ok(());
    }

    util::defer_ephemeral(ctx, Component(component)).await?;

    let leaders = ReportEntry::count(
        &data.inner.pool, component.channel_id, None
    ).await?;

    if leaders.is_empty() {
        util::edit_reply(ctx, Component(component), "Error: no results recorded!").await?;
    } else {
        util::edit_reply(
            ctx, Component(component), 
            &format!(
                "All-time leaderboard:\n```\n{}\n```", 
                leaders.iter().map(|(nation, count)| format!("{nation}: {count}")).join("\n")
            )
        ).await?;
    }

    Ok(())
}

pub async fn handle_stat_csv_all(
    ctx: &Context, data: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    if !data.inner.queues.lock().await.contains_key(&component.channel_id) {
        util::direct_reply(
            ctx, Component(component),
            "Invalid interaction: no queue linked to channel", true
        ).await?;

        return Ok(());
    }

    util::defer_ephemeral(ctx, Component(component)).await?;

    let entries = ReportEntry::query(
        &data.inner.pool, component.channel_id, None
    ).await?;

    if entries.is_empty() {
        util::edit_reply(ctx, Component(component), "Error: no results recorded!").await?;
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

        component.edit_response(
ctx.http(), EditInteractionResponse::new().content(
                "All-time telegram data:"
            ).new_attachment(
                CreateAttachment::bytes(output, "vanille-report.csv")
            )
        ).await?;
    }

    Ok(())
}