use serenity::all::{Context, FullEvent, Interaction, Ready};
use futures::future;
use log::warn;

use crate::bot::{Data, Error};
use crate::tasks::spawn_background_tasks;
use crate::interactions::{handle_component_interaction, handle_modal_interaction};

pub async fn event_handler(
    ctx: &Context,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot, .. } => {
            warn!("Logged in as {}", data_about_bot.user.name);

            on_ready(ctx, data, data_about_bot).await;

            Ok(())
        },
        FullEvent::InteractionCreate { interaction } => {
            match interaction {
                Interaction::Component(component) => {
                    handle_component_interaction(ctx, data, component).await.or_else(|err| {
                        warn!("Component interaction failed: {}", err);
                        Err(err)
                    })
                },
                Interaction::Modal(modal) => {
                    handle_modal_interaction(ctx, data, modal).await.or_else(|err| {
                        warn!("Modal interaction failed: {}", err);
                        Err(err)
                    })
                },
                _ => Ok(()),
            }
        }
        _ => {
            Ok(())
        }
    }
}

async fn on_ready(ctx: &Context, data: &Data, _: &Ready) {
    spawn_background_tasks(ctx, data).await;
    update_all_queues(ctx, data).await;
}

// FIXME: find a better location
async fn update_all_queues(ctx: &Context, data: &Data) {
    let queue_updates = {
        let queues = data.inner.queues.lock().await;

        queues.values().map(|queue| {
            queue.generate_queue_update()
        }).collect::<Vec<_>>()
    };

    future::join_all(queue_updates.into_iter().map(async |update| {
        update.execute(ctx.clone()).await;
    })).await;
}