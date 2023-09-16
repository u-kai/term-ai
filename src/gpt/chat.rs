use super::client::{ChatResponse, Message, Role};

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
        self.inner.push(Message::new(
            Role::Assistant,
            message.delta_content().to_string(),
        ));
    }
    fn push_request(&mut self, message: impl Into<String>, role: Role) {
        self.inner.push(Message::new(role, message));
    }
}

mod tests {
    use super::*;
    #[test]
    fn historyの履歴はクリア可能() {
        let mut chat_history = ChatHistory::new();
        chat_history.push_request("hello", Role::User);
        chat_history.push_response(ChatResponse::from("hello,i am gpt"));
        chat_history.push_request("thanks", Role::User);
        chat_history.push_response(ChatResponse::from("thanks too."));
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
        chat_history.push_response(ChatResponse::from("test"));
        assert_eq!(chat_history.last_response(), Some("test"));
    }
}
