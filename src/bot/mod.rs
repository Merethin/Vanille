pub mod util;
mod handler;

use serenity::all::{ComponentInteraction, ClientBuilder, ChannelId, UserId, GatewayIntents};
use sqlx::PgPool;
use tokio::sync::Mutex;
use std::{collections::HashMap, error::Error as StdError, sync::Arc};

use caramel::ns::{UserAgent, api::Client};

use crate::{config::Config, models::{queue::Queue, session::Session, user_data::UserData}};
use crate::commands::create_command_list;

use handler::event_handler;

pub struct InnerData {
    pub pool: PgPool,
    pub user_agent: UserAgent,
    pub queues: Mutex<HashMap<ChannelId, Queue>>,
    pub sessions: Mutex<HashMap<UserId, Session>>,
    pub user_data: Mutex<HashMap<(ChannelId, UserId), UserData>>,
    pub cooldowns: Mutex<HashMap<(UserId, String), (i64, Option<ComponentInteraction>)>>,
    pub channel: lapin::Channel,
    pub config: Config,
    pub api_client: Client,
    pub interaction_tokens: Mutex<HashMap<String, String>>,
}

#[derive(Clone)]
pub struct Data {
    pub inner: Arc<InnerData>,
}

impl Data {
    pub fn new(
        pool: PgPool,
        user_agent: UserAgent,
        queues: HashMap<ChannelId, Queue>,
        user_data: HashMap<(ChannelId, UserId), UserData>,
        channel: lapin::Channel,
        config: Config,
        api_client: Client,
    ) -> Self {
        Data {
            inner: Arc::new(InnerData { 
                pool,
                user_agent: user_agent.clone(),
                queues: Mutex::new(queues),
                sessions: Mutex::new(HashMap::new()),
                user_data: Mutex::new(user_data),
                cooldowns: Mutex::new(HashMap::new()),
                channel,
                config,
                api_client,
                interaction_tokens: Mutex::new(HashMap::new()),
            }),
        }
    }
}

pub type Error = Box<dyn StdError + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn start_client(
    token: String,
    pool: PgPool,
    user_agent: UserAgent,
    channel: lapin::Channel,
    config: Config,
    client: Client
) -> Result<(), Error> {
    let intents = GatewayIntents::non_privileged() 
                                | GatewayIntents::MESSAGE_CONTENT
                                | GatewayIntents::GUILD_MEMBERS;

    let framework = 
        poise::Framework::builder().options(poise::FrameworkOptions {
            commands: create_command_list(),
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        }).setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                let queues = Queue::query(&pool).await?;
                let user_data= UserData::query(&pool).await?;
                Ok(Data::new(pool, user_agent, queues, user_data, channel, config, client))
            })
        }).build();

    let mut client = ClientBuilder::new(
        token, intents
    ).framework(framework).await?;

    client.start().await?;

    Ok(())
}