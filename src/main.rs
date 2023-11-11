use std::env;

use dotenvy::dotenv;
use poise::serenity_prelude::{self as serenity};

mod channel;
mod config;

pub struct AppContext {
    pub config: config::Config,
}

type Context<'a> = poise::Context<'a, AppContext, anyhow::Error>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt().with_ansi(false).init();

    let config = config::Config::from_env();
    let app_context = AppContext { config };
    let token = env::var("DISCORD_TOKEN").expect("discord token");

    let intents =
        serenity::GatewayIntents::GUILD_MEMBERS | serenity::GatewayIntents::GUILD_MESSAGES;

    let framework: poise::FrameworkBuilder<AppContext, anyhow::Error> = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![channel::role(), channel::archive()],
            ..Default::default()
        })
        .token(token)
        .intents(intents)
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                let guild_id = ready.guilds[0].id;
                poise::builtins::register_in_guild(&ctx, &framework.options().commands, guild_id)
                    .await?;
                Ok(app_context)
            })
        });

    Ok(framework.run().await?)
}
