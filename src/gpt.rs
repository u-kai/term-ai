use std::{f32::consts::E, fmt::Display};

use reqwest::{header::CONTENT_TYPE, Response};

pub struct GptClient {
    api_key: String,
}
impl GptClient {
    pub fn from_env() -> Result<Self> {
        match std::env::var("OPENAI_API_KEY") {
            Ok(api_key) => Ok(Self { api_key }),
            Err(_) => Err(GptClientError {
                message: "Cause Error at GptClient::from_env".to_string(),
                kind: GptClientErrorKind::NotFoundEnvAPIKey,
            }),
        }
    }
    pub async fn chat(&self, message: impl Into<String>) -> Result<()> {
        let url = "https://api.openai.com/v1/chat/completions";
        let body = Self::make_chat_body(message)?;
        let response = self.post(url, body).await?;
        let mut text = response.text().await.unwrap();
        println!("{}", text);
        Ok(())
    }
    async fn post(&self, url: &'static str, body: String) -> Result<Response> {
        match self.make_post_request(url, body).send().await {
            Ok(res) => Ok(res),
            Err(e) => Err(GptClientError {
                message: "Cause Error at GptClient::post".to_string(),
                kind: GptClientErrorKind::RequestError(e.to_string()),
            }),
        }
    }
    fn make_post_request(&self, url: &'static str, body: String) -> reqwest::RequestBuilder {
        reqwest::Client::new()
            .post(url)
            .body(body)
            .bearer_auth(self.api_key.as_str())
            .header(CONTENT_TYPE, "application/json")
    }
    fn make_chat_body(message: impl Into<String>) -> Result<String> {
        let message = message.into();
        match serde_json::to_string(&ChatRequest {
            model: OpenAIModel::Gpt3Dot5Turbo,
            messages: vec![Message {
                role: Role::User,
                content: message,
            }],
        }) {
            Err(e) => Err(GptClientError {
                message: "Cause generate api json body".to_string(),
                kind: GptClientErrorKind::NotMakeChatBody(e.to_string()),
            }),
            Ok(body) => Ok(body),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ChatRequest {
    model: OpenAIModel,
    messages: Vec<Message>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    role: Role,
    content: String,
}
#[derive(Debug, Clone, serde::Deserialize)]
pub enum Role {
    User,
}
impl Into<&'static str> for Role {
    fn into(self) -> &'static str {
        match self {
            Self::User => "user",
        }
    }
}
impl serde::Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let role = match self {
            Self::User => "user",
        };
        serializer.serialize_str(role)
    }
}
#[derive(Debug, Clone, serde::Deserialize)]
pub enum OpenAIModel {
    Gpt3Dot5Turbo,
}
impl serde::Serialize for OpenAIModel {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let model = match self {
            Self::Gpt3Dot5Turbo => "gpt-3.5-turbo",
        };
        serializer.serialize_str(model)
    }
}

impl Into<&'static str> for OpenAIModel {
    fn into(self) -> &'static str {
        match self {
            Self::Gpt3Dot5Turbo => "gpt-3.5-turbo",
        }
    }
}
#[derive(Debug)]
pub struct GptClientError {
    message: String,
    kind: GptClientErrorKind,
}

impl Display for GptClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "kind : {}\n message : {}", self.kind, self.message)
    }
}
#[derive(Debug)]
pub enum GptClientErrorKind {
    NotFoundEnvAPIKey,
    RequestError(String),
    NotMakeChatBody(String),
}
impl Display for GptClientErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self {
            Self::NotFoundEnvAPIKey => "Not found OPENAI_API_KEY in env".to_string(),
            Self::RequestError(s) => format!("Request Error to {}", s),
            Self::NotMakeChatBody(s) => format!("Not make chat body from {}", s),
        };
        write!(f, "{}", kind)
    }
}
impl std::error::Error for GptClientError {}
pub type Result<T> = std::result::Result<T, GptClientError>;
