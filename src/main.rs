mod auth;
mod auth_handler;
mod global_handler;
mod settings;
mod kick_api;
mod actions;

use openaction::{OpenActionResult, register_action, run};
use openaction::global_events::set_global_event_handler;
use global_handler::KickGlobalHandler;
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode, ColorChoice};

use crate::actions::{
    chat_message::ChatMessageAction,
    viewer_count::ViewerCountAction,
    slow_chat::SlowChatAction,
    ban_user::BanUserAction,
    unban_user::UnbanUserAction,
    mute_user::MuteUserAction,
};

#[tokio::main]
async fn main() -> OpenActionResult<()> {
    let _ = TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );

    set_global_event_handler(Box::leak(Box::new(KickGlobalHandler)));

    register_action(ChatMessageAction).await;
    register_action(ViewerCountAction).await;
    register_action(SlowChatAction).await;
    register_action(BanUserAction).await;
    register_action(UnbanUserAction).await;
    register_action(MuteUserAction).await;

    run(std::env::args().collect()).await
}
