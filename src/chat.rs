mod request;
mod response;

use serde::{Deserialize, Serialize};
use serenity::{
    framework::standard::{macros::*, CommandResult},
    model::channel::Message as DiscordMessage,
    prelude::*,
};

use request::Request;
use response::Response;

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

const ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";

pub async fn chat_api(content: &str, token: &str) -> anyhow::Result<String> {
    let body = Request {
        model: "gpt-3.5-turbo".to_owned(),
        messages: vec![Message {
            role: Role::User,
            content: content.to_owned(),
        }],
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

    Ok(res.choices[0].message.content.clone())
}

#[command]
pub async fn chat(ctx: &Context, msg: &DiscordMessage) -> CommandResult {
    let res = chat_api(&msg.content, &config::config().chatgpt_token).await?;

    dbg!(&msg.content, &res);

    msg.reply(ctx, res).await?;

    Ok(())
}
