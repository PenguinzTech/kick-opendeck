use crate::auth::get_valid_token;
use crate::settings::EmptySettings;
use crate::kick_api;
use openaction::{Action, Instance, OpenActionResult, async_trait};
use serde_json::Value;

pub struct ViewerCountAction;

fn format_viewers(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 10_000 {
        format!("{:.0}K", n as f64 / 1_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

#[async_trait]
impl Action for ViewerCountAction {
    type Settings = EmptySettings;
    const UUID: &'static str = "dev.penguin.kick.viewercount";

    async fn will_appear(&self, instance: &Instance, _settings: &Self::Settings) -> OpenActionResult<()> {
        match get_valid_token().await {
            Some((token, user_id)) => {
                match kick_api::get_viewer_count(&token, &user_id).await {
                    Ok(Some(count)) => instance.set_title(Some(format_viewers(count)), None).await?,
                    Ok(None) => instance.set_title(Some("Offline"), None).await?,
                    Err(_) => instance.set_title(Some("N/A"), None).await?,
                }
            }
            None => instance.set_title(Some("N/A"), None).await?,
        }

        let instance_id = instance.instance_id.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            interval.tick().await;
            loop {
                interval.tick().await;
                let Some(inst) = openaction::get_instance(instance_id.clone()).await else { break };
                if let Some((token, user_id)) = get_valid_token().await {
                    match kick_api::get_viewer_count(&token, &user_id).await {
                        Ok(Some(count)) => { let _ = inst.set_title(Some(format_viewers(count)), None).await; }
                        Ok(None) => { let _ = inst.set_title(Some("Offline"), None).await; }
                        Err(_) => {}
                    }
                }
            }
        });
        Ok(())
    }

    async fn send_to_plugin(&self, instance: &Instance, _settings: &Self::Settings, payload: &Value) -> OpenActionResult<()> {
        crate::auth_handler::handle_auth_message(instance, payload).await
    }
}
