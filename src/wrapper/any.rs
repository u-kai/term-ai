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
impl<F: Fn(&str)> GptInput<F> {
    fn exe(&self) {
        (self.f)(&self.input);
    }
}

pub struct AnyHandler<T: ChatGpt> {
    gpt: T,
    sse_handlers: Vec<Box<dyn SseEventHandler>>,
    response_handlers: Vec<Box<dyn ResponseHandler>>,
    model: OpenAIModel,
}
impl<T: ChatGpt> AnyHandler<T> {
    pub fn new(gpt: T) -> Self {
        let sse_handlers: Vec<Box<dyn SseEventHandler>> = Vec::new();
        let response_handlers: Vec<Box<dyn ResponseHandler>> = Vec::new();
        Self {
            sse_handlers,
            response_handlers,
            gpt,
            model: OpenAIModel::Gpt3Dot5Turbo,
        }
    }
    pub fn add_listener(&mut self, handler: Box<dyn SseEventHandler>) {
        self.sse_handlers.push(handler);
    }
    pub fn add_response_handler(&mut self, handler: Box<dyn ResponseHandler>) {
        self.response_handlers.push(handler);
    }
    pub fn input(&mut self, input: &str) {
        let response = self
            .gpt
            .chat(GptInput {
                input: input.to_string(),
                model: self.model,
                role: Role::User,
                f: |_: &str| {
                    self.sse_handlers.iter().for_each(|handler| {
                        if handler.do_action(input) {
                            handler.handle(input);
                        }
                    });
                },
            })
            .unwrap();
        self.response_handlers.iter_mut().for_each(|handler| {
            if handler.do_action(input) {
                handler.handle(&response);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn 複数のresponse_handlerを登録して実行可能_成功() {
        let mut fake_gpt = FakeChatGpt::new();
        fake_gpt.add_response("ok");
        let mut sut = AnyHandler::new(fake_gpt);
        let listener1 = ResponseChecker::new("ok");
        let listener2 = ResponseLenChecker::new(2);
        sut.add_response_handler(Box::new(listener1));
        sut.add_response_handler(Box::new(listener2));
        // assert inner input history
        sut.input("hello");
    }
    #[test]
    #[should_panic]
    fn 複数のresponse_handlerを登録して実行可能_失敗() {
        let mut fake_gpt = FakeChatGpt::new();
        fake_gpt.add_response("ok");
        let mut sut = AnyHandler::new(fake_gpt);
        // invalid
        let listener1 = ResponseChecker::new("ng");
        let listener2 = ResponseLenChecker::new(2);
        sut.add_response_handler(Box::new(listener1));
        sut.add_response_handler(Box::new(listener2));
        // assert inner input history
        sut.input("hello");

        let mut fake_gpt = FakeChatGpt::new();
        fake_gpt.add_response("ok");
        let mut sut = AnyHandler::new(fake_gpt);
        let listener1 = ResponseChecker::new("ok");
        // invalid
        let listener2 = ResponseLenChecker::new(3);
        sut.add_response_handler(Box::new(listener1));
        sut.add_response_handler(Box::new(listener2));
        // assert inner input history
        sut.input("hello");
    }
    #[test]
    fn 複数のsse_event_handlerを登録して実行可能_成功() {
        let mut fake_gpt = FakeChatGpt::new();
        fake_gpt.add_response("hello");
        let mut sut = AnyHandler::new(fake_gpt);
        let listener1 = InputChecker::new("hello");
        let listener2 = InputLenChecker::new(5);
        sut.add_listener(Box::new(listener1));
        sut.add_listener(Box::new(listener2));
        // assert inner input history
        sut.input("hello");
    }
    #[test]
    #[should_panic]
    fn 複数のsse_event_handlerを登録して実行可能_error() {
        let mut fake_gpt = FakeChatGpt::new();
        fake_gpt.add_response("hello");
        let mut sut = AnyHandler::new(fake_gpt);
        let listener1 = InputChecker::new("hello");
        // invalid
        let listener2 = InputLenChecker::new(6);
        sut.add_listener(Box::new(listener1));
        sut.add_listener(Box::new(listener2));
        // assert inner input history
        sut.input("hello");

        let mut fake_gpt = FakeChatGpt::new();
        fake_gpt.add_response("hello");
        let mut sut = AnyHandler::new(fake_gpt);
        // invalid
        let listener1 = InputChecker::new("good bye");
        let listener2 = InputLenChecker::new(8);
        sut.add_listener(Box::new(listener1));
        sut.add_listener(Box::new(listener2));
        // assert inner input history
        sut.input("hello");
    }
    struct FakeChatGpt {
        responses: Vec<String>,
        index: usize,
    }
    impl FakeChatGpt {
        fn new() -> Self {
            Self {
                index: 0,
                responses: Vec::new(),
            }
        }
        fn add_response(&mut self, response: &str) {
            self.responses.push(response.to_string());
        }
    }

    impl super::ChatGpt for FakeChatGpt {
        fn chat<F: Fn(&str)>(&mut self, input: super::GptInput<F>) -> super::Result<String> {
            input.exe();
            Ok(self.responses.get(self.index).unwrap().to_string())
        }
    }

    struct InputChecker {
        input: String,
    }
    impl InputChecker {
        fn new(input: &str) -> Self {
            Self {
                input: input.to_string(),
            }
        }
    }
    impl super::SseEventHandler for InputChecker {
        fn do_action(&self, _: &str) -> bool {
            true
        }
        fn handle(&self, input: &str) {
            assert_eq!(input, self.input);
        }
    }

    struct ResponseChecker {
        expect: String,
    }
    impl ResponseChecker {
        fn new(expect: &str) -> Self {
            Self {
                expect: expect.to_string(),
            }
        }
    }
    impl ResponseHandler for ResponseChecker {
        fn do_action(&self, response: &str) -> bool {
            true
        }
        fn handle(&mut self, response: &str) {
            assert_eq!(self.expect, response);
        }
    }

    struct InputLenChecker {
        len: usize,
    }

    impl InputLenChecker {
        fn new(len: usize) -> Self {
            Self { len }
        }
    }
    impl SseEventHandler for InputLenChecker {
        fn do_action(&self, _: &str) -> bool {
            true
        }
        fn handle(&self, _: &str) {
            assert_eq!(self.len, 5);
        }
    }
    struct ResponseLenChecker {
        len: usize,
    }
    impl ResponseLenChecker {
        fn new(len: usize) -> Self {
            Self { len }
        }
    }
    impl ResponseHandler for ResponseLenChecker {
        fn do_action(&self, _: &str) -> bool {
            true
        }
        fn handle(&mut self, response: &str) {
            assert_eq!(self.len, response.len());
        }
    }
}
