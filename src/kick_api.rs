use reqwest::Client;
use serde::Deserialize;
use std::sync::LazyLock;

static HTTP: LazyLock<Client> = LazyLock::new(Client::new);
const API_BASE: &str = "https://api.kick.com/public/v1";

#[derive(Debug)]
pub enum KickApiError {
    Http(reqwest::Error),
    Api { status: u16, message: String },
}

impl From<reqwest::Error> for KickApiError {
    fn from(err: reqwest::Error) -> Self {
        KickApiError::Http(err)
    }
}

impl std::fmt::Display for KickApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KickApiError::Http(e) => write!(f, "HTTP error: {}", e),
            KickApiError::Api { status, message } => write!(f, "API error ({}): {}", status, message),
        }
    }
}

impl std::error::Error for KickApiError {}

fn auth_headers(token: &str) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Authorization", format!("Bearer {}", token).parse().unwrap());
    headers
}

async fn check_response(response: reqwest::Response) -> Result<serde_json::Value, KickApiError> {
    let status = response.status().as_u16();
    let body = response.text().await?;
    if status >= 200 && status < 300 {
        if body.is_empty() {
            Ok(serde_json::json!({}))
        } else {
            serde_json::from_str(&body).map_err(|_| KickApiError::Api {
                status,
                message: "Failed to parse response".to_string(),
            })
        }
    } else {
        let error_msg = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(String::from))
            .unwrap_or(body);
        Err(KickApiError::Api { status, message: error_msg })
    }
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ChannelInfo {
    pub id: Option<u64>,
    pub slug: Option<String>,
    pub is_live: Option<bool>,
    pub viewer_count: Option<u64>,
    pub chatroom_id: Option<u64>,
}

// ============================================================================
// API Functions
// ============================================================================

/// Get channel info by broadcaster user ID.
pub async fn get_channel(
    token: &str,
    broadcaster_user_id: &str,
) -> Result<Option<ChannelInfo>, KickApiError> {
    let url = format!(
        "{}/channels?broadcaster_user_id={}",
        API_BASE, broadcaster_user_id
    );

    let response = HTTP.get(&url).headers(auth_headers(token)).send().await?;
    let body = check_response(response).await?;

    // The API returns { "data": [...] }
    let channel = body
        .get("data")
        .and_then(|d| d.as_array())
        .and_then(|arr| arr.first())
        .and_then(|ch| serde_json::from_value::<ChannelInfo>(ch.clone()).ok());

    Ok(channel)
}

/// Send a chat message.
pub async fn send_chat_message(
    token: &str,
    broadcaster_user_id: &str,
    message: &str,
) -> Result<(), KickApiError> {
    let url = format!("{}/chat", API_BASE);
    let body = serde_json::json!({
        "broadcaster_user_id": broadcaster_user_id.parse::<u64>().unwrap_or(0),
        "content": message,
        "type": "user"
    });

    let response = HTTP
        .post(&url)
        .headers(auth_headers(token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    check_response(response).await?;
    Ok(())
}

/// Get viewer count for a channel.
pub async fn get_viewer_count(
    token: &str,
    broadcaster_user_id: &str,
) -> Result<Option<u64>, KickApiError> {
    let channel = get_channel(token, broadcaster_user_id).await?;
    Ok(channel.and_then(|c| {
        if c.is_live == Some(true) {
            c.viewer_count
        } else {
            None
        }
    }))
}

/// Toggle slow mode for a chatroom.
/// Uses the v2 chatroom settings endpoint.
/// `slow_mode_seconds`: 0 = off, >0 = on with that delay.
pub async fn set_slow_mode(
    token: &str,
    chatroom_id: &str,
    slow_mode_seconds: u32,
) -> Result<(), KickApiError> {
    let url = format!("https://api.kick.com/api/v2/chatrooms/{}", chatroom_id);
    let body = serde_json::json!({
        "slow_mode": slow_mode_seconds > 0,
        "message_interval": slow_mode_seconds
    });

    let response = HTTP
        .put(&url)
        .headers(auth_headers(token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    check_response(response).await?;
    Ok(())
}

/// Ban a user from the channel.
/// `duration_minutes`: None = permanent, Some(minutes) = temporary.
pub async fn ban_user(
    token: &str,
    channel_slug: &str,
    username: &str,
    duration_minutes: Option<u32>,
) -> Result<(), KickApiError> {
    let url = format!(
        "https://api.kick.com/api/v2/channels/{}/bans",
        channel_slug
    );

    let mut body = serde_json::json!({
        "banned_username": username,
        "permanent": duration_minutes.is_none()
    });

    if let Some(mins) = duration_minutes {
        body["duration"] = serde_json::json!(mins);
    }

    let response = HTTP
        .post(&url)
        .headers(auth_headers(token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    check_response(response).await?;
    Ok(())
}

/// Unban a user from the channel.
pub async fn unban_user(
    token: &str,
    channel_slug: &str,
    username: &str,
) -> Result<(), KickApiError> {
    let url = format!(
        "https://api.kick.com/api/v2/channels/{}/bans/{}",
        channel_slug, username
    );

    let response = HTTP
        .delete(&url)
        .headers(auth_headers(token))
        .send()
        .await?;

    let status = response.status().as_u16();
    if status >= 200 && status < 300 {
        Ok(())
    } else {
        let body = response.text().await?;
        Err(KickApiError::Api { status, message: body })
    }
}

/// Mute a user in the channel.
pub async fn mute_user(
    token: &str,
    channel_slug: &str,
    username: &str,
) -> Result<(), KickApiError> {
    let url = format!(
        "https://api.kick.com/api/v1/channels/{}/mute-user",
        channel_slug
    );
    let body = serde_json::json!({
        "username": username
    });

    let response = HTTP
        .post(&url)
        .headers(auth_headers(token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    check_response(response).await?;
    Ok(())
}
