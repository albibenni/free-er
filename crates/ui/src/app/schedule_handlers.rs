use crate::ipc_client;
use relm4::ComponentSender;
use shared::ipc::{ScheduleSummary, ScheduleType};
use tracing::{error, warn};
use uuid::Uuid;

use super::{App, AppMsg};

pub(super) fn create_schedule(
    name: String,
    days: Vec<u8>,
    start_min: u32,
    end_min: u32,
    specific_date: String,
    rule_set_id: Option<Uuid>,
    schedule_type: ScheduleType,
    sender: ComponentSender<App>,
) {
    tokio::spawn(async move {
        match ipc_client::add_schedule(
            &name,
            days,
            start_min,
            end_min,
            Some(specific_date),
            rule_set_id,
            schedule_type,
        )
        .await
        {
            Ok(_) => sender.input(AppMsg::RefreshSchedules),
            Err(e) => error!("add_schedule failed: {e}"),
        }
    });
}

pub(super) fn update_schedule(
    id: Uuid,
    name: String,
    days: Vec<u8>,
    start_min: u32,
    end_min: u32,
    rule_set_id: Option<Uuid>,
    specific_date: Option<String>,
    schedule_type: ScheduleType,
    sender: ComponentSender<App>,
) {
    tokio::spawn(async move {
        match ipc_client::update_schedule(
            id,
            &name,
            days,
            start_min,
            end_min,
            rule_set_id,
            specific_date,
            schedule_type,
        )
        .await
        {
            Ok(_) => sender.input(AppMsg::RefreshSchedules),
            Err(e) => error!("update_schedule failed: {e}"),
        }
    });
}

pub(super) fn delete_schedule(id: Uuid, sender: ComponentSender<App>) {
    tokio::spawn(async move {
        match ipc_client::remove_schedule(id).await {
            Ok(_) => sender.input(AppMsg::RefreshSchedules),
            Err(e) => error!("remove_schedule failed: {e}"),
        }
    });
}

pub(super) fn refresh_schedules(sender: ComponentSender<App>) {
    tokio::spawn(async move {
        match ipc_client::list_schedules().await {
            Ok(schedules) => sender.input(AppMsg::SchedulesUpdated(schedules)),
            Err(e) => warn!("list_schedules failed: {e}"),
        }
    });
}

pub(super) fn schedules_updated(
    schedule_sender: &relm4::Sender<crate::sections::schedule::ScheduleInput>,
    schedules: Vec<ScheduleSummary>,
) {
    use crate::sections::schedule::ScheduleInput;
    schedule_sender.emit(ScheduleInput::SchedulesUpdated(schedules));
}
