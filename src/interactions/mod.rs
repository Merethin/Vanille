mod click;
mod form;
mod dropdown;

use serenity::all::{Context, ComponentInteraction, ModalInteraction, Member};

use crate::bot::{Data, Error};

pub fn check_interaction_authorization(member: &Option<Member>) -> Option<&'static str> {
    match member {
        None => {
            return Some("This interaction cannot be used in DMs!");
        },
        Some(member) => {
            if !member.permissions.map_or(false, |p| p.manage_guild()) {
                return Some("You are not allowed to use this interaction!");
            }

            return None;
        }
    }
}

pub async fn handle_component_interaction(
    ctx: &Context, data: &Data, component: &ComponentInteraction
) -> Result<(), Error> {
    match component.data.custom_id.as_str() {
        // Main embed buttons
        "recruit-oneshot" => click::handle_recruit_oneshot(ctx, data, component).await,
        "recruit-stream" => form::spawn_session_form(ctx, data, component).await,
        "setup" => form::spawn_setup_form(ctx, data, component).await,
        "statistics" => click::create_statistics_menu(ctx, data, component).await,
        // Statistics menu buttons
        "stat-leaders-all" => click::handle_stat_leaders_all(ctx, data, component).await,
        "stat-csv-all" => click::handle_stat_csv_all(ctx, data, component).await,
        "stat-leaders-custom" => form::spawn_stat_time_form(ctx, data, component, "stat-leaders-custom-report").await,
        "stat-csv-custom" => form::spawn_stat_time_form(ctx, data, component, "stat-csv-custom-report").await,
        // Session DM buttons
        "stream-end" => click::handle_stream_end(ctx, data, component).await,
        // Queue editing buttons
        "edit-queue-size" => form::spawn_queue_size_form(ctx, data, component).await,
        "edit-queue-regions" => form::spawn_queue_regions_form(ctx, data, component).await,
        "edit-queue-threshold" => form::spawn_queue_threshold_form(ctx, data, component).await,
        "delete-queue-threshold" => click::handle_delete_queue_threshold(ctx, data, component).await,
        "clear-queue-role-channel" => click::handle_clear_queue_role_and_channel(ctx, data, component).await,
        // Queue editing dropdowns
        "edit-queue-role" => dropdown::handle_edit_queue_role(ctx, data, component).await,
        "edit-queue-channel" => dropdown::handle_edit_queue_channel(ctx, data, component).await,
        _ => Ok(())
    }
}

pub async fn handle_modal_interaction(
    ctx: &Context, data: &Data, modal: &ModalInteraction
) -> Result<(), Error> {
    if let Some((custom_id, key)) = modal.data.custom_id.split_once(':') {
        match custom_id {
            "queue-size-modal" => form::process_queue_size_form(ctx, data, modal, key).await,
            "queue-regions-modal" => form::process_queue_regions_form(ctx, data, modal, key).await,
            "queue-threshold-modal" => form::process_queue_threshold_form(ctx, data, modal, key).await,
            _ => Ok(()),
        }
    } else {
        match modal.data.custom_id.as_str() {
            "setup-modal" => form::process_setup_form(ctx, data, modal).await,
            "stream-start-modal" => form::process_session_form(ctx, data, modal).await,
            "stat-leaders-custom-report" => form::process_stat_leaders_custom_form(ctx, data, modal).await,
            "stat-csv-custom-report" => form::process_stat_csv_custom_form(ctx, data, modal).await,
            _ => Ok(())
        }
    }
}