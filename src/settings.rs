use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use tokio::sync::RwLock;

pub const BUTTON_LABEL_MAX: usize = 10;

/// Global plugin settings persisted in OpenDeck
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GlobalSettings {
    /// Kick OAuth2 Client ID
    pub client_id: String,
    /// Kick OAuth2 Client Secret
    pub client_secret: String,
    /// OAuth2 access token
    pub access_token: Option<String>,
    /// OAuth2 refresh token
    pub refresh_token: Option<String>,
    /// Unix timestamp when access_token expires
    pub token_expires_at: Option<i64>,
    /// Kick broadcaster user ID
    pub user_id: Option<String>,
    /// Kick username (for display)
    pub username: Option<String>,
    /// Kick channel slug
    pub channel_slug: Option<String>,
    /// Kick chatroom ID
    pub chatroom_id: Option<String>,
}

impl GlobalSettings {
    pub fn is_authenticated(&self) -> bool {
        self.access_token.is_some()
            && self.user_id.is_some()
            && !self.client_id.is_empty()
    }
}

pub static SETTINGS: LazyLock<RwLock<GlobalSettings>> =
    LazyLock::new(|| RwLock::new(GlobalSettings::default()));

pub async fn save_settings(settings: GlobalSettings) -> openaction::OpenActionResult<()> {
    *SETTINGS.write().await = settings.clone();
    openaction::set_global_settings(&settings).await
}

pub async fn read_settings() -> GlobalSettings {
    SETTINGS.read().await.clone()
}

/// Settings for Send Chat Message action
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ChatMessageSettings {
    pub message: String,
    #[serde(default)]
    pub button_label: Option<String>,
}

/// Settings for Slow Chat action
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SlowChatSettings {
    pub slow_mode_seconds: u32,
    #[serde(default)]
    pub button_label: Option<String>,
}

impl Default for SlowChatSettings {
    fn default() -> Self {
        Self { slow_mode_seconds: 30, button_label: None }
    }
}

/// Settings for Ban User action
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BanUserSettings {
    pub target_username: String,
    #[serde(default)]
    pub ban_duration_minutes: Option<u32>,
    #[serde(default)]
    pub button_label: Option<String>,
}

/// Settings for Unban User action
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UnbanUserSettings {
    pub target_username: String,
    #[serde(default)]
    pub button_label: Option<String>,
}

/// Settings for Mute User action
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MuteUserSettings {
    pub target_username: String,
    #[serde(default)]
    pub button_label: Option<String>,
}

/// Settings for actions with no configuration (viewer_count)
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EmptySettings {
    #[serde(default)]
    pub button_label: Option<String>,
}
