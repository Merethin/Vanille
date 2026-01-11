use itertools::Itertools;
use serenity::all::{ButtonStyle, ChannelId, ChannelType, CreateActionRow, CreateButton, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, FormattedTimestamp, FormattedTimestampStyle, Mentionable, RoleId};

use caramel::ns::{UserAgent, format::prettify_name};

use crate::models::{queue::{Nation, Queue}, session::RecruitDelay};

pub fn create_queue_embed(
    queue: &Queue
) -> (CreateEmbed, Vec<CreateActionRow>) {
    let embed = CreateEmbed::new().title(
        format!("{} Recruitment Center", prettify_name(&queue.region))
    ).fields(vec![
        ("Nations in Queue", format!("`{}`", &queue.amount_in_queue().to_string()), false),
        ("Last Nation Added", FormattedTimestamp::new(queue.last_updated(), Some(FormattedTimestampStyle::RelativeTime)).to_string(), false),
        ("Last Telegram Sent", FormattedTimestamp::new(queue.last_telegram_sent(), Some(FormattedTimestampStyle::RelativeTime)).to_string(), false),
    ]);

    let components = vec![
        CreateActionRow::Buttons(vec![
            CreateButton::new("recruit-oneshot").label("Recruit: Oneshot"),
            CreateButton::new("recruit-stream").label("Recruit: Stream"),
        ]),
        CreateActionRow::Buttons(vec![
            CreateButton::new("setup").label("Setup Templates").style(ButtonStyle::Danger),
            CreateButton::new("statistics").label("Statistics").emoji('📊').style(ButtonStyle::Success),
        ]),
    ];

    (embed, components)
}

pub fn create_telegram_embed(
    nations: &Vec<Nation>,
    template: &String,
    sender: &String,
    cooldown: i64,
    user_agent: &UserAgent,
    add_stop_button: bool,
) -> (CreateEmbed, Vec<CreateActionRow>) {
    let embed = CreateEmbed::new().field(
        "Recipients", nations.iter().map(|v| &v.name).join(", "), false
    ).field(
        "Template", format!("`{}`", template), true
    ).field(
        "Cooldown Ends", format!("<t:{}:R>", cooldown), true
    );

    let mut row = vec![CreateButton::new_link(format!(
        "https://www.nationstates.net/container={}/nation={}/page=compose_telegram?tgto={}&message={}&generated_by={}",
        sender, sender,
        nations.iter().map(|v| &v.name).join(","),
        urlencoding::encode(template).to_string(),
        user_agent.web()
    )).label("Send Telegram")];

    if add_stop_button {
        row.push(
            CreateButton::new("stream-end").style(ButtonStyle::Danger).label("Stop Session")
        );
    }

    (embed, vec![CreateActionRow::Buttons(row)])
}

pub fn create_statistics_embed() -> (CreateEmbed, Vec<CreateActionRow>) {
    let embed = CreateEmbed::new().title("Recruitment Statistics");

    let components = vec![
        CreateActionRow::Buttons(vec![
            CreateButton::new("stat-leaders-all").label("Leaderboard (All Time)").style(ButtonStyle::Danger),
            CreateButton::new("stat-csv-all").label("CSV (All Time)").style(ButtonStyle::Success),
        ]),
        CreateActionRow::Buttons(vec![
            CreateButton::new("stat-leaders-custom").label("Leaderboard (Custom)").style(ButtonStyle::Danger),
            CreateButton::new("stat-csv-custom").label("CSV (Custom)").style(ButtonStyle::Success),
        ]),
    ];

    (embed, components)
}

pub fn create_session_start_embed(
    nation: &String,
    delay: &RecruitDelay
) -> (CreateEmbed, Vec<CreateActionRow>) {
    let embed = CreateEmbed::new().title(
        "Session Started"
    ).description(
        "Press the 'Stop Session' button on this embed or any subsequent embed to end the session."
    ).field(
        "Started by", nation, true
    ).field(
        "Delay", delay.to_string(), true
    );

    let components = vec![
        CreateActionRow::Buttons(vec![
            CreateButton::new("stream-end").label("Stop Session").style(ButtonStyle::Danger),
        ]),
    ];

    (embed, components)
}

pub fn create_edit_queue_embed(
    region: &String,
    size: usize,
    filter: &Vec<String>,
    thresholds: &Option<(u64, u64)>,
    ping_channel: &Option<ChannelId>,
    ping_role: &Option<RoleId>,
) -> (CreateEmbed, Vec<CreateActionRow>) {
    let embed = CreateEmbed::new().title(
        format!("Editing Queue: {}", prettify_name(&region))
    ).field(
        "Maximum Size", size.to_string(), false
    ).field(
        "Excluded Regions", if filter.is_empty() { "None".into() } else { filter.iter().join(", ") }, false
    ).field(
        "Reminder Threshold", thresholds.map_or(
            "No reminders".into(), |(fill, time)| format!("Queue over {fill} nations and last telegram over {time} minutes")
        ), false
    ).field(
        "Reminder Role", ping_role.map_or(
            "None (reminders won't ping)".into(), |role| role.mention().to_string()
        ), false
    ).field(
        "Reminder Channel", ping_channel.map_or(
            "None (reminders won't be sent)".into(), |channel| channel.mention().to_string()
        ), false
    );

    (embed, vec![
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("edit-queue-role", CreateSelectMenuKind::Role { default_roles: None }).placeholder(
                "Select a reminder role"
            )
        ),
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new("edit-queue-channel", CreateSelectMenuKind::Channel { 
                channel_types: Some(vec![ChannelType::Text]), default_channels: None 
            }).placeholder(
                "Select a reminder channel"
            ),
        ),
        CreateActionRow::Buttons(vec![
            CreateButton::new("edit-queue-size").label("Edit Size"),
            CreateButton::new("edit-queue-regions").label("Edit Excluded Regions")
        ]),
        CreateActionRow::Buttons(vec![
            CreateButton::new("edit-queue-threshold").label("Edit Threshold"),
            CreateButton::new("delete-queue-threshold").label("Delete Threshold").style(ButtonStyle::Danger)
        ]),
        CreateActionRow::Buttons(vec![
            CreateButton::new("clear-queue-role-channel").label("Clear Role and Channel").style(ButtonStyle::Danger)
        ]),
    ])
}