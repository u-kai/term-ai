use std::io::Write;

use crate::gpt::{GptClient, OpenAIModel, Result, Role};

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
    fn convertor(&self, input: GptInput) -> GptInput;
}
pub trait ChatGpt {
    fn chat<F: Fn(&str)>(&mut self, input: &GptInput, handler: &F) -> Result<String>;
}

impl ChatGpt for GptClient {
    fn chat<F: Fn(&str)>(&mut self, input: &GptInput, handler: &F) -> Result<String> {
        self.chat(input.model, input.role, &input.input, handler)
    }
}

pub struct Printer {}
impl Printer {
    pub fn new() -> Self {
        println!();
        Self {}
    }
}
impl SseEventHandler for Printer {
    fn handle(&self, event: &str) {
        print!("{}", event);
        std::io::stdout().flush().unwrap();
    }
    fn do_action(&self, _: &str) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct GptInput {
    input: String,
    model: OpenAIModel,
    role: Role,
}
impl GptInput {
    pub fn new(input: impl Into<String>, model: OpenAIModel, role: Role) -> Self {
        Self {
            input: input.into(),
            model,
            role,
        }
    }
    pub fn input(&self) -> &str {
        &self.input
    }
    pub fn change_input(&mut self, input: impl Into<String>) {
        self.input = input.into();
    }
    pub fn change_model(&mut self, model: OpenAIModel) {
        self.model = model;
    }
    pub fn change_role(&mut self, role: Role) {
        self.role = role;
    }
}

pub struct AnyHandler<T: ChatGpt> {
    gpt: T,
    sse_handlers: Vec<Box<dyn SseEventHandler>>,
    response_handlers: Vec<Box<dyn ResponseHandler>>,
    input_convertor: Vec<Box<dyn InputConvertor>>,
}
impl<T: ChatGpt> AnyHandler<T> {
    pub fn new(gpt: T) -> Self {
        let sse_handlers: Vec<Box<dyn SseEventHandler>> = Vec::new();
        let response_handlers: Vec<Box<dyn ResponseHandler>> = Vec::new();
        let input_convertor: Vec<Box<dyn InputConvertor>> = Vec::new();
        Self {
            sse_handlers,
            response_handlers,
            input_convertor,
            gpt,
        }
    }
    pub fn add_event_handler(&mut self, handler: Box<dyn SseEventHandler>) {
        self.sse_handlers.push(handler);
    }
    pub fn add_response_handler(&mut self, handler: Box<dyn ResponseHandler>) {
        self.response_handlers.push(handler);
    }
    pub fn add_input_convertor(&mut self, handler: Box<dyn InputConvertor>) {
        self.input_convertor.push(handler);
    }
    pub fn handle(&mut self, input: GptInput) -> Result<()> {
        let input_str = input.input.clone();
        let input = self
            .input_convertor
            .iter()
            .filter(|handler| handler.do_convert(&input_str))
            .fold(input, |acc, handler| handler.convertor(acc));
        let response = self.gpt.chat(&input, &|event: &str| {
            self.sse_handlers
                .iter()
                .filter(|handler| handler.do_action(&input.input))
                .for_each(|handler| handler.handle(event));
        })?;
        self.response_handlers
            .iter_mut()
            .filter(|handler| handler.do_action(&input.input))
            .for_each(|handler| handler.handle(&response));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "本物のChatGPTを利用するため"]
    fn 本物のgptを利用して実行可能() {
        let gpt = GptClient::from_env().unwrap();
        let mut sut = AnyHandler::new(gpt);
        let event_checker = EventChecker {
            input: "".to_string(),
            condition: |s, _| {
                s.len() == 0
                    || !s
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .nth(0)
                        .unwrap()
                        .is_ascii()
            },
        };
        let response_checker = ResponseChecker {
            expect: "".to_string(),
            condition: |s, _| s.len() > 0,
        };
        let response_checker2 = ResponseChecker {
            expect: "".to_string(),
            condition: |s, _| {
                !s.chars()
                    .filter(|c| !c.is_whitespace())
                    .nth(0)
                    .unwrap()
                    .is_ascii()
            },
        };
        let input = GptInput::new("hello.", OpenAIModel::Gpt3Dot5Turbo, Role::User);
        let input_convertor = InputAdder::new("あなたは日本語で返事をしてくださいね．");
        sut.add_event_handler(Box::new(event_checker));
        sut.add_response_handler(Box::new(response_checker));
        sut.add_response_handler(Box::new(response_checker2));
        sut.add_input_convertor(Box::new(input_convertor));
        sut.handle(input).unwrap();
    }
    #[test]
    fn 複数のinput_convertorを登録して実行可能_成功() {
        let mut fake_gpt = FakeChatGpt::new();
        fake_gpt.add_response("ok");
        let mut sut = AnyHandler::new(fake_gpt);
        let convertor1 = InputAdder::new("hello");
        let convertor2 = InputAdder::new(" world");
        let checker = EventChecker::new("hello world");
        sut.add_input_convertor(Box::new(convertor1));
        sut.add_input_convertor(Box::new(convertor2));
        sut.add_event_handler(Box::new(checker));
        // assert inner input history
        sut.handle(GptInput::new("", OpenAIModel::Gpt3Dot5Turbo, Role::User))
            .unwrap();
    }
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
        sut.handle(GptInput::new(
            "hello",
            OpenAIModel::Gpt3Dot5Turbo,
            Role::User,
        ))
        .unwrap();
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
        sut.handle(GptInput::new(
            "hello",
            OpenAIModel::Gpt3Dot5Turbo,
            Role::User,
        ))
        .unwrap();

        let mut fake_gpt = FakeChatGpt::new();
        fake_gpt.add_response("ok");
        let mut sut = AnyHandler::new(fake_gpt);
        let listener1 = ResponseChecker::new("ok");
        // invalid
        let listener2 = ResponseLenChecker::new(3);
        sut.add_response_handler(Box::new(listener1));
        sut.add_response_handler(Box::new(listener2));
        // assert inner input history
        sut.handle(GptInput::new(
            "hello",
            OpenAIModel::Gpt3Dot5Turbo,
            Role::User,
        ))
        .unwrap();
    }
    #[test]
    fn 複数のsse_event_handlerを登録して実行可能_成功() {
        let mut fake_gpt = FakeChatGpt::new();
        fake_gpt.add_response("hello");
        let mut sut = AnyHandler::new(fake_gpt);
        let listener1 = EventChecker::new("hello");
        let listener2 = InputLenChecker::new(5);
        sut.add_event_handler(Box::new(listener1));
        sut.add_event_handler(Box::new(listener2));
        // assert inner input history
        sut.handle(GptInput::new(
            "hello",
            OpenAIModel::Gpt3Dot5Turbo,
            Role::User,
        ))
        .unwrap();
    }
    #[test]
    #[should_panic]
    fn 複数のsse_event_handlerを登録して実行可能_error() {
        let mut fake_gpt = FakeChatGpt::new();
        fake_gpt.add_response("hello");
        let mut sut = AnyHandler::new(fake_gpt);
        let listener1 = EventChecker::new("hello");
        // invalid
        let listener2 = InputLenChecker::new(6);
        sut.add_event_handler(Box::new(listener1));
        sut.add_event_handler(Box::new(listener2));
        // assert inner input history
        sut.handle(GptInput::new(
            "hello",
            OpenAIModel::Gpt3Dot5Turbo,
            Role::User,
        ))
        .unwrap();

        let mut fake_gpt = FakeChatGpt::new();
        fake_gpt.add_response("hello");
        let mut sut = AnyHandler::new(fake_gpt);
        // invalid
        let listener1 = EventChecker::new("good bye");
        let listener2 = InputLenChecker::new(8);
        sut.add_event_handler(Box::new(listener1));
        sut.add_event_handler(Box::new(listener2));
        // assert inner input history
        sut.handle(GptInput::new(
            "hello",
            OpenAIModel::Gpt3Dot5Turbo,
            Role::User,
        ))
        .unwrap();
    }
    /// テスト用のChatGpt
    /// 与えられたinputをそのままEventとして出力する
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
        fn chat<F: Fn(&str)>(&mut self, input: &GptInput, handler: &F) -> super::Result<String> {
            handler(&input.input);
            Ok(self.responses.get(self.index).unwrap().to_string())
        }
    }
    struct InputAdder {
        input: String,
    }
    impl InputAdder {
        fn new(input: &str) -> Self {
            Self {
                input: input.to_string(),
            }
        }
    }
    impl InputConvertor for InputAdder {
        fn do_convert(&self, _input: &str) -> bool {
            true
        }
        fn convertor(&self, input: GptInput) -> GptInput {
            GptInput {
                input: format!("{}{}", input.input, self.input),
                ..input
            }
        }
    }
    struct EventChecker<F: Fn(&str, &str) -> bool> {
        input: String,
        condition: F,
    }
    impl EventChecker<fn(&str, &str) -> bool> {
        fn new(input: &str) -> Self {
            Self {
                input: input.to_string(),
                condition: |l, r| l == r,
            }
        }
    }
    impl<F: Fn(&str, &str) -> bool> super::SseEventHandler for EventChecker<F> {
        fn do_action(&self, _: &str) -> bool {
            true
        }
        fn handle(&self, input: &str) {
            println!("input: {}", input);
            assert!((self.condition)(input, &self.input))
        }
    }

    struct ResponseChecker<F: Fn(&str, &str) -> bool> {
        expect: String,
        condition: F,
    }
    impl ResponseChecker<fn(&str, &str) -> bool> {
        fn new(expect: &str) -> Self {
            Self {
                expect: expect.to_string(),
                condition: |l, r| l == r,
            }
        }
    }
    impl<F: Fn(&str, &str) -> bool> ResponseHandler for ResponseChecker<F> {
        fn do_action(&self, _: &str) -> bool {
            true
        }
        fn handle(&mut self, response: &str) {
            println!("response: {}", response);
            assert!((self.condition)(response, &self.expect))
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
