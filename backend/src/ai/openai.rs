use reqwest::{self};
use serde::{Deserialize, Serialize};

use super::AiApi;

struct OpenAi {
    model: String,
    token: String,
    endpoint: String,
}

pub enum OpenAiError {
    NoChoices,
    WrongFormat,
    RewestError(reqwest::Error),
}

impl From<reqwest::Error> for OpenAiError {
    fn from(error: reqwest::Error) -> Self {
        Self::RewestError(error)
    }
}

const OPENAI_SYSTEM_MESSAGE: &'static str = include_str!("system_message.txt");

impl AiApi for OpenAi {
    type AiError = OpenAiError;

    fn rate_limit(&self) -> u32 {
        todo!()
    }

    async fn request(&mut self, input: String) -> Result<String, Self::AiError> {
        let request = OpenAiRequest::new(&self.model)
            .append_message(Role::System, OPENAI_SYSTEM_MESSAGE)
            .append_message(Role::User, input);

        let response = reqwest::Client::new()
            .post(&self.endpoint)
            .bearer_auth(&self.token)
            .json(&request)
            .send()
            .await?
            .json::<OpenAiResponse>()
            .await?;

        if response.choices.is_empty() {
            return Err(OpenAiError::NoChoices);
        }

        let content = response.choices.get(0).unwrap().message.content.to_owned();

        if !content.starts_with("[TRUE]") || !content.starts_with("[FALSE]") {
            return Err(OpenAiError::WrongFormat);
        }

        Ok(content)
    }
}

impl OpenAiRequest {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            response_format: None,
            messages: vec![],
        }
    }

    pub fn append_message(mut self, role: Role, msg: impl Into<String>) -> Self {
        self.messages.push(Message {
            role,
            content: msg.into(),
        });
        self
    }

    pub fn response_format(mut self, fmt: Option<ResponseFormat>) -> Self {
        self.response_format = fmt;
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct OpenAiRequest {
    model: String,
    response_format: Option<ResponseFormat>,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum ResponseFormat {
    JsonObject,
    Text,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Role {
    System,
    User,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: Role,
    content: String,
}

#[derive(Deserialize, Debug)]
struct OpenAiResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
}
