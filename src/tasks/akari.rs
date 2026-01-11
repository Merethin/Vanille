use std::process::exit;
use serenity::all::Context;
use futures::future;
use log::error;

use caramel::akari;

use crate::bot::Data;
use crate::models::report::ReportEntry;

pub async fn akari_task(ctx: Context, data: Data) {
    let mut consumer = akari::create_consumer(
        &data.inner.channel, &data.inner.config.input.exchange_name, Some(vec!["nfound", "nrefound", "move"])
    ).await.unwrap_or_else(|err| {
        error!("Failed to create Akari consumer: {}", err);
        exit(1);
    });

    while let Some(event) = akari::consume(&mut consumer).await {
        match event.category.as_str() {
            "nfound" | "nrefound" => {
                let nation = event.actor.expect(&format!("{} event doesn't have a nation", event.category));
                let region = event.origin.expect(&format!("{} event doesn't have a region", event.category));

                let queue_updates = {
                    let mut queues = data.inner.queues.lock().await;

                    queues.values_mut().flat_map(|queue| {
                        queue.add_to_queue(&nation, match event.category.as_str() {
                            "nfound" => "newfound",
                            "nrefound" => "refound",
                            _ => unreachable!(),
                        }, &region)
                    }).collect::<Vec<_>>()
                };

                future::join_all(queue_updates.into_iter().map(async |update| {
                    update.execute(ctx.clone()).await;
                })).await;
            },
            "move" => {
                let nation = event.actor.expect(&format!("{} event doesn't have a nation", event.category));
                let region = event.destination.expect(&format!("{} event doesn't have a region", event.category));

                let channels = {
                    let queues = data.inner.queues.lock().await;

                    queues.values().flat_map(|queue| {
                        if region == queue.region {
                            Some(queue.channel)
                        } else { None }
                    }).collect::<Vec<_>>()
                };

                for queue in channels {
                    ReportEntry::mark_move(&data.inner.pool, queue, &nation, event.time).await;
                }
            }
            _ => ()
        }
    }
}