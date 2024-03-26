use anyhow::anyhow;
use chrono::{Duration, Utc};
use poise::serenity_prelude::{self as serenity};
use serde_json::{json, Map};
use shuttle_runtime::tokio;

fn find_category<'a>(
    channels: &'a [serenity::GuildChannel],
    name: &str,
) -> anyhow::Result<&'a serenity::GuildChannel> {
    channels
        .iter()
        .find(|c| c.kind == serenity::ChannelType::Category && c.name == name)
        .ok_or_else(|| anyhow!(format!("`{}` category not found", name)))
}

async fn edit_channel_category(
    ctx: super::Context<'_>,
    channel_id: serenity::ChannelId,
    category_id: serenity::ChannelId,
) -> anyhow::Result<()> {
    let mut param = Map::new();
    param.insert("parent_id".to_owned(), json!(category_id));
    ctx.http().edit_channel(channel_id, &param, None).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn archive(ctx: super::Context<'_>) -> anyhow::Result<()> {
    let config = &ctx.data().config;
    let guild_id = ctx.guild_id().ok_or_else(|| anyhow!("no guild"))?;

    let channels = ctx.http().get_channels(guild_id).await?;

    let active_category = find_category(&channels, &config.active_category)?;
    let archive_category = find_category(&channels, &config.archive_category)?;

    let threshold_days = Duration::try_days(config.threshold_days)
        .ok_or_else(|| anyhow!("invalid threshold days"))?;

    let target_channles: Vec<_> = channels
        .iter()
        .filter(|channel| {
            if channel.kind != serenity::ChannelType::Text {
                return false;
            }

            if channel.parent_id != Some(active_category.id) {
                return false;
            }

            channel
                .last_message_id
                .map(|id| {
                    let threshold = Utc::now() - threshold_days;
                    let target = id.created_at().with_timezone(&Utc);

                    threshold > target
                })
                .unwrap_or(false)
        })
        .collect();

    if target_channles.is_empty() {
        ctx.reply("no target").await?;

        return Ok(());
    }

    let ids = target_channles
        .iter()
        .map(|c| format!("<#{}>", c.id))
        .collect::<Vec<_>>()
        .join(", ");

    for channel in target_channles {
        edit_channel_category(ctx, channel.id, archive_category.id).await?;
    }

    ctx.reply(format!("archived channels: {}", ids)).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn role(ctx: super::Context<'_>) -> anyhow::Result<()> {
    let guild_id = ctx.guild_id().ok_or_else(|| anyhow!("no guild"))?;

    let channel_name = match ctx.channel_id().to_channel(&ctx).await? {
        serenity::Channel::Guild(c) => c.name,
        _ => return Err(anyhow!("not a guild channel")),
    };

    let reply = ctx
        .reply(format!("preparing role {channel_name}..."))
        .await?;

    let client = ctx.serenity_context().http.clone();
    let reply = reply.into_message().await?;

    tokio::spawn(async move {
        let roles = client.get_guild_roles(guild_id).await?;
        let human_role_ids: Vec<_> = roles
            .iter()
            .filter_map(|r| {
                if r.name == "ひと" || r.name == "いちばんつよいひと" {
                    Some(r.id)
                } else {
                    None
                }
            })
            .collect();

        let mut humans: Vec<_> = client
            .get_guild_members(guild_id, None, None)
            .await?
            .into_iter()
            .filter(|m| {
                m.roles
                    .iter()
                    .any(|r| human_role_ids.iter().any(|id| r == id))
            })
            .collect();

        let new_role = {
            let mut param = Map::new();
            param.insert("name".to_owned(), json!(channel_name));
            param.insert("mentionable".to_owned(), json!(true));
            client.create_role(guild_id, &param, None).await?
        };

        for human in humans.iter_mut() {
            human.add_role(&client, new_role.id).await?;
        }

        reply
            .reply(&client, format!("role {} is created", &new_role.name))
            .await?;

        anyhow::Ok(())
    });

    Ok(())
}
