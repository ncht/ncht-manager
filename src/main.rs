use dotenv::dotenv;
use serenity::{
    framework::{standard::macros::*, StandardFramework},
    model::gateway::Ready,
    prelude::*,
    Result,
};
use std::env;

mod channel;
mod config;

use channel::*;

#[group]
#[commands(archive, restore)]
struct Channel;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected", ready.user.name);
    }
}

fn main() -> Result<()> {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("discord token");

    let mut client = Client::new(token, Handler).expect("create client");

    let app = client.cache_and_http.http.get_current_application_info()?;

    let bot_id = app.id;

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.allow_dm(false).on_mention(Some(bot_id)).prefix("!"))
            .group(&CHANNEL_GROUP),
    );

    client.start()
}
