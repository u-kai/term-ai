use super::client::{
    ChatRequest, ChatResponse, GptClient, GptClientError, HandleResult, Message, OpenAIModel, Role,
};

#[derive(Debug, Clone, PartialEq)]
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
    fn push_response(&mut self, message: impl Into<String>) {
        self.inner
            .push(Message::new(Role::Assistant, message.into()));
    }
    fn push_request(&mut self, message: impl Into<String>, role: Role) {
        self.inner.push(Message::new(role, message));
    }
}

pub struct ChatManager {
    delta_store: DeltaContentStore,
    history: ChatHistory,
}
impl ChatManager {
    pub fn new() -> Self {
        Self {
            history: ChatHistory::new(),
            delta_store: DeltaContentStore::new(),
        }
    }
    pub fn update_by_request(&mut self, message: impl Into<String>, role: Role) {
        self.history.push_request(message, role);
    }
    pub fn update_by_response(&mut self, res: &ChatResponse) {
        if res.is_done() {
            self.history.push_response(self.delta_store.all_content());
            self.delta_store = DeltaContentStore::new();
            return;
        }
        self.delta_store.push(res);
    }
    pub fn last_response(&self) -> &str {
        self.history.last_response().unwrap_or("")
    }
}

#[derive(Debug, Clone)]
struct DeltaContentStore {
    inner: Vec<String>,
}

impl DeltaContentStore {
    fn new() -> Self {
        Self { inner: Vec::new() }
    }
    fn push(&mut self, message: &ChatResponse) {
        self.inner.push(message.delta_content().to_string());
    }
    fn all_content(&self) -> String {
        self.inner.join("")
    }
}

pub struct ChatGpt {
    client: GptClient,
    manager: ChatManager,
}
impl ChatGpt {
    pub fn from_env() -> Result<Self, GptClientError> {
        Ok(Self {
            client: GptClient::from_env()?,
            manager: ChatManager::new(),
        })
    }
    pub fn chat_gpt3<F: FnMut(&ChatResponse) -> HandleResult>(
        &mut self,
        message: impl Into<String>,
        f: F,
    ) {
        self.manager.update_by_request(message, Role::User);
        let req = ChatRequest::new(
            OpenAIModel::Gpt3Dot5Turbo,
            vec![Message::new(Role::User, self.manager.last_response())],
        );
        self.client.request_mut_fn(req, f);
    }
}

#[cfg(test)]
mod tests {

    use crate::gpt::client::{fakes::MockMutHandler, StreamChatMutHandler};

    use super::*;
    #[test]
    #[ignore = "gpt3のapiを叩くので、テストはスキップ"]
    fn gptとchatが可能() {
        let mut sut = ChatGpt::from_env().unwrap();
        let mut handler = MockMutHandler::new();
        let mut v = Vec::new();

        sut.chat_gpt3("こんにちは", |res| {
            handler.handle(&res);
            match res {
                ChatResponse::DeltaContent(s) => {
                    v.push(s.clone());
                    HandleResult::Progress
                }
                ChatResponse::Done => HandleResult::Done,
            }
        });

        println!("{:#?}", v);
        assert!(handler.called_time() > 0);
        assert!(v.len() > 0);
    }
    #[test]
    fn chat_managerはsseレスポンスからhistoryを更新する() {
        let mut sut = ChatManager::new();
        sut.update_by_request("こんにちは", Role::User);
        sut.update_by_response(&ChatResponse::DeltaContent("hello".to_string()));
        sut.update_by_response(&ChatResponse::DeltaContent(" world".to_string()));
        sut.update_by_response(&ChatResponse::Done);

        let mut expect = ChatHistory::new();
        expect.push_request("こんにちは", Role::User);
        expect.push_response("hello world");

        assert_eq!(sut.history, expect);
    }
    #[test]
    fn gptのsseレスポンスを保持可能() {
        let mut sut = DeltaContentStore::new();
        sut.push(&ChatResponse::DeltaContent("hello".to_string()));
        sut.push(&ChatResponse::DeltaContent(" world".to_string()));

        assert_eq!(sut.all_content(), "hello world");
    }
    #[test]
    fn historyの履歴はクリア可能() {
        let mut chat_history = ChatHistory::new();
        chat_history.push_request("hello", Role::User);
        chat_history.push_response("hello,i am gpt");
        chat_history.push_request("thanks", Role::User);
        chat_history.push_response("thanks too.");
        assert_eq!(
            chat_history.all(),
            vec![
                Message::new(Role::User, "hello".to_string(),),
                Message::new(Role::Assistant, "hello,i am gpt".to_string()),
                Message::new(Role::User, "thanks".to_string()),
                Message::new(Role::Assistant, "thanks too.".to_string()),
            ]
        );
        chat_history.clear();
        assert_eq!(chat_history.all(), vec![]);
    }
    #[test]
    fn historyの最後のデータを取得可能() {
        let mut chat_history = ChatHistory::new();
        chat_history.push_response("test");
        assert_eq!(chat_history.last_response(), Some("test"));
    }
}
