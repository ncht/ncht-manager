use crate::config;
use chrono::{Duration, Utc};
use itertools::Itertools;
use serde_json::{json, Map};
use serenity::{
    framework::standard::{macros::*, CommandError, CommandResult},
    model::{
        channel::{ChannelType, GuildChannel, Message},
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
        .ok_or_else(|| CommandError(format!("`{}` category not found", name)))
}

fn edit_channel_category(
    ctx: &Context,
    channel_id: ChannelId,
    category_id: ChannelId,
) -> Result<(), CommandError> {
    let mut param = Map::new();
    param.insert("parent_id".to_owned(), json!(category_id.as_u64()));
    ctx.http.edit_channel(channel_id.into(), &param)?;

    Ok(())
}

#[command]
pub fn archive(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| CommandError("no guild".to_owned()))?;

    let channels = ctx.http.get_channels(guild_id.into())?;

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
        msg.reply(&ctx, "no target".to_owned())?;

        return Ok(());
    }

    let ids = target_channles
        .iter()
        .map(|c| format!("<#{}>", c.id))
        .join(", ");

    for channel in target_channles {
        edit_channel_category(ctx, channel.id, archive_category.id)?;
    }

    msg.reply(&ctx, format!("archived channels: {}", ids))?;

    Ok(())
}

#[command]
pub fn restore(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or_else(|| CommandError("no guild".to_owned()))?;

    let channels = ctx.http.get_channels(guild_id.into())?;
    let active_category = find_category(&channels, &config::ACTIVE_CATEGORY)?;
    let archive_category = find_category(&channels, &config::ARCHIVE_CATEGORY)?;

    let channel = channels
        .iter()
        .find(|c| c.id == msg.channel_id && c.category_id == Some(archive_category.id));

    if let Some(channel) = channel {
        edit_channel_category(ctx, channel.id, active_category.id)?;
        msg.reply(&ctx, format!("restored",))?;
    } else {
        msg.reply(&ctx, "not archived channel")?;
    }

    Ok(())
}
