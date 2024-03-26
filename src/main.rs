use anyhow::Context as _;
use poise::serenity_prelude::{self as serenity, ClientBuilder};
use shuttle_secrets::SecretStore;
use shuttle_serenity::ShuttleSerenity;

mod channel;
mod config;

pub struct AppContext {
    pub config: config::Config,
}

type Context<'a> = poise::Context<'a, AppContext, anyhow::Error>;

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    let config = config::Config::from_secret_store(&secret_store)?;
    let app_context = AppContext { config };
    let token = secret_store.get("DISCORD_TOKEN").context("DISCORD_TOKEN")?;

    let intents =
        serenity::GatewayIntents::GUILD_MEMBERS | serenity::GatewayIntents::GUILD_MESSAGES;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![channel::role(), channel::archive()],
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                let guild_id = ready.guilds[0].id;
                poise::builtins::register_in_guild(&ctx, &framework.options().commands, guild_id)
                    .await?;
                Ok(app_context)
            })
        })
        .build();

    let client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}
