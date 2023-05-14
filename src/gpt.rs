use std::{cell::RefCell, fmt::Display, io::BufRead};

use rsse::{client::SseClient, request_builder::RequestBuilder};

pub struct GptClient {
    api_key: String,
    sse_client: SseClient,
    factory: RefCell<ChatFactory>,
}
impl GptClient {
    const URL: &'static str = "https://api.openai.com/v1/chat/completions";
    pub fn from_env() -> Result<Self> {
        match std::env::var("OPENAI_API_KEY") {
            Ok(api_key) => Ok(Self {
                api_key,
                sse_client: SseClient::default(Self::URL).unwrap(),
                factory: RefCell::new(ChatFactory::new()),
            }),
            Err(_) => Err(GptClientError {
                message: "Cause Error at GptClient::from_env".to_string(),
                kind: GptClientErrorKind::NotFoundEnvAPIKey,
            }),
        }
    }
    pub fn stream_chat(
        &mut self,
        message: impl Into<String>,
        handler: impl Fn(String) -> Result<()>,
    ) -> Result<()> {
        let json = self.make_chat_body(message)?;
        let request = RequestBuilder::new(Self::URL)
            .post()
            .bearer_auth(self.api_key.as_str())
            .json(json)
            .build();
        let mut reader = self
            .sse_client
            .stream_reader(request)
            .map_err(|e| GptClientError {
                kind: GptClientErrorKind::ReadStreamError(e.to_string()),
                message: "Cause Error at GptClient::stream_chat".to_string(),
            })?;
        let mut line = String::new();
        let mut read_len = 1;
        let mut err_num = 0;
        while read_len > 0 {
            match reader.read_line(&mut line) {
                Ok(len) => read_len = len,
                Err(e) => {
                    err_num += 1;
                    println!("network error");
                    if err_num > 3 {
                        return Err(GptClientError {
                            kind: GptClientErrorKind::ReadStreamError(e.to_string()),
                            message: "Cause Error at GptClient::stream_chat".to_string(),
                        });
                    }
                    println!("retrying...");
                    continue;
                }
            }
            if line.starts_with("data:") {
                let data = line.trim_start_matches("data:").trim();
                let chat: serde_json::Result<StreamChat> = serde_json::from_str(data);
                match chat {
                    Ok(chat) => {
                        if let Some(content) = chat.last_response() {
                            self.factory.borrow_mut().push_response(content.as_str());
                            match handler(content) {
                                Ok(_) => (),
                                Err(e) => return Err(e),
                            }
                        }
                    }
                    Err(e) => {
                        if self.is_end_answer(data) {
                            return Ok(());
                        }
                        return Err(GptClientError {
                            kind: GptClientErrorKind::ResponseDeserializeError(e.to_string()),
                            message: "Cause Error at GptClient::stream_chat".to_string(),
                        });
                    }
                }
            }
            line.clear();
        }
        Ok(())
    }
    fn is_end_answer(&self, data: &str) -> bool {
        data == "[DONE]"
    }
    fn make_chat_body(&self, message: impl Into<String>) -> Result<ChatRequest> {
        let message = message.into();
        let message = message.trim().to_string();
        self.factory.borrow_mut().push_request(&message);
        Ok(self.factory.borrow().make_request())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ChatRequest {
    model: OpenAIModel,
    messages: Vec<Message>,
    stream: bool,
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
    ReadStreamError(String),
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
            Self::ReadStreamError(s) => format!("Not Read Stream. Error is : {}", s),
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
pub struct StreamChat {
    pub choices: Vec<StreamChatChoices>,
    pub created: usize,
    pub id: String,
    pub model: String,
    pub object: String,
}
impl StreamChat {
    pub fn last_response(mut self) -> Option<String> {
        self.choices.pop()?.delta.content
    }
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct StreamChatChoices {
    pub delta: StreamChatChoicesDelta,
    pub finish_reason: serde_json::Value,
    pub index: usize,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct StreamChatChoicesDelta {
    pub content: Option<String>,
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
            stream: true,
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
