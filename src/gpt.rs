use std::{cell::RefCell, fmt::Display};

use reqwest::{header::CONTENT_TYPE, Response};

pub struct GptClient {
    api_key: String,
    factory: RefCell<ChatFactory>,
}
impl GptClient {
    pub fn from_env() -> Result<Self> {
        match std::env::var("OPENAI_API_KEY") {
            Ok(api_key) => Ok(Self {
                api_key,
                factory: RefCell::new(ChatFactory::new()),
            }),
            Err(_) => Err(GptClientError {
                message: "Cause Error at GptClient::from_env".to_string(),
                kind: GptClientErrorKind::NotFoundEnvAPIKey,
            }),
        }
    }
    pub async fn chat(&self, message: impl Into<String>) -> Result<()> {
        let url = "https://api.openai.com/v1/chat/completions";
        let body = self.make_chat_body(message)?;
        println!("send request ...");
        println!("wait response ...");
        let response = self.post(url, body).await?;
        let res_message = self.response_text(response).await?;
        println!();
        println!("gpt > : {}", res_message);
        println!();
        Ok(())
    }
    async fn response_text(&self, response: Response) -> Result<String> {
        match response.text().await {
            Ok(response) => {
                let response = serde_json::from_str::<ChatResponse>(&response);
                match response {
                    Ok(response) => {
                        let Some(res_message) = response.last_response() else {
                            return Err(GptClientError {
                                message: "Cause Error at GptClient::response_text".to_string(),
                                kind: GptClientErrorKind::NotFoundResponseContent,
                            })
                        };
                        self.factory.borrow_mut().push_response(&res_message);
                        Ok(res_message)
                    }
                    Err(e) => Err(GptClientError {
                        message: "Cause Error at GptClient::response_text".to_string(),
                        kind: GptClientErrorKind::ResponseDeserializeError(e.to_string()),
                    }),
                }
            }
            Err(e) => Err(GptClientError {
                message: "Cause Error at GptClient::response_text".to_string(),
                kind: GptClientErrorKind::RequestError(e.to_string()),
            }),
        }
    }
    async fn post(&self, url: &'static str, body: String) -> Result<Response> {
        match reqwest::Client::new()
            .post(url)
            .body(body)
            .bearer_auth(self.api_key.as_str())
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
        {
            Ok(res) => Ok(res),
            Err(e) => Err(GptClientError {
                message: "Cause Error at GptClient::post".to_string(),
                kind: GptClientErrorKind::RequestError(e.to_string()),
            }),
        }
    }
    fn make_chat_body(&self, message: impl Into<String>) -> Result<String> {
        let message = message.into();
        self.factory.borrow_mut().push_request(&message);
        match serde_json::to_string(&self.factory.borrow().make_request()) {
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
#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}
impl Role {
    fn into_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}
impl serde::Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let role: &str = self.into_str();
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
        serializer.serialize_str(self.into_str())
    }
}

impl OpenAIModel {
    pub fn into_str(&self) -> &'static str {
        match self {
            Self::Gpt3Dot5Turbo => "gpt-3.5-turbo",
        }
    }
}
impl Into<&'static str> for OpenAIModel {
    fn into(self) -> &'static str {
        self.into_str()
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
    NotFoundResponseContent,
    RequestError(String),
    ResponseDeserializeError(String),
    NotMakeChatBody(String),
}
impl Display for GptClientErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self {
            Self::NotFoundEnvAPIKey => "Not found OPENAI_API_KEY in env".to_string(),
            Self::RequestError(s) => format!("Request Error to {}", s),
            Self::NotMakeChatBody(s) => format!("Not make chat body from {}", s),
            Self::NotFoundResponseContent => format!("Response Content is Not Found"),
            Self::ResponseDeserializeError(s) => {
                format!("Not Deserialize response. Serde Error is :  {}", s)
            }
        };
        write!(f, "{}", kind)
    }
}
impl std::error::Error for GptClientError {}
pub type Result<T> = std::result::Result<T, GptClientError>;
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ChatResponse {
    pub choices: Option<Vec<ChatResponseChoices>>,
    pub created: Option<usize>,
    pub id: Option<String>,
    pub object: Option<String>,
    pub usage: Option<ChatResponseUsage>,
}
impl ChatResponse {
    fn last_response(self) -> Option<String> {
        self.choices?.pop()?.message?.content
    }
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ChatResponseChoices {
    pub finish_reason: Option<String>,
    pub index: Option<usize>,
    pub message: Option<ChatResponseChoicesMessage>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ChatResponseChoicesMessage {
    pub content: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ChatResponseUsage {
    pub completion_tokens: Option<usize>,
    pub prompt_tokens: Option<usize>,
    pub total_tokens: Option<usize>,
}

#[derive(Debug)]
struct ChatFactory {
    stack: Vec<Message>,
}

impl ChatFactory {
    fn new() -> Self {
        Self { stack: Vec::new() }
    }

    fn make_request(&self) -> ChatRequest {
        ChatRequest {
            model: OpenAIModel::Gpt3Dot5Turbo,
            messages: self.stack.clone(),
        }
    }
    fn push_response(&mut self, message: impl Into<String>) {
        self.stack.push(Message {
            role: Role::Assistant,
            content: message.into(),
        });
    }
    fn push_request(&mut self, message: impl Into<String>) {
        self.stack.push(Message {
            role: Role::User,
            content: message.into(),
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chatの結果を保存してリクエストを作成する() {
        let mut sut = ChatFactory::new();
        sut.push_request("hello world");
        let request = sut.make_request();
        assert_eq!(request.messages[0].content, "hello world");
        assert_eq!(request.messages[0].role, Role::User);
        sut.push_response("hello world! have a nice day");
        sut.push_request("what?");
        let request = sut.make_request();
        assert_eq!(request.messages[0].content, "hello world");
        assert_eq!(request.messages[1].content, "hello world! have a nice day");
        assert_eq!(request.messages[1].role, Role::Assistant);
        assert_eq!(request.messages[2].content, "what?");
        assert_eq!(request.messages[2].role, Role::User);
    }
}
