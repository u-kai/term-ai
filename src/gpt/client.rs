use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use rsse::{
    client::{SseClient, SseClientBuilder},
    http::url::Url,
    sse::{
        connector::SseTlsConnector,
        response::SseResponse,
        subscriber::{HandleProgress, SseHandler, SseMutHandler, SseSubscribeError},
    },
};

pub struct GptSseHandler<R, T: StreamChatHandler<R>> {
    handler: T,
    _phantom: PhantomData<R>,
}
impl<R, T: StreamChatHandler<R>> GptSseHandler<R, T> {
    pub fn new(handler: T) -> Self {
        Self {
            handler,
            _phantom: PhantomData,
        }
    }
    pub fn handler(&self) -> &T {
        &self.handler
    }
}

impl From<HandleResult> for HandleProgress<GptClientError> {
    fn from(res: HandleResult) -> Self {
        match res {
            HandleResult::Progress => HandleProgress::Progress,
            HandleResult::Done => HandleProgress::Done,
            HandleResult::Err(e) => HandleProgress::Err(e),
        }
    }
}
impl<R, T: StreamChatHandler<R>> SseHandler<R, GptClientError> for GptSseHandler<R, T> {
    fn handle(&self, res: SseResponse) -> rsse::sse::subscriber::HandleProgress<GptClientError> {
        match ChatResponse::from_sse(res) {
            Ok(res) => HandleProgress::from(self.handler.handle(&res)),
            Err(e) => HandleProgress::Err(e),
        }
    }
    fn result(&self) -> Result<R> {
        Ok(self.handler.result())
    }
}

pub struct GptSseMutHandler<R, T: StreamChatMutHandler<R>> {
    handler: T,
    _phantom: PhantomData<R>,
}
impl<R, T: StreamChatMutHandler<R>> GptSseMutHandler<R, T> {
    pub fn new(handler: T) -> Self {
        Self {
            handler,
            _phantom: PhantomData,
        }
    }
    pub fn handler(&self) -> &T {
        &self.handler
    }
}
impl<R, T: StreamChatMutHandler<R>> SseMutHandler<R, GptClientError> for GptSseMutHandler<R, T> {
    fn handle(
        &mut self,
        res: SseResponse,
    ) -> rsse::sse::subscriber::HandleProgress<GptClientError> {
        match ChatResponse::from_sse(res) {
            Ok(res) => HandleProgress::from(self.handler.handle(&res)),
            Err(e) => HandleProgress::Err(e),
        }
    }
    fn result(&self) -> Result<R> {
        Ok(self.handler.result())
    }
}
pub trait StreamChatHandler<T> {
    fn handle(&self, res: &ChatResponse) -> HandleResult;
    fn result(&self) -> T;
}
pub trait StreamChatMutHandler<T> {
    fn handle(&mut self, res: &ChatResponse) -> HandleResult;
    fn result(&self) -> T;
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum HandleResult {
    Progress,
    Done,
    Err(GptClientError),
}
impl From<&ChatResponse> for HandleResult {
    fn from(res: &ChatResponse) -> Self {
        match res {
            ChatResponse::DeltaContent(_content) => HandleResult::Progress,
            ChatResponse::Done => HandleResult::Done,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default, Eq)]
pub struct GptClientOption {
    proxy: Option<String>,
    ca_path: Option<String>,
}

pub struct GptClient {
    key: OpenAIKey,
    sse_client: SseClient<SseTlsConnector>,
}
impl GptClient {
    const URL: &'static str = "https://api.openai.com/v1/chat/completions";
    pub fn new(key: OpenAIKey, option: GptClientOption) -> Result<Self> {
        let mut builder = Self::client_builder();
        if let Some(proxy) = option.proxy {
            builder = builder
                .proxy(&Url::from_str(&proxy).map_err(|_e| {
                    GptClientError::new(
                        "invalid proxy url".to_string(),
                        GptClientErrorKind::InvalidUrl(proxy.clone()),
                    )
                })?)
                .map_err(|_e| {
                    GptClientError::new(
                        "invalid proxy url".to_string(),
                        GptClientErrorKind::InvalidUrl(proxy),
                    )
                })?;
        }
        if let Some(ca) = option.ca_path {
            builder = builder.add_ca(&ca).map_err(|_e| {
                GptClientError::new(
                    "invalid ca".to_string(),
                    GptClientErrorKind::NotFoundCAFile(ca),
                )
            })?;
        }
        let sse_client = builder.build();
        Ok(Self { key, sse_client })
    }
    pub fn from_env() -> Result<Self> {
        let key = OpenAIKey::from_env()?;

        Self::new(
            key,
            GptClientOption {
                proxy: proxy_from_env(),
                ca_path: root_ca_from_env(),
            },
        )
    }
    pub fn request_mut_fn<F: FnMut(&ChatResponse) -> HandleResult>(
        &mut self,
        request: ChatRequest,
        mut f: F,
    ) -> Result<()> {
        self.send_before(request);
        self.sse_client
            .send_mut_fn(|sse_response| {
                let res = ChatResponse::from_sse(sse_response);
                match res {
                    Ok(res) => {
                        let result = f(&res);
                        HandleProgress::from(result)
                    }
                    Err(e) => HandleProgress::Err(e),
                }
            })
            .map_err(GptClientError::from)
    }
    pub fn request_mut<R, T: StreamChatMutHandler<R>>(
        &mut self,
        request: ChatRequest,
        handler: &mut GptSseMutHandler<R, T>,
    ) -> Result<R> {
        self.send_before(request);
        self.sse_client
            .send_mut(handler)
            .map_err(GptClientError::from)
    }
    pub fn request<R, T: StreamChatHandler<R>>(
        &mut self,
        request: ChatRequest,
        handler: &GptSseHandler<R, T>,
    ) -> Result<R> {
        self.send_before(request);
        self.sse_client.send(handler).map_err(GptClientError::from)
    }
    fn send_before(&mut self, req: ChatRequest) {
        self.sse_client.post().bearer_auth(self.key.key()).json(req);
    }
    fn client_builder() -> SseClientBuilder<SseTlsConnector> {
        SseClientBuilder::new(&Self::URL.try_into().unwrap())
    }
}

fn root_ca_from_env() -> Option<String> {
    match std::env::var("CA_BUNDLE") {
        Ok(ca) => Some(ca),
        Err(_) => match std::env::var("ca_bundle") {
            Ok(ca) => Some(ca),
            Err(_) => None,
        },
    }
}

fn proxy_from_env() -> Option<String> {
    match std::env::var("HTTPS_PROXY") {
        Ok(proxy) => Some(proxy),
        Err(_) => match std::env::var("https_proxy") {
            Ok(proxy) => Some(proxy),
            Err(_) => match std::env::var("HTTP_PROXY") {
                Ok(proxy) => Some(proxy),
                Err(_) => match std::env::var("http_proxy") {
                    Ok(proxy) => Some(proxy),
                    Err(_) => None,
                },
            },
        },
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatResponse {
    Done,
    DeltaContent(String),
}
impl ChatResponse {
    const GPT_DONE: &'static str = "[DONE]";
    fn from_sse(sse_res: SseResponse) -> Result<Self> {
        match sse_res {
            SseResponse::Data(data) => {
                if data.starts_with(Self::GPT_DONE) {
                    return Ok(Self::Done);
                };
                match serde_json::from_str::<StreamChat>(&data) {
                    Ok(chat) => Ok(Self::from(chat)),
                    Err(e) => Err(GptClientError {
                        message: format!("Failed to parse chat response: {}", e),
                        kind: GptClientErrorKind::ParseError(data),
                    }),
                }
            }
            _ => todo!(),
        }
    }
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done)
    }
    pub fn delta_content(&self) -> &str {
        match self {
            Self::DeltaContent(s) => s.as_str(),
            _ => "",
        }
    }
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct StreamChat {
    choices: Vec<StreamChatChoices>,
    created: usize,
    id: String,
    model: String,
    object: String,
}
impl StreamChat {
    fn last_response(mut self) -> Option<String> {
        self.choices.pop()?.delta.content
    }
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct StreamChatChoices {
    delta: StreamChatChoicesDelta,
    finish_reason: serde_json::Value,
    index: usize,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct StreamChatChoicesDelta {
    content: Option<String>,
}
impl From<StreamChat> for ChatResponse {
    fn from(s: StreamChat) -> Self {
        s.last_response()
            .map_or_else(|| Self::DeltaContent(String::new()), Self::DeltaContent)
    }
}
impl<T: Into<String>> From<T> for ChatResponse {
    fn from(s: T) -> Self {
        Self::DeltaContent(s.into())
    }
}
#[derive(Clone)]
pub struct OpenAIKey(String);

impl OpenAIKey {
    pub fn from_env() -> Result<Self> {
        Ok(Self(std::env::var("OPENAI_API_KEY").map_err(|_| {
            GptClientError::new(
                "OPENAI_API_KEY is not found".to_string(),
                GptClientErrorKind::NotFoundEnvAPIKey,
            )
        })?))
    }
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
    fn key(&self) -> &str {
        self.0.as_str()
    }
}
impl Debug for OpenAIKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "x".repeat(self.0.len()))
    }
}
impl Display for OpenAIKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "x".repeat(self.0.len()))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ChatRequest {
    model: OpenAIModel,
    messages: Vec<Message>,
    stream: bool,
}
impl ChatRequest {
    pub fn new(model: OpenAIModel, messages: Vec<Message>) -> Self {
        Self {
            model,
            messages,
            stream: true,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Message {
    role: Role,
    pub(crate) content: String,
}
impl Message {
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }
    pub fn change_content(&mut self) -> &mut String {
        &mut self.content
    }
    pub fn change_role(&mut self, role: Role) {
        self.role = role;
    }
}

#[derive(Debug, Clone, Copy, serde::Deserialize, PartialEq, Eq)]
pub enum Role {
    User,
    System,
    Assistant,
}
impl Role {
    fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::System => "system",
            Self::Assistant => "assistant",
        }
    }
}
impl serde::Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let role: &str = self.as_str();
        serializer.serialize_str(role)
    }
}
#[derive(Debug, Clone, Copy, serde::Deserialize, PartialEq, Eq)]
pub enum OpenAIModel {
    Gpt3Dot5Turbo,
    Gpt4,
    Gpt40314,
    Gpt4032k,
    Gpt4032k0314,
}
impl Default for OpenAIModel {
    fn default() -> Self {
        Self::Gpt3Dot5Turbo
    }
}
impl serde::Serialize for OpenAIModel {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl OpenAIModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gpt3Dot5Turbo => "gpt-3.5-turbo",
            Self::Gpt4 => "gpt-4",
            Self::Gpt40314 => "gpt-4-0314",
            Self::Gpt4032k => "gpt-4-032k",
            Self::Gpt4032k0314 => "gpt-4-032k-0314",
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GptClientError {
    message: String,
    pub kind: GptClientErrorKind,
}
impl GptClientError {
    pub fn new(message: String, kind: GptClientErrorKind) -> Self {
        Self { message, kind }
    }
}
impl From<SseSubscribeError<GptClientError>> for GptClientError {
    fn from(e: SseSubscribeError<GptClientError>) -> Self {
        match e {
            SseSubscribeError::HandlerError(e) => e,
            SseSubscribeError::InvalidUrl(s) => GptClientError {
                message: s.clone(),
                kind: GptClientErrorKind::InvalidUrl(s),
            },
            SseSubscribeError::HttpError(e) => GptClientError {
                message: e.to_string(),
                kind: GptClientErrorKind::RequestError(e.to_string()),
            },
            SseSubscribeError::ConnectionError(e) => GptClientError {
                message: e.to_string(),
                kind: GptClientErrorKind::RequestError(e.to_string()),
            },
        }
    }
}

impl Display for GptClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "kind : {}\n message : {}", self.kind, self.message)
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GptClientErrorKind {
    NotFoundCAFile(String),
    InvalidUrl(String),
    ParseError(String),
    NotFoundEnvAPIKey,
    ProxyConnectionError(String),
    ReadStreamError(String),
    RequestError(String),
    ResponseDeserializeError(String),
    NotMakeChatBody(String),
    ResponseError(String),
}
impl Display for GptClientErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self {
            Self::ProxyConnectionError(s) => format!("Proxy Connection Error. Error is : {}", s),
            Self::NotFoundCAFile(s) => format!("Not found CA File. File is : {}", s),
            Self::NotFoundEnvAPIKey => "Not found OPENAI_API_KEY in env".to_string(),
            Self::RequestError(s) => format!("Request Error to {}", s),
            Self::NotMakeChatBody(s) => format!("Not make chat body from {}", s),
            Self::ReadStreamError(s) => format!("Not Read Stream. Error is : {}", s),
            Self::ResponseDeserializeError(s) => {
                format!("Not Deserialize response. Serde Error is :  {}", s)
            }
            Self::ResponseError(s) => format!("Response Error. Error is : {}", s),
            Self::ParseError(s) => format!("Parse Error. Error is : {}", s),
            Self::InvalidUrl(s) => format!("Invalid Url. Url is : {}", s),
        };
        write!(f, "{}", kind)
    }
}
impl std::error::Error for GptClientError {}
pub type Result<T> = std::result::Result<T, GptClientError>;
#[cfg(test)]
mod tests {
    use rsse::sse::{response::SseResponse, subscriber::HandleProgress};

    use super::fakes::*;
    use super::*;

    #[test]
    #[ignore = "実際にproxy通信するので、CIでのテストは行わない"]
    fn proxyを利用してgpt通信を行うことが可能() {
        std::env::set_var("HTTPS_PROXY", "http://localhost:8080");
        let mut client = GptClient::from_env().unwrap();
        let handler = GptSseHandler::new(MockHandler::new());

        let result = client.request(
            ChatRequest::new(
                OpenAIModel::Gpt3Dot5Turbo,
                vec![Message::new(Role::User, "hello".to_string())],
            ),
            &handler,
        );

        std::env::remove_var("HTTPS_PROXY");
        assert!(result.as_ref().unwrap().len() > 0);
        assert!(handler.handler().called_time() > 0);
        for c in result.unwrap().chars() {
            println!("{}", c);
            assert!(c.is_ascii());
        }
    }
    #[test]
    #[ignore = "実際に通信するので、CIでのテストは行わない"]
    fn gptと実際の通信を行うことが可能() {
        let mut client = GptClient::from_env().unwrap();
        let handler = GptSseHandler::new(MockHandler::new());

        let result = client.request(
            ChatRequest::new(
                OpenAIModel::Gpt3Dot5Turbo,
                vec![Message::new(Role::User, "hello".to_string())],
            ),
            &handler,
        );

        assert!(result.as_ref().unwrap().len() > 0);
        assert!(handler.handler().called_time() > 0);
    }

    #[test]
    fn gpt_sse_handlerはgptからのレスポンス終了時に任意の値を返すことができる() {
        let handler = MockHandler::new();
        let handler = GptSseHandler::new(handler);
        handler.handle(SseResponse::Data(make_stream_chat_json("Hello World")));
        handler.handle(SseResponse::Data(make_stream_chat_json(" Good Bye")));

        let result = handler.result().unwrap();

        assert_eq!(result, "Hello World Good Bye");
    }
    #[test]
    fn gpt_sse_handlerはgptからのsseレスポンスを処理して内部の可変handlerに渡す() {
        let handler = MockMutHandler::new();
        let mut handler = GptSseMutHandler::new(handler);

        let progress = handler.handle(SseResponse::Data(make_stream_chat_json("Hello World")));

        matches!(progress, HandleProgress::Progress);
        assert_eq!(handler.handler().called_time(), 1);

        let done = handler.handle(SseResponse::Data("[DONE]".to_string()));
        matches!(done, HandleProgress::Done);
        assert_eq!(handler.handler().called_time(), 2);
    }
    #[test]
    fn gpt_sse_handlerはgptからのsseレスポンスを処理して内部のhandlerに渡す() {
        let handler = MockHandler::new();
        let handler = GptSseHandler::new(handler);

        let progress = handler.handle(SseResponse::Data(make_stream_chat_json("Hello World")));

        matches!(progress, HandleProgress::Progress);
        assert_eq!(handler.handler().called_time(), 1);

        let done = handler.handle(SseResponse::Data("[DONE]".to_string()));
        matches!(done, HandleProgress::Done);
        assert_eq!(handler.handler().called_time(), 2);
    }
    #[test]
    #[allow(non_snake_case)]
    fn gptのsseレスポンスをChatResponseに変換可能() {
        let response = SseResponse::Data(make_stream_chat_json("Hello World"));
        assert_eq!(
            ChatResponse::from_sse(response).unwrap().delta_content(),
            "Hello World"
        );
    }
    #[test]
    #[allow(non_snake_case)]
    fn gptのsseレスポンスをChatResponseに変換可能_done() {
        let response = SseResponse::Data("[DONE]".to_string());
        assert_eq!(
            ChatResponse::from_sse(response).unwrap(),
            ChatResponse::Done
        );
    }
    #[test]
    #[allow(non_snake_case)]
    fn gptのレスポンスはChatResponseに変換可能() {
        let response = ChatResponse::from(make_stream_chat("Hello World"));
        assert_eq!(response.delta_content(), "Hello World");
    }
}

#[cfg(test)]
pub mod fakes {
    use std::{cell::RefCell, io::Write};

    use super::*;
    pub struct MockMutHandler {
        called_time: usize,
        responses: Vec<String>,
    }
    impl MockMutHandler {
        pub fn new() -> Self {
            Self {
                called_time: 0,
                responses: Vec::new(),
            }
        }
        pub fn called_time(&self) -> usize {
            self.called_time
        }
        fn inc_called_time(&mut self) {
            self.called_time += 1;
        }
    }
    impl StreamChatMutHandler<String> for MockMutHandler {
        fn handle(&mut self, res: &ChatResponse) -> HandleResult {
            print!("{}", res.delta_content());
            std::io::stdout().flush().unwrap();
            self.inc_called_time();
            self.responses.push(res.delta_content().to_string());
            match res {
                ChatResponse::Done => HandleResult::Done,
                _ => HandleResult::Progress,
            }
        }
        fn result(&self) -> String {
            self.responses.join("")
        }
    }
    pub struct MockHandler {
        called_time: RefCell<usize>,
        responses: RefCell<Vec<String>>,
    }
    impl MockHandler {
        pub fn new() -> Self {
            Self {
                called_time: RefCell::new(0),
                responses: RefCell::new(Vec::new()),
            }
        }
        pub fn called_time(&self) -> usize {
            *self.called_time.borrow()
        }
        fn inc_called_time(&self) {
            *self.called_time.borrow_mut() += 1;
        }
    }

    impl StreamChatHandler<String> for MockHandler {
        fn handle(&self, res: &ChatResponse) -> HandleResult {
            print!("{}", res.delta_content());
            std::io::stdout().flush().unwrap();
            self.inc_called_time();
            self.responses
                .borrow_mut()
                .push(res.delta_content().to_string());
            match res {
                ChatResponse::Done => HandleResult::Done,
                _ => HandleResult::Progress,
            }
        }
        fn result(&self) -> String {
            self.responses.borrow().join("")
        }
    }
    pub fn make_stream_chat_json(message: &str) -> String {
        format!(
            r#"
            {{
              "id": "chatcmpl-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
              "object": "chat.completion.chunk",
              "created": 1694832938,
              "model": "gpt-3.5-turbo-0613",
              "choices": [
                {{ "index": 0, "delta": {{ "content": "{}" }}, "finish_reason": null }}
              ]
            }}"#,
            message
        )
    }
    pub(crate) fn make_stream_chat(message: &str) -> StreamChat {
        serde_json::from_str(&make_stream_chat_json(message)).unwrap()
    }
}
