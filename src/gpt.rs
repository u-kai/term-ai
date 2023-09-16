use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    marker::PhantomData,
};

use rsse::{
    client::{SseClient, SseClientBuilder},
    sse::{
        connector::SseTlsConnector,
        response::SseResponse,
        subscriber::{HandleProgress, SseHandler},
    },
};

#[derive(Debug, Clone)]
pub struct GptClient {
    proxy_url: Option<String>,
    api_key: OpenAIKey,
    history: ChatHistory,
}

impl GptClient {
    const URL: &'static str = "https://api.openai.com/v1/chat/completions";
    pub fn from_env() -> Result<Self> {
        match std::env::var("OPENAI_API_KEY") {
            Ok(api_key) => Ok(Self {
                proxy_url: proxy_from_env(),
                api_key: OpenAIKey::new(api_key),
                history: ChatHistory::new(),
            }),
            Err(_) => Err(GptClientError {
                message: "Cause Error at GptClient::from_env".to_string(),
                kind: GptClientErrorKind::NotFoundEnvAPIKey,
            }),
        }
    }
    pub fn set_proxy(&mut self, proxy_url: impl Into<String>) {
        self.proxy_url = Some(proxy_url.into());
    }
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
    //pub fn chat<F: Fn(&str) -> ()>(
    //    &mut self,
    //    model: OpenAIModel,
    //    role: Role,
    //    message: impl Into<String>,
    //    f: &F,
    //) -> Result<String> {
    //    let message: String = message.into();
    //    self.history.push_request(message.clone(), role);
    //    let request = self.make_stream_request(model);
    //    let client = match &self.proxy_url {
    //        Some(proxy_url) => Self::client_with_proxy(f, proxy_url.as_str()),
    //        None => Self::client(f),
    //    };
    //    let result = client
    //        .bearer_auth(self.api_key.key())
    //        .post()
    //        .json(&request)
    //        .handle_event()
    //        .map_err(|e| GptClientError {
    //            message: "Cause Error at GptClient::chat".to_string(),
    //            kind: GptClientErrorKind::RequestError(e.to_string()),
    //        })?;
    //    //match result {
    //    //    Connecte => Ok("".to_string()),
    //    //    SseResult::Retry => self.chat(model, Role::User, message, f),
    //    //    SseResult::Finished(c) => {
    //    //        self.history.push_response(c);
    //    //        Ok(self.history.last_response().unwrap().to_string())
    //    //    }
    //    //}
    //}
    fn make_stream_request(&mut self, model: OpenAIModel) -> ChatRequest {
        let messages = self.history.all();
        ChatRequest {
            model,
            messages,
            stream: true,
        }
    }
    //fn client_with_proxy<F: Fn(&str) -> ()>(
    //    f: F,
    //    proxy_url: impl Into<String>,
    //) -> SseClient<ChatHandler<F>, ChatErrorHandler, ChatResponse> {
    //    SseClient::new(Self::URL, ChatHandler::new(f), ChatErrorHandler::new())
    //        .unwrap()
    //        .set_proxy_url(proxy_url.into().as_str())
    //}
    //    fn client<F: Fn(&str) -> ()>(f: F) -> SseClient<SseTlsConnector> {
    //        SseClientBuilder::new(Self::URL).post().json(&request).build()
    //    }
}
//#[derive(Debug)]
//pub struct ChatHandler<F: Fn(&str) -> ()> {
//    f: F,
//    stream: RefCell<ChatStream>,
//}
//impl<F: Fn(&str) -> ()> ChatHandler<F> {
//    const GPT_DONE: &'static str = "[DONE]";
//    pub fn new(f: F) -> Self {
//        Self {
//            f,
//            stream: RefCell::new(ChatStream::new()),
//        }
//    }
//}
//impl<F: Fn(&str) -> ()> EventHandler<ChatResponse> for ChatHandler<F> {
//    type Err = GptClientError;
//    fn finished(&self) -> std::result::Result<SseResult<ChatResponse>, Self::Err> {
//        Ok(SseResult::Finished(self.stream.borrow().gen_response()))
//    }
//    fn handle(&self, event: &str) -> std::result::Result<SseResult<ChatResponse>, Self::Err> {
//        let chat: serde_json::Result<StreamChat> = serde_json::from_str(event);
//        match chat {
//            Ok(chat) => {
//                let Some(response ) =  chat.last_response() else {
//                    return Ok(SseResult::Continue);
//                };
//                (self.f)(response.as_str());
//                self.stream.borrow_mut().join_response(response.as_str());
//                Ok(SseResult::Continue)
//            }
//            Err(e) => {
//                if event == Self::GPT_DONE {
//                    return self.finished();
//                }
//                return Err(GptClientError {
//                    message: e.to_string(),
//                    kind: GptClientErrorKind::ResponseDeserializeError(event.to_string()),
//                });
//            }
//        }
//    }
//}

#[derive(Debug)]
struct ChatErrorHandler {
    err_counter: RefCell<usize>,
}

//impl ErrorHandler<ChatResponse> for ChatErrorHandler {
//    type Err = GptClientError;
//    fn catch(
//        &self,
//        error: rsse::SseHandlerError,
//    ) -> std::result::Result<SseResult<ChatResponse>, Self::Err> {
//        let mut err_counter = self.err_counter.borrow_mut();
//        *err_counter += 1;
//        match error {
//            rsse::SseHandlerError::HttpResponseError {
//                message,
//                read_line,
//                response,
//            } => {
//                if *err_counter > 3 {
//                    return Err(GptClientError {
//                        message: "Cause Error at ChatErrorHandler::catch".to_string(),
//                        kind: GptClientErrorKind::RequestError(read_line),
//                    });
//                }
//                return Err(GptClientError {
//                    message: message,
//                    kind: GptClientErrorKind::RequestError(response.status_text().to_string()),
//                });
//            }
//            _ => {
//                if *err_counter > 3 {
//                    return Err(GptClientError {
//                        message: "Cause Error at ChatErrorHandler::catch".to_string(),
//                        kind: GptClientErrorKind::RequestError(error.to_string()),
//                    });
//                }
//                return Ok(SseResult::Retry);
//            }
//        }
//    }
//}
//impl ChatErrorHandler {
//    fn new() -> Self {
//        Self {
//            err_counter: RefCell::new(0),
//        }
//    }
//}

#[derive(Debug)]
pub struct GptClientError {
    message: String,
    kind: GptClientErrorKind,
}
impl GptClientError {
    pub fn new(message: String, kind: GptClientErrorKind) -> Self {
        Self { message, kind }
    }
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
    ResponseError(String),
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
            Self::ResponseError(s) => format!("Response Error. Error is : {}", s),
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

#[derive(Debug, Clone)]
struct ChatHistory {
    inner: Vec<Message>,
}
impl ChatHistory {
    fn new() -> Self {
        Self { inner: Vec::new() }
    }
    fn all(&self) -> Vec<Message> {
        self.inner.clone()
    }
    fn clear(&mut self) {
        self.inner.clear();
    }
    fn last_response(&self) -> Option<&str> {
        self.inner.last().map(|m| m.content.as_str())
    }
    fn push_response(&mut self, message: ChatResponse) {
        self.inner.push(Message {
            role: Role::Assistant,
            content: message.delta_content().to_string(),
        });
    }
    fn push_request(&mut self, message: impl Into<String>, role: Role) {
        self.inner.push(Message {
            role,
            content: message.into(),
        });
    }
}

#[derive(Debug)]
struct ChatStream(String);
impl ChatStream {
    fn new() -> Self {
        Self(String::new())
    }
    fn gen_response(&self) -> ChatResponse {
        ChatResponse::DeltaContent(self.0.clone())
    }
    fn join_response(&mut self, message: &str) {
        self.0.push_str(message);
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum ChatResponse {
    Done,
    DeltaContent(String),
}
impl ChatResponse {
    const GPT_DONE: &'static str = "[DONE]";
    fn from_sse(sse_res: SseResponse) -> std::result::Result<Self, String> {
        match sse_res {
            SseResponse::Data(data) => {
                if data.starts_with(Self::GPT_DONE) {
                    return Ok(Self::Done);
                };
                match serde_json::from_str::<StreamChat>(&data) {
                    Ok(chat) => Ok(Self::from(chat)),
                    Err(e) => todo!(),
                }
            }
            _ => todo!(),
        }
    }
    fn delta_content(&self) -> &str {
        match self {
            Self::DeltaContent(s) => s.as_str(),
            _ => "",
        }
    }
}
impl From<StreamChat> for ChatResponse {
    fn from(s: StreamChat) -> Self {
        s.last_response().map_or_else(
            || Self::DeltaContent(String::new()),
            |s| Self::DeltaContent(s.to_string()),
        )
    }
}
impl<T: Into<String>> From<T> for ChatResponse {
    fn from(s: T) -> Self {
        Self::DeltaContent(s.into())
    }
}
#[derive(Clone)]
struct OpenAIKey(String);

impl OpenAIKey {
    fn from_env() -> Result<Self> {
        Ok(Self(std::env::var("OPENAI_API_KEY").map_err(|_| {
            GptClientError::new(
                "OPENAI_API_KEY is not found".to_string(),
                GptClientErrorKind::NotFoundEnvAPIKey,
            )
        })?))
    }
    fn new(key: impl Into<String>) -> Self {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ChatRequest {
    model: OpenAIModel,
    messages: Vec<Message>,
    stream: bool,
}
impl ChatRequest {
    fn user_gpt3(message: &str) -> Self {
        Self {
            model: OpenAIModel::Gpt3Dot5Turbo,
            messages: vec![Message {
                role: Role::User,
                content: message.to_string(),
            }],
            stream: true,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Message {
    role: Role,
    content: String,
}
#[derive(Debug, Clone, Copy, serde::Deserialize, PartialEq, Eq)]
pub enum Role {
    User,
    System,
    Assistant,
}
impl Role {
    fn into_str(&self) -> &'static str {
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
        let role: &str = self.into_str();
        serializer.serialize_str(role)
    }
}
#[derive(Debug, Clone, Copy, serde::Deserialize)]
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
        serializer.serialize_str(self.into_str())
    }
}

impl OpenAIModel {
    pub fn into_str(&self) -> &'static str {
        match self {
            Self::Gpt3Dot5Turbo => "gpt-3.5-turbo",
            Self::Gpt4 => "gpt-4",
            Self::Gpt40314 => "gpt-4-0314",
            Self::Gpt4032k => "gpt-4-032k",
            Self::Gpt4032k0314 => "gpt-4-032k-0314",
        }
    }
}
impl Into<&'static str> for OpenAIModel {
    fn into(self) -> &'static str {
        self.into_str()
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

//pub struct ChatGptClient<T: ChatGpt> {
//    gpt: T,
//}
//
//impl<T: ChatGpt> ChatGptClient<T> {
//    pub fn new(gpt: T) -> Self {
//        Self { gpt }
//    }
//    pub fn chat<H: StreamChatHandler>(
//        &mut self,
//        message: impl Into<String>,
//        handler: &ChatGptSseHandler<H>,
//    ) {
//        self.gpt.chat(message.into(), handler);
//    }
//}
pub struct ChatGptSseHandler<R, T: StreamChatHandler<R>> {
    handler: T,
    _phantom: PhantomData<R>,
}
impl<R, T: StreamChatHandler<R>> ChatGptSseHandler<R, T> {
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
impl<R, T: StreamChatHandler<R>> SseHandler<R, ()> for ChatGptSseHandler<R, T> {
    fn handle(&self, res: SseResponse) -> rsse::sse::subscriber::HandleProgress<()> {
        let res = ChatResponse::from_sse(res).unwrap();
        match self.handler.handle(&res) {
            HandleResult::Progress => HandleProgress::Progress,
            HandleResult::Done => HandleProgress::Done,
        }
    }
    fn result(&self) -> std::result::Result<R, ()> {
        Ok(self.handler.result())
    }
}
pub trait ChatGpt {
    fn chat<R, T: StreamChatHandler<R>>(
        &mut self,
        message: String,
        handler: &ChatGptSseHandler<R, T>,
    );
}
pub trait StreamChatHandler<T> {
    fn handle(&self, res: &ChatResponse) -> HandleResult;
    fn result(&self) -> T;
}
pub enum HandleResult {
    Progress,
    Done,
}

pub struct ChatGptClient {
    key: OpenAIKey,
    sse_client: SseClient<SseTlsConnector>,
}
impl ChatGptClient {
    const URL: &'static str = "https://api.openai.com/v1/chat/completions";
    pub fn from_env() -> Result<Self> {
        let key = OpenAIKey::from_env()?;
        let sse_client = Self::client();
        Ok(Self { key, sse_client })
    }
    pub fn chat<T: StreamChatHandler<String>>(
        &mut self,
        message: &str,
        handler: &ChatGptSseHandler<String, T>,
    ) -> Result<String> {
        self.sse_client
            .post()
            .bearer_auth(self.key.key())
            .json(ChatRequest::user_gpt3(message));
        let result = self.sse_client.send(handler);
        Ok(result.unwrap())
    }
    fn client() -> SseClient<SseTlsConnector> {
        SseClientBuilder::new(&Self::URL.try_into().unwrap()).build()
    }
}

#[cfg(test)]
mod tests {
    use rsse::sse::{response::SseResponse, subscriber::HandleProgress};

    use crate::gpt::fakes::{make_stream_chat, make_stream_chat_json};

    use super::{fakes::MockHandler, *};

    #[test]
    #[ignore = "実際に通信するので、CIでのテストは行わない"]
    fn chat_gptと実際の通信を行うことが可能() {
        let mut client = ChatGptClient::from_env().unwrap();
        let handler = ChatGptSseHandler::new(MockHandler::new());

        let result = client.chat("日本語で絶対返事してね!", &handler);

        assert!(result.as_ref().unwrap().len() > 0);
        assert!(handler.handler().called_time() > 0);
        for c in result.unwrap().chars() {
            println!("{}", c);
            assert!(!c.is_ascii());
        }

        let result = client.chat("Hello World", &handler);

        assert!(result.unwrap().len() > 0);
        assert!(handler.handler().called_time() > 0);
    }

    #[test]
    fn chat_gpt_sse_handlerはchat_gptからのレスポンス終了時に任意の値を返すことができる() {
        let handler = MockHandler::new();
        let handler = ChatGptSseHandler::new(handler);
        handler.handle(SseResponse::Data(make_stream_chat_json("Hello World")));
        handler.handle(SseResponse::Data(make_stream_chat_json(" Good Bye")));

        let result = handler.result().unwrap();

        assert_eq!(result, "Hello World Good Bye");
    }
    #[test]
    fn chat_gpt_sse_handlerはchat_gptからのsseレスポンスを処理して内部のhandlerに渡す() {
        let handler = MockHandler::new();
        let handler = ChatGptSseHandler::new(handler);

        let progress = handler.handle(SseResponse::Data(make_stream_chat_json("Hello World")));

        matches!(progress, HandleProgress::Progress);
        assert_eq!(handler.handler().called_time(), 1);

        let done = handler.handle(SseResponse::Data("[DONE]".to_string()));
        matches!(done, HandleProgress::Done);
        assert_eq!(handler.handler().called_time(), 2);
    }
    #[test]
    #[allow(non_snake_case)]
    fn chat_gptのsseレスポンスをChatResponseに変換可能() {
        let response = SseResponse::Data(make_stream_chat_json("Hello World"));
        assert_eq!(
            ChatResponse::from_sse(response).unwrap().delta_content(),
            "Hello World"
        );
    }
    #[test]
    #[allow(non_snake_case)]
    fn chat_gptのsseレスポンスをChatResponseに変換可能_done() {
        let response = SseResponse::Data("[DONE]".to_string());
        assert_eq!(
            ChatResponse::from_sse(response).unwrap(),
            ChatResponse::Done
        );
    }
    #[test]
    #[allow(non_snake_case)]
    fn chat_gptのレスポンスはChatResponseに変換可能() {
        let response = ChatResponse::from(make_stream_chat("Hello World"));
        assert_eq!(response.delta_content(), "Hello World");
    }

    // #[test]
    // fn gptのsseレスポンスを随時処理する() {
    //     let mock_handler = MockHandler::new();
    //     let gpt_handler = ChatGptHandler::new(mock_handler);

    //     let mut fake = FakeChatGpt::new();
    //     fake.set_chat_response("hello ");
    //     fake.set_chat_response("i ");
    //     fake.set_chat_response("am ");
    //     fake.set_chat_response("gpt.");

    //     let mut client = ChatGptClient::new(fake);

    //     client.chat("hello", &gpt_handler);
    //     assert_eq!(gpt_handler.handler_state().called_time(), 4);
    // }
    #[test]
    fn test_clear() {
        let mut chat_history = ChatHistory::new();
        chat_history.push_request("hello", Role::User);
        chat_history.push_response(ChatResponse::from("hello,i am gpt"));
        chat_history.push_request("thanks", Role::User);
        chat_history.push_response(ChatResponse::from("thanks too."));
        assert_eq!(
            chat_history.all(),
            vec![
                Message {
                    role: Role::User,
                    content: "hello".to_string(),
                },
                Message {
                    role: Role::Assistant,
                    content: "hello,i am gpt".to_string(),
                },
                Message {
                    role: Role::User,
                    content: "thanks".to_string(),
                },
                Message {
                    role: Role::Assistant,
                    content: "thanks too.".to_string(),
                },
            ]
        );
        chat_history.clear();
        assert_eq!(chat_history.all(), vec![]);
    }
    #[test]
    fn test_last_response() {
        let mut chat_history = ChatHistory::new();
        chat_history.push_response(ChatResponse::from("test"));
        assert_eq!(chat_history.last_response(), Some("test"));
    }
}

#[cfg(test)]
pub mod fakes {
    use std::io::Write;

    use super::*;
    pub struct FakeChatGpt {
        responses: Vec<String>,
        request_count: usize,
    }
    impl FakeChatGpt {
        pub fn new() -> Self {
            Self {
                responses: Vec::new(),
                request_count: 0,
            }
        }
        pub fn set_chat_response(&mut self, response: impl Into<String>) {
            self.responses.push(response.into());
        }
        pub fn request_count(&self) -> usize {
            self.request_count
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
    pub fn make_stream_chat(message: &str) -> StreamChat {
        serde_json::from_str(&make_stream_chat_json(message)).unwrap()
    }
}
