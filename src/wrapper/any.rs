use crate::gpt::{OpenAIModel, Result, Role};

pub trait SseEventHandler {
    fn do_action(&self, input: &str) -> bool;
    fn handle(&self, event: &str);
}
pub trait ResponseHandler {
    fn do_action(&self, input: &str) -> bool;
    fn handle(&mut self, response: &str);
}
pub trait InputConvertor {
    fn do_convert(&self, input: &str) -> bool;
    fn convertor<F1: Fn(&str), F2: Fn(&str)>(&self, input: GptInput<F1>) -> GptInput<F2>;
}
pub trait ChatGpt {
    fn chat<F: Fn(&str)>(&mut self, input: GptInput<F>) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct GptInput<F: Fn(&str)> {
    input: String,
    model: OpenAIModel,
    role: Role,
    f: F,
}

pub struct AnyHandler<T: ChatGpt> {
    gpt: T,
    handlers: Vec<Box<dyn SseEventHandler>>,
    model: OpenAIModel,
}
impl<T: ChatGpt> AnyHandler<T> {
    pub fn new(gpt: T) -> Self {
        let handlers: Vec<Box<dyn SseEventHandler>> = Vec::new();
        Self {
            handlers,
            gpt,
            model: OpenAIModel::Gpt3Dot5Turbo,
        }
    }
    pub fn add_listener(&mut self, handler: Box<dyn SseEventHandler>) {
        self.handlers.push(handler);
    }
    pub fn input(&mut self, input: &str) {
        self.gpt
            .chat(GptInput {
                input: input.to_string(),
                model: self.model,
                role: Role::User,
                f: |_: &str| {
                    self.handlers.iter().for_each(|handler| {
                        if handler.do_action(input) {
                            handler.handle(input);
                        }
                    });
                },
            })
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    struct FakeChatGpt {}
    impl FakeChatGpt {
        fn new() -> Self {
            Self {}
        }
    }
    impl super::ChatGpt for FakeChatGpt {
        fn chat<F: Fn(&str)>(&mut self, input: super::GptInput<F>) -> super::Result<String> {
            (input.f)("message");
            Ok("response".to_string())
        }
    }

    struct InputHistory {
        input: String,
    }
    impl InputHistory {
        fn new(input: &str) -> Self {
            Self {
                input: input.to_string(),
            }
        }
    }
    impl super::SseEventHandler for InputHistory {
        fn do_action(&self, _: &str) -> bool {
            true
        }
        fn handle(&self, input: &str) {
            assert_eq!(input, self.input);
        }
    }
    use super::*;
    #[test]
    fn 複数のevent_listenerを登録して実行可能() {
        let mut sut = AnyHandler::new(FakeChatGpt::new());
        let listener1 = InputHistory::new("hello");
        sut.add_listener(Box::new(listener1));
        // assert inner input history
        sut.input("hello");
    }
}
