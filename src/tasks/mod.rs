mod akari;
mod cooldown;
mod reminders;

use serenity::all::Context;
use tokio::sync::OnceCell;
use crate::bot::Data;

use cooldown::cooldown_task;
use reminders::reminders_task;
use akari::akari_task;

static BACKGROUND_TASK_LOCK: OnceCell<()> = OnceCell::const_new();

pub async fn spawn_background_tasks(ctx: &Context, data: &Data) {
    BACKGROUND_TASK_LOCK.get_or_init(|| async { 
        tokio::spawn(cooldown_task(ctx.clone(), data.clone()));
        tokio::spawn(reminders_task(ctx.clone(), data.clone()));
        tokio::spawn(akari_task(ctx.clone(), data.clone()));
    }).await;
}