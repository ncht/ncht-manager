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

    let http = Http::new_with_token(&token);
    let app = http.get_current_application_info().await?;
    let bot_id = app.id;

    let framework = StandardFramework::new()
        .configure(|c| c.allow_dm(false).on_mention(Some(bot_id)).prefix("!"))
        .group(&CHANNEL_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await?;

    client.start().await
}
