use itertools::Itertools;
use regex::Regex;
use serenity::all::{ButtonStyle, ChannelId, ChannelType, CreateActionRow, CreateButton, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, FormattedTimestamp, FormattedTimestampStyle, Mentionable, RoleId, UserId};

use caramel::ns::{UserAgent, format::prettify_name};

use crate::models::{queue::{Nation, Queue}, report::TemplateStats, session::RecruitDelay};

pub fn create_queue_embed(
    queue: &Queue,
    sessions: Vec<UserId>,
    leaders: &Vec<(UserId, usize)>
) -> (CreateEmbed, Vec<CreateActionRow>) {
    let embed = CreateEmbed::new().title(
        format!("{} Recruitment Center", prettify_name(&queue.region))
    ).fields(vec![
        ("Nations in Queue", format!("`{}`", &queue.amount_in_queue().to_string()), false),
        ("Last Nation Added", FormattedTimestamp::new(queue.last_updated(), Some(FormattedTimestampStyle::RelativeTime)).to_string(), false),
        ("Last Telegram Sent", match queue.last_telegram_sent() {
            Some((time, user)) => {
                let t = FormattedTimestamp::new(time, Some(FormattedTimestampStyle::RelativeTime)).to_string();
                format!("{} by {}", t, user.mention())
            },
            None => "None".to_string()
        }, false),
        ("Sessions Active", if sessions.is_empty() { 
            "None".to_string() 
        } else { 
            sessions.into_iter().map(|v| v.mention()).join(" ") 
        }, false),
        ("Top Recruiters (Last 24h)", if leaders.is_empty() { 
            "None".to_string() 
        } else { 
            leaders.into_iter().take(3).map(|(user, count)| format!("{} - {} telegrams", user.mention(), count)).join("\n")
        }, false)
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

pub fn create_pause_embed() -> (CreateEmbed, Vec<CreateActionRow>) {
    let embed = CreateEmbed::new().title("Still Here?").description(
        "Recruitment session has been paused. Click Continue to resume it. Otherwise, the session will automatically be closed for inactivity in 5 minutes."
    );

    let row = vec![
        CreateButton::new("stream-resume").style(ButtonStyle::Success).label("Continue"),
        CreateButton::new("stream-end").style(ButtonStyle::Danger).label("Stop Session")
    ];

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
        CreateActionRow::Buttons(vec![
            CreateButton::new("stat-templates-top").label("Top Templates").style(ButtonStyle::Danger),
            CreateButton::new("stat-template-check").label("Check Template").style(ButtonStyle::Success),
        ]),
    ];

    (embed, components)
}

pub fn create_session_start_embed(
    nation: &String,
    delay: &RecruitDelay,
    session_type: &Option<String>
) -> (CreateEmbed, Vec<CreateActionRow>) {
    let embed = CreateEmbed::new().title(
        "Session Started"
    ).description(
        "Press the 'Stop Session' button on this embed or any subsequent embed to end the session."
    ).field(
        "Started by", nation, true
    ).field(
        "Delay", delay.to_string(), true
    ).field(
        "Activity check", if session_type.clone().unwrap_or("".into()).len() > 0 { "Reaction" } else { "Periodic" }, true
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
    regex_filters: &Vec<Regex>,
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
    ).field(
        "Regex Filters", if regex_filters.is_empty() { "None".into() } else { regex_filters.iter().map(|v| format!("`{}`", v.as_str())).join("\n") }, false
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
            CreateButton::new("edit-queue-regions").label("Edit Excluded Regions"),
            CreateButton::new("edit-queue-filter").label("Edit Filters")
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

pub fn create_template_embed(
    stats: &TemplateStats
) -> CreateEmbed {
    let overall_ratio = 100f64 * (stats.total_recruited as f64) / (stats.total_sent as f64) ;
    let newfound_ratio = 100f64 * (stats.recruited_newfounds as f64) / (stats.sent_newfounds as f64);
    let refound_ratio = 100f64 * (stats.recruited_refounds as f64) / (stats.sent_refounds as f64);

    CreateEmbed::new().title(
        format!("Template Statistics: {}", stats.template)
    ).field(
        "Created By", format!("`{}`", stats.sender), false
    ).field(
        "Total Telegrams", format!(
            "{} sent / {} recruited ({:.2}%)",
            stats.total_sent, stats.total_recruited, overall_ratio
        ), false
    ).field(
        "Telegrams to Newfounds", format!(
            "{} sent / {} recruited ({:.2}%)",
            stats.sent_newfounds, stats.recruited_newfounds, newfound_ratio
        ), false
    ).field(
        "Telegrams to Refounds", format!(
            "{} sent / {} recruited ({:.2}%)",
            stats.sent_refounds, stats.recruited_refounds, refound_ratio
        ), false
    )
}

const MIN_SAMPLE_SIZE: i64 = 1000; // Minimum sample size for a template to be considered
const TEMPLATE_COUNT: usize = 10; // Show only this number of templates

pub fn create_top_templates_embed(
    stats: &Vec<TemplateStats>
) -> CreateEmbed {
    // Sort by highest newfound ratio, hence inverse sort order and index 3
    let mut templates: Vec<_> = stats.iter().filter(
        |v| v.total_sent >= MIN_SAMPLE_SIZE
    ).map(|stats| {
        let overall_ratio = 100f64 * (stats.total_recruited as f64) / (stats.total_sent as f64) ;
        let newfound_ratio = 100f64 * (stats.recruited_newfounds as f64) / (stats.sent_newfounds as f64);
        let refound_ratio = 100f64 * (stats.recruited_refounds as f64) / (stats.sent_refounds as f64);

        (stats.template.clone(), stats.sender.clone(), overall_ratio, newfound_ratio, refound_ratio)
    }).sorted_by(|a, b| b.3.partial_cmp(&a.3).unwrap()).collect();

    templates.truncate(TEMPLATE_COUNT);

    CreateEmbed::new().title(
        "Best Performing Templates"
    ).description(
        if templates.is_empty() {
            format!("No templates match the sample size (at least {} telegrams sent)", MIN_SAMPLE_SIZE)
        } else {
            templates.into_iter().map(|v| {
                format!(
                    "`{}` by `{}`: {:.2}% total, {:.2}% newfounds, {:.2}% refounds",
                    v.0, v.1, v.2, v.3, v.4
                )
            }).join("\n")
        }
    )
}