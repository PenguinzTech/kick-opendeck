use crate::auth::get_valid_token;
use crate::settings::{BanUserSettings, read_settings};
use crate::kick_api;
use openaction::{Action, Instance, OpenActionResult, async_trait};
use serde_json::Value;

pub struct BanUserAction;

#[async_trait]
impl Action for BanUserAction {
    type Settings = BanUserSettings;
    const UUID: &'static str = "dev.penguin.kick.banuser";

    async fn will_appear(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
        if let Some(l) = &settings.button_label { crate::auth_handler::set_bold_title(instance, Some(l.as_str())).await?; }
        Ok(())
    }

    async fn key_down(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
        if settings.target_username.is_empty() {
            instance.show_alert().await?;
            return Ok(());
        }
        let global = read_settings().await;
        let slug = match &global.channel_slug {
            Some(s) => s.clone(),
            None => { instance.show_alert().await?; return Ok(()); }
        };

        match get_valid_token().await {
            Some((token, _)) => {
                match kick_api::ban_user(&token, &slug, &settings.target_username, settings.ban_duration_minutes).await {
                    Ok(_) => instance.show_ok().await?,
                    Err(e) => { log::error!("ban_user failed: {}", e); instance.show_alert().await?; }
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
