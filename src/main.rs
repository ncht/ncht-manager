use dotenv::dotenv;
use serenity::{
    async_trait,
    framework::{standard::macros::*, StandardFramework},
    http::Http,
    model::gateway::Ready,
    prelude::*,
    Result,
};
use std::env;

mod channel;
mod config;

use channel::*;

#[group]
#[commands(archive, restore, role)]
struct Channel;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected", ready.user.name);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("discord token");

    let http = Http::new(&token);
    let user = http.get_current_user().await?;
    let intents = GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILD_MESSAGES;

    let framework = StandardFramework::new()
        .configure(|c| c.allow_dm(false).on_mention(Some(user.id)).prefix("!"))
        .group(&CHANNEL_GROUP);

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await?;

    client.start().await
}
