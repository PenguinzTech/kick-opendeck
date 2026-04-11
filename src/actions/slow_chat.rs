use crate::auth::get_valid_token;
use crate::settings::{SlowChatSettings, read_settings};
use crate::kick_api;
use openaction::{Action, Instance, OpenActionResult, async_trait};
use serde_json::Value;
use std::sync::atomic::Ordering;

pub struct SlowChatAction;

#[async_trait]
impl Action for SlowChatAction {
    type Settings = SlowChatSettings;
    const UUID: &'static str = "io.pngz.kick.slowchat";

    async fn will_appear(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
        use crate::auth_handler::{restore_title, set_button_image};
        restore_title(instance, settings.button_label.as_deref()).await?;
        set_button_image(instance, settings.button_image.as_deref()).await?;
        Ok(())
    }

    async fn key_down(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
        let global = read_settings().await;
        let chatroom_id = match &global.chatroom_id {
            Some(id) => id.clone(),
            None => { instance.show_alert().await?; return Ok(()); }
        };

        match get_valid_token().await {
            Some((token, _)) => {
                let current_state = instance.current_state_index.load(Ordering::Relaxed);
                let new_seconds = if current_state == 0 { settings.slow_mode_seconds } else { 0 };
                match kick_api::set_slow_mode(&token, &chatroom_id, new_seconds).await {
                    Ok(_) => {
                        let new_state = if new_seconds > 0 { 1 } else { 0 };
                        instance.set_state(new_state).await?;
                        instance.show_ok().await?;
                    }
                    Err(e) => { log::error!("set_slow_mode failed: {}", e); instance.show_alert().await?; }
                }
            }
            None => instance.show_alert().await?,
        }
        Ok(())
    }

    async fn send_to_plugin(&self, instance: &Instance, _settings: &Self::Settings, payload: &Value) -> OpenActionResult<()> {
        crate::auth_handler::handle_auth_message(instance, payload).await
    }
}
