use crate::auth::get_valid_token;
use crate::settings::ChatMessageSettings;
use crate::kick_api;
use openaction::{Action, Instance, OpenActionResult, async_trait};

pub struct ChatMessageAction;

#[async_trait]
impl Action for ChatMessageAction {
    type Settings = ChatMessageSettings;
    const UUID: &'static str = "dev.penguin.kick.chatmessage";

    async fn will_appear(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
        if let Some(l) = &settings.button_label { crate::auth_handler::set_bold_title(instance, Some(l.as_str())).await?; }
        Ok(())
    }

    async fn key_down(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
        if settings.message.is_empty() {
            instance.show_alert().await?;
            return Ok(());
        }
        match get_valid_token().await {
            Some((token, user_id)) => {
                match kick_api::send_chat_message(&token, &user_id, &settings.message).await {
                    Ok(_) => instance.show_ok().await?,
                    Err(e) => { log::error!("send_chat_message failed: {}", e); instance.show_alert().await?; }
                }
            }
            None => instance.show_alert().await?,
        }
        Ok(())
    }

    async fn send_to_plugin(&self, instance: &Instance, _settings: &Self::Settings, payload: &serde_json::Value) -> OpenActionResult<()> {
        crate::auth_handler::handle_auth_message(instance, payload).await
    }
}
