use crate::auth::refresh_access_token;
use crate::settings::{read_settings, save_settings};
use chrono::Utc;
use openaction::{Action, Instance, OpenActionResult, async_trait};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SetupSettings {}

pub struct SetupAction;

#[async_trait]
impl Action for SetupAction {
    type Settings = SetupSettings;
    const UUID: &'static str = "io.pngz.kick.setup";

    async fn will_appear(&self, _instance: &Instance, _settings: &Self::Settings) -> OpenActionResult<()> {
        Ok(())
    }

    async fn key_down(&self, instance: &Instance, _settings: &Self::Settings) -> OpenActionResult<()> {
        let settings = read_settings().await;
        let refresh_token = match settings.refresh_token.clone() {
            Some(rt) if !rt.is_empty() => rt,
            _ => {
                log::warn!("Setup: no refresh token available — user must log in first");
                instance.show_alert().await?;
                return Ok(());
            }
        };

        match refresh_access_token(&settings.client_id, &settings.client_secret, &refresh_token).await {
            Ok(new_token) => {
                let new_expires_at = Utc::now().timestamp() + new_token.expires_in.unwrap_or(3600);
                let mut updated = settings;
                updated.access_token = Some(new_token.access_token);
                if new_token.refresh_token.is_some() {
                    updated.refresh_token = new_token.refresh_token;
                }
                updated.token_expires_at = Some(new_expires_at);
                if let Err(e) = save_settings(updated).await {
                    log::error!("Setup: failed to save refreshed token: {}", e);
                    instance.show_alert().await?;
                } else {
                    log::info!("Setup: token refreshed successfully");
                    instance.show_ok().await?;
                }
            }
            Err(e) => {
                log::error!("Setup: token refresh failed: {}", e);
                instance.show_alert().await?;
            }
        }
        Ok(())
    }

    async fn send_to_plugin(&self, instance: &Instance, _settings: &Self::Settings, payload: &serde_json::Value) -> OpenActionResult<()> {
        crate::auth_handler::handle_auth_message(instance, payload).await
    }
}
