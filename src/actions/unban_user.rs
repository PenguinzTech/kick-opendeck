use crate::auth::get_valid_token;
use crate::settings::{UnbanUserSettings, read_settings};
use crate::kick_api;
use openaction::{Action, Instance, OpenActionResult, async_trait};
use serde_json::Value;

pub struct UnbanUserAction;

#[async_trait]
impl Action for UnbanUserAction {
    type Settings = UnbanUserSettings;
    const UUID: &'static str = "io.pngz.kick.unbanuser";

    async fn will_appear(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
        use crate::auth_handler::{restore_title, set_button_image};
        restore_title(instance, settings.button_label.as_deref()).await?;
        set_button_image(instance, settings.button_image.as_deref()).await?;
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
                match kick_api::unban_user(&token, &slug, &settings.target_username).await {
                    Ok(_) => instance.show_ok().await?,
                    Err(e) => { log::error!("unban_user failed: {}", e); instance.show_alert().await?; }
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
