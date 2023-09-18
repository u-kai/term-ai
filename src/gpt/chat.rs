use super::client::{
    ChatRequest, ChatResponse, GptClient, GptClientError, HandleResult, Message, OpenAIKey,
    OpenAIModel, Result, Role,
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
    fn last_request(&self) -> Option<&Message> {
        if self.inner.len() < 2 {
            self.inner.last()
        } else {
            self.inner.get(self.inner.len() - 2)
        }
    }
    fn last_response(&self) -> Option<&str> {
        if self.inner.len() < 2 {
            None
        } else {
            self.inner.last().map(|m| m.content.as_str())
        }
    }
    fn push_response(&mut self, message: impl Into<String>) {
        self.inner
            .push(Message::new(Role::Assistant, message.into()));
    }
    fn push_request(&mut self, message: Message) {
        self.inner.push(message);
    }
}

#[derive(Debug)]
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
    pub fn make_request(&self, model: OpenAIModel) -> ChatRequest {
        ChatRequest::new(model, self.history.all())
    }
    pub fn update_by_request(&mut self, message: Message) {
        self.history.push_request(message);
    }
    pub fn update_by_response(&mut self, res: &ChatResponse) {
        if res.is_done() {
            self.history.push_response(self.delta_store.all_content());
            self.delta_store = DeltaContentStore::new();
            return;
        }
        self.delta_store.push(res);
    }
    pub fn last_request(&self) -> Option<&Message> {
        self.history.last_request()
    }
    pub fn last_response(&self) -> &str {
        self.history.last_response().unwrap_or("")
    }
    pub fn clear(&mut self) {
        self.history.clear();
        self.delta_store = DeltaContentStore::new();
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
    pub(crate) manager: ChatManager,
}
impl ChatGpt {
    pub fn new(key: OpenAIKey) -> Self {
        Self {
            client: GptClient::new(key),
            manager: ChatManager::new(),
        }
    }
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            client: GptClient::from_env()?,
            manager: ChatManager::new(),
        })
    }

    pub fn chat<F: FnMut(&ChatResponse) -> HandleResult>(
        &mut self,
        model: OpenAIModel,
        message: Message,
        f: &mut F,
    ) -> Result<()> {
        self.manager.update_by_request(message);
        let req = self.manager.make_request(model);
        println!("req: {:?}", req);
        self.client.request_mut_fn(req, |res| {
            self.manager.update_by_response(res);
            f(res)
        })
    }
    pub fn clear(&mut self) {
        self.manager.clear();
    }
    pub fn last_request(&self) -> Option<&Message> {
        self.manager.last_request()
    }
    pub fn last_response(&self) -> &str {
        self.manager.last_response()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    #[ignore = "gpt3のapiを叩くので、テストはスキップ"]
    fn gptとchatが可能() {
        let mut sut = ChatGpt::from_env().unwrap();
        let mut buf = String::new();

        sut.chat(
            OpenAIModel::Gpt3Dot5Turbo,
            Message::new(Role::User, "こんにちは"),
            &mut |res| match res {
                ChatResponse::DeltaContent(s) => {
                    buf.push_str(s);
                    HandleResult::Progress
                }
                ChatResponse::Done => HandleResult::Done,
            },
        )
        .unwrap();

        assert_eq!(
            sut.last_request().unwrap(),
            &Message::new(Role::User, "こんにちは")
        );
        assert_eq!(sut.last_response(), buf);
    }
    #[test]
    #[allow(non_snake_case)]
    fn chat_managerは全ての履歴と次のリクエストメッセージからChatRequestを作成する() {
        let gpt3 = OpenAIModel::Gpt3Dot5Turbo;
        let mut sut = ChatManager::new();

        sut.update_by_request(Message::new(Role::User, "こんにちは"));

        let req = sut.make_request(gpt3);
        assert_eq!(
            req,
            ChatRequest::new(gpt3, vec![Message::new(Role::User, "こんにちは")])
        );

        sut.update_by_response(&ChatResponse::DeltaContent("hello".to_string()));
        sut.update_by_response(&ChatResponse::DeltaContent(" world".to_string()));
        sut.update_by_response(&ChatResponse::Done);

        sut.update_by_request(Message::new(Role::User, "僕ってかっこいいですか？"));
        let req = sut.make_request(gpt3);
        assert_eq!(
            req,
            ChatRequest::new(
                gpt3,
                vec![
                    Message::new(Role::User, "こんにちは"),
                    Message::new(Role::Assistant, "hello world"),
                    Message::new(Role::User, "僕ってかっこいいですか？"),
                ]
            )
        );
    }
    #[test]
    fn chat_managerはsseレスポンスからhistoryを更新する() {
        let mut sut = ChatManager::new();
        sut.update_by_request(Message::new(Role::User, "こんにちは"));
        sut.update_by_response(&ChatResponse::DeltaContent("hello".to_string()));
        sut.update_by_response(&ChatResponse::DeltaContent(" world".to_string()));
        sut.update_by_response(&ChatResponse::Done);

        let mut expect = ChatHistory::new();
        expect.push_request(Message::new(Role::User, "こんにちは"));
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
        chat_history.push_request(Message::new(Role::User, "hello"));
        chat_history.push_response("hello,i am gpt");
        chat_history.push_request(Message::new(Role::User, "thanks"));
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
        chat_history.push_request(Message::new(Role::User, "hello"));
        chat_history.push_response("test");
        assert_eq!(chat_history.last_response(), Some("test"));
    }
}
