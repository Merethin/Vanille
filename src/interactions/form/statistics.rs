use itertools::Itertools;
use serenity::all::{
    ActionRowComponent, CacheHttp, ComponentInteraction, Context, CreateActionRow, CreateAttachment, CreateInputText, CreateInteractionResponse, CreateModal, EditInteractionResponse, InputTextStyle, ModalInteraction
};

use crate::bot::{Data, Error, util::{self, Modal}};
use crate::models::report::ReportEntry;

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

pub async fn process_stat_leaders_custom_form(
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

    let Some(range) = extract_time_range_from_modal(ctx, modal).await? else {
        return Ok(());
    };

    let leaders = ReportEntry::count(
        &data.inner.pool, modal.channel_id, Some(range)
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

pub async fn process_stat_csv_custom_form(
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

    let Some(range) = extract_time_range_from_modal(ctx, modal).await? else {
        return Ok(());
    };

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

async fn extract_time_range_from_modal(
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