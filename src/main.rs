use dotenvy::dotenv;
use serenity::{
    async_trait,
    framework::{standard::macros::*, StandardFramework},
    http::Http,
    model::gateway::Ready,
    prelude::*,
    Result,
};
use std::env;
#[cfg(feature = "chatgpt")]
use std::sync::Arc;
use tracing::info;

mod channel;
#[cfg(feature = "chatgpt")]
mod chat;
mod config;

use channel::*;
#[cfg(feature = "chatgpt")]
use chat::*;

#[group]
#[commands(archive, restore, role)]
struct Channel;

#[cfg(feature = "chatgpt")]
#[group]
#[commands(chat, histsize)]
struct Chat;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is connected", ready.user.name);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    config::init_config();

    let token = env::var("DISCORD_TOKEN").expect("discord token");

    let http = Http::new(&token);
    let user = http.get_current_user().await?;
    let intents = GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = StandardFramework::new()
        .configure(|c| c.allow_dm(false).on_mention(Some(user.id)).prefix("!"));
    let framework = framework.group(&CHANNEL_GROUP);

    #[cfg(feature = "chatgpt")]
    let framework = framework.group(&CHAT_GROUP);

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await?;

    #[cfg(feature = "chatgpt")]
    {
        let mut data = client.data.write().await;
        data.insert::<chat::Data>(Arc::new(RwLock::new(chat::Data::default())))
    }

    client.start().await
}
