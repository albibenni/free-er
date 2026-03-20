use crate::ipc_client;
use crate::sections::settings::{AI_SITES, SEARCH_ENGINES};
use relm4::ComponentSender;
use shared::ipc::Command;
use tracing::error;
use uuid::Uuid;

use super::{App, AppMsg};

pub(super) fn add_url(
    url: String,
    default_rule_set_id: Option<Uuid>,
    sender: ComponentSender<App>,
) {
    tokio::spawn(async move {
        if let Some(id) = default_rule_set_id {
            if let Err(e) = ipc_client::send(&Command::AddUrlToRuleSet {
                rule_set_id: id,
                url,
            })
            .await
            {
                error!("AddUrlToRuleSet IPC failed: {e}");
            }
        } else {
            match ipc_client::add_rule_set("Default").await {
                Ok(id) => {
                    sender.input(AppMsg::RefreshRuleSets);
                    if let Err(e) = ipc_client::send(&Command::AddUrlToRuleSet {
                        rule_set_id: id,
                        url,
                    })
                    .await
                    {
                        error!("AddUrlToRuleSet IPC failed: {e}");
                    }
                }
                Err(e) => error!("AddRuleSet IPC failed: {e}"),
            }
        }
    });
}

pub(super) fn remove_url(url: String, default_rule_set_id: Option<Uuid>) {
    if let Some(id) = default_rule_set_id {
        tokio::spawn(async move {
            if let Err(e) = ipc_client::send(&Command::RemoveUrlFromRuleSet {
                rule_set_id: id,
                url,
            })
            .await
            {
                error!("RemoveUrlFromRuleSet IPC failed: {e}");
            }
        });
    }
}

pub(super) fn add_url_to_list(rule_set_id: Uuid, url: String) {
    tokio::spawn(async move {
        if let Err(e) = ipc_client::send(&Command::AddUrlToRuleSet { rule_set_id, url }).await {
            error!("AddUrlToRuleSet IPC failed: {e}");
        }
    });
}

pub(super) fn remove_url_from_list(rule_set_id: Uuid, url: String) {
    tokio::spawn(async move {
        if let Err(e) = ipc_client::send(&Command::RemoveUrlFromRuleSet { rule_set_id, url }).await
        {
            error!("RemoveUrlFromRuleSet IPC failed: {e}");
        }
    });
}

pub(super) fn create_rule_set(name: String, sender: ComponentSender<App>) {
    tokio::spawn(async move {
        match ipc_client::add_rule_set(&name).await {
            Ok(_) => sender.input(AppMsg::RefreshRuleSets),
            Err(e) => error!("AddRuleSet IPC failed: {e}"),
        }
    });
}

pub(super) fn delete_rule_set(id: Uuid, sender: ComponentSender<App>) {
    tokio::spawn(async move {
        match ipc_client::remove_rule_set(id).await {
            Ok(_) => sender.input(AppMsg::RefreshRuleSets),
            Err(e) => error!("RemoveRuleSet IPC failed: {e}"),
        }
    });
}

pub(super) fn toggle_ai_sites(
    enabled: bool,
    default_rule_set_id: Option<Uuid>,
    sender: ComponentSender<App>,
) {
    tokio::spawn(async move {
        let Some(id) = resolve_or_create_rule_set(default_rule_set_id, sender).await else {
            return;
        };
        for url in AI_SITES {
            let cmd = if enabled {
                Command::AddUrlToRuleSet {
                    rule_set_id: id,
                    url: url.to_string(),
                }
            } else {
                Command::RemoveUrlFromRuleSet {
                    rule_set_id: id,
                    url: url.to_string(),
                }
            };
            if let Err(e) = ipc_client::send(&cmd).await {
                error!("AI sites toggle IPC failed: {e}");
            }
        }
    });
}

pub(super) fn toggle_search_engines(
    enabled: bool,
    default_rule_set_id: Option<Uuid>,
    sender: ComponentSender<App>,
) {
    tokio::spawn(async move {
        let Some(id) = resolve_or_create_rule_set(default_rule_set_id, sender).await else {
            return;
        };
        for url in SEARCH_ENGINES {
            let cmd = if enabled {
                Command::AddUrlToRuleSet {
                    rule_set_id: id,
                    url: url.to_string(),
                }
            } else {
                Command::RemoveUrlFromRuleSet {
                    rule_set_id: id,
                    url: url.to_string(),
                }
            };
            if let Err(e) = ipc_client::send(&cmd).await {
                error!("Search engines toggle IPC failed: {e}");
            }
        }
    });
}

#[cfg(test)]
#[path = "url_handlers_tests.rs"]
mod tests;

async fn resolve_or_create_rule_set(
    existing: Option<Uuid>,
    sender: ComponentSender<App>,
) -> Option<Uuid> {
    if let Some(id) = existing {
        return Some(id);
    }
    match ipc_client::add_rule_set("Default").await {
        Ok(id) => {
            sender.input(AppMsg::RefreshRuleSets);
            Some(id)
        }
        Err(e) => {
            error!("AddRuleSet IPC failed: {e}");
            None
        }
    }
}
