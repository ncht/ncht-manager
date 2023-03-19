mod request;
mod response;

use std::{collections::VecDeque, sync::Arc};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serenity::{
    framework::standard::{macros::*, CommandError, CommandResult},
    model::channel::Message as DiscordMessage,
    prelude::*,
};

use request::Request;
use response::Response;
use tracing::{info, instrument};

use crate::config;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    System,
    Assistant,
}

#[derive(Debug, Default)]
pub struct Data {
    histories: VecDeque<Message>,
    histsize: usize,
}

impl Data {
    pub fn add_history(&mut self, history: Message) {
        while self.histories.len() >= self.histsize {
            self.histories.pop_front();
        }
        self.histories.push_back(history);
        info!("capacity: {}/{}", self.histories.len(), self.histsize);
    }

    pub fn histories(&self) -> Vec<Message> {
        let (s1, s2) = self.histories.as_slices();
        [s1, s2].concat()
    }

    pub fn histsize(&self) -> usize {
        self.histsize
    }

    pub fn set_histsize(&mut self, histsize: usize) {
        self.histsize = histsize;
    }
}

impl TypeMapKey for Data {
    type Value = Arc<RwLock<Data>>;
}

const ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";

#[instrument(skip(token, histories))]
pub async fn chat_api(
    histories: &[Message],
    role: Role,
    content: &str,
    token: &str,
) -> anyhow::Result<(Message, Message)> {
    let message = Message {
        role,
        content: content.to_owned(),
    };

    let body = Request {
        model: "gpt-3.5-turbo".to_owned(),
        messages: [histories, &[message.clone()]].concat(),
    };

    let client = reqwest::Client::new();
    let res: Response = client
        .post(ENDPOINT)
        .bearer_auth(token)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    Ok((message, res.choices[0].message.clone()))
}

pub async fn chat_with_role(
    ctx: &Context,
    msg: &DiscordMessage,
    prefix: &str,
    role: Role,
) -> CommandResult {
    let message = msg.content.strip_prefix(prefix).unwrap();
    let histories = {
        let data_read = ctx.data.read().await;
        let data_lock = data_read
            .get::<Data>()
            .ok_or_else(|| anyhow!("data is not initialized"))?;
        let data = data_lock.read().await;
        data.histories()
    };

    let (req, res) = chat_api(&histories, role, message, &config::config().chatgpt_token).await?;

    msg.reply(ctx, &res.content).await?;

    {
        let mut data_write = ctx.data.write().await;
        let data_lock = data_write
            .get_mut::<Data>()
            .ok_or_else(|| anyhow!("data is not initialized"))?;
        let mut data = data_lock.write().await;
        data.add_history(req);
        data.add_history(res);
    }

    Ok(())
}

#[command]
pub async fn chat(ctx: &Context, msg: &DiscordMessage) -> CommandResult {
    chat_with_role(ctx, msg, "!chat ", Role::User).await
}

#[command]
pub async fn chat_system(ctx: &Context, msg: &DiscordMessage) -> CommandResult {
    chat_with_role(ctx, msg, "!chat_system ", Role::System).await
}

#[command]
pub async fn histsize(ctx: &Context, msg: &DiscordMessage) -> CommandResult {
    let cmd: Vec<_> = msg.content.split_ascii_whitespace().collect();

    match cmd.len() {
        1 => {
            let histsize = {
                let data_read = ctx.data.read().await;
                let data_lock = data_read
                    .get::<Data>()
                    .ok_or_else(|| anyhow!("data is not initialized"))?;
                let data = data_lock.read().await;
                data.histsize()
            };

            msg.reply(ctx, format!("histsize: {histsize}")).await?;
            Ok(())
        }
        2 => {
            let histsize: usize = cmd[1].parse()?;
            let prev_histsize = {
                let mut data_write = ctx.data.write().await;
                let data_lock = data_write
                    .get_mut::<Data>()
                    .ok_or_else(|| anyhow!("data is not initialized"))?;
                let mut data = data_lock.write().await;
                let prev_histsize = data.histsize();
                data.set_histsize(histsize);
                prev_histsize
            };
            msg.reply(ctx, format!("histsize: {prev_histsize} -> {histsize}"))
                .await?;
            Ok(())
        }
        _ => Err(CommandError::from(anyhow!("invalid parameter"))),
    }
}
