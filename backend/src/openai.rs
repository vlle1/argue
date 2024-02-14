use reqwest::{self, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}
#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
}

pub async fn request(system_message: String, user_message: String) -> Result<String, reqwest::Error> {
    let messages: Vec<Message> = vec![
        Message {
            role: "system".to_owned(),
            content: system_message,
        },
        Message {
            role: "user".to_owned(),
            content: user_message,
        },
    ];
    let dummyrequest = OpenAIRequest {
        model: "gpt-3.5-turbo".to_owned(),
        messages: messages,
    };

    // post request:
    let key = include_str!("../../openai.key").trim();
    let resp = reqwest::Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", key))
        .json(&dummyrequest)
        .send()
        .await;

    match resp.and_then(Response::error_for_status) {
        Ok(resp) => Ok(resp
            .json::<OpenAIResponse>()
            .await
            .unwrap()
            .choices
            .into_iter()
            .next()
            .unwrap()
            .message
            .content),
        Err(e) => Err(e),
    }
}
