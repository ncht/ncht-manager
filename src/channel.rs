use crate::config;
use anyhow::anyhow;
use chrono::{Duration, Utc};
use itertools::Itertools;
use serde_json::{json, Map};
use serenity::{
    framework::standard::{macros::*, CommandError, CommandResult},
    model::{
        channel::{Channel, ChannelType, GuildChannel, Message},
        id::ChannelId,
    },
    prelude::*,
};

fn find_category<'a>(
    channels: &'a [GuildChannel],
    name: &str,
) -> Result<&'a GuildChannel, CommandError> {
    channels
        .iter()
        .find(|c| c.kind == ChannelType::Category && c.name == name)
        .ok_or_else(|| anyhow!(format!("`{}` category not found", name)).into())
}

async fn edit_channel_category(
    ctx: &Context,
    channel_id: ChannelId,
    category_id: ChannelId,
) -> Result<(), CommandError> {
    let mut param = Map::new();
    param.insert("parent_id".to_owned(), json!(category_id.as_u64()));
    ctx.http.edit_channel(channel_id.into(), &param).await?;

    Ok(())
}

#[command]
pub async fn archive(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| CommandError::from(anyhow!("no guild")))?;

    let channels = ctx.http.get_channels(guild_id.into()).await?;

    let active_category = find_category(&channels, &config::ACTIVE_CATEGORY)?;
    let archive_category = find_category(&channels, &config::ARCHIVE_CATEGORY)?;

    let target_channles: Vec<_> = channels
        .iter()
        .filter(|channel| {
            if channel.kind != ChannelType::Text {
                return false;
            }

            if channel.category_id != Some(active_category.id) {
                return false;
            }

            channel
                .last_message_id
                .map(|id| {
                    let threshold = Utc::now() - Duration::days(*config::THRESHOLD_DAYS);
                    let target = id.created_at().with_timezone(&Utc);

                    threshold > target
                })
                .unwrap_or(false)
        })
        .collect();

    if target_channles.is_empty() {
        msg.reply(&ctx, "no target".to_owned()).await?;

        return Ok(());
    }

    let ids = target_channles
        .iter()
        .map(|c| format!("<#{}>", c.id))
        .join(", ");

    for channel in target_channles {
        edit_channel_category(ctx, channel.id, archive_category.id).await?;
    }

    msg.reply(&ctx, format!("archived channels: {}", ids))
        .await?;

    Ok(())
}

#[command]
pub async fn restore(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| CommandError::from(anyhow!("no guild")))?;

    let channels = ctx.http.get_channels(guild_id.into()).await?;
    let active_category = find_category(&channels, &config::ACTIVE_CATEGORY)?;
    let archive_category = find_category(&channels, &config::ARCHIVE_CATEGORY)?;

    let channel = channels
        .iter()
        .find(|c| c.id == msg.channel_id && c.category_id == Some(archive_category.id));

    if let Some(channel) = channel {
        edit_channel_category(ctx, channel.id, active_category.id).await?;
        msg.reply(&ctx, format!("restored",)).await?;
    } else {
        msg.reply(&ctx, "not archived channel").await?;
    }

    Ok(())
}

#[command]
pub async fn role(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| CommandError::from(anyhow!("no guild")))?;

    let roles = ctx.http.get_guild_roles(guild_id.into()).await?;
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

    let mut humans: Vec<_> = ctx
        .http
        .get_guild_members(guild_id.into(), None, None)
        .await?
        .into_iter()
        .filter(|m| {
            m.roles
                .iter()
                .any(|r| human_role_ids.iter().any(|id| r == id))
        })
        .collect();

    let channel_name = match ctx.http.get_channel(msg.channel_id.into()).await? {
        Channel::Guild(c) => c.name,
        _ => return Err(CommandError::from(anyhow!("guild channel is expected"))),
    };

    let new_role = {
        let mut param = Map::new();
        param.insert("name".to_owned(), json!(channel_name));
        param.insert("mentionable".to_owned(), json!(true));
        ctx.http.create_role(guild_id.into(), &param).await?
    };

    for human in humans.iter_mut() {
        human.add_role(ctx, new_role.id).await?;
    }

    msg.reply(&ctx, format!("create role: {}", channel_name))
        .await?;

    Ok(())
}
