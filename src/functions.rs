use crate::gpt::client::{ChatResponse, HandleResult, Message, Role};

pub mod code_capture;
pub mod code_reviewer;
mod common;
pub mod repl;
#[cfg(target_os = "macos")]
pub mod speaker;
pub mod translator;

pub struct GptFunctionContainer {
    functions: Vec<Box<dyn GptFunction>>,
}

impl GptFunctionContainer {
    pub fn new() -> Self {
        Self {
            functions: vec![Box::new(GptDefaultFunction::new())],
        }
    }
    pub fn add_functions(&mut self, f: Box<dyn GptFunction>) {
        self.functions.push(f);
    }
}
impl Default for GptFunctionContainer {
    fn default() -> Self {
        Self::new()
    }
}
impl Default for GptDefaultFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl GptFunction for GptFunctionContainer {
    fn setup_for_action(&mut self, input: &UserInput) {
        self.functions
            .iter_mut()
            .for_each(|f| f.setup_for_action(input));
    }
    // only one function can action input to message
    fn input_to_messages(&self, input: UserInput) -> Vec<Message> {
        self.functions
            .iter()
            .filter(|f| f.can_action())
            .next()
            .as_ref()
            // This struct is must contain default function
            // so unwrap is safe
            .unwrap()
            .input_to_messages(input)
    }
    fn handle_stream(&mut self, response: &ChatResponse) -> HandleResult {
        self.functions
            .iter_mut()
            .fold(HandleResult::Progress, |_acc, f| f.handle_stream(response))
    }
    fn action_at_end(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        self.functions
            .iter_mut()
            .fold(Ok(()), |acc, f| acc.and_then(|_| f.action_at_end()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        functions::{GptFunction, UserInput},
        gpt::client::{ChatResponse, HandleResult},
    };

    use super::GptFunctionContainer;

    #[test]
    #[allow(non_snake_case)]
    fn UserInputはGPT_REQUEST_LIMITを超える文字列を分割してMessageに変換する() {
        let input = UserInput::new("hello world. hello. world.");
        let messages = input.to_messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "hello world.");
        assert_eq!(messages[1].content, " hello. world.");
    }
    #[test]
    #[allow(non_snake_case)]
    fn GptFunctionContainerはGptDefaultFunctionを保持しておりhandle_streamでProgressDoneを正常に判断できる(
    ) {
        let mut container = GptFunctionContainer::new();

        let progress = container.handle_stream(&ChatResponse::DeltaContent("hello".to_string()));
        assert_eq!(progress, HandleResult::Progress);
        let progress = container.handle_stream(&ChatResponse::Done);
        assert_eq!(progress, HandleResult::Done);
    }
    #[test]
    #[allow(non_snake_case)]
    fn 最後に追加されたhandle_streamの結果が返る() {
        struct TestFunction {
            result: HandleResult,
        }
        impl GptFunction for TestFunction {
            fn handle_stream(&mut self, _response: &ChatResponse) -> HandleResult {
                self.result.clone()
            }
        }
        let mut container = GptFunctionContainer::new();
        container.add_functions(Box::new(TestFunction {
            result: HandleResult::Progress,
        }));
        container.add_functions(Box::new(TestFunction {
            result: HandleResult::Done,
        }));

        let progress = container.handle_stream(&ChatResponse::DeltaContent("hello".to_string()));
        assert_eq!(progress, HandleResult::Done);
    }
}

pub struct GptDefaultFunction {}

impl GptDefaultFunction {
    pub fn new() -> Self {
        Self {}
    }
}
impl GptFunction for GptDefaultFunction {}

#[cfg(not(test))]
const GPT_REQUEST_LIMIT: usize = 4096;
#[cfg(test)]
const GPT_REQUEST_LIMIT: usize = 15;
#[derive(Debug, Clone)]
pub struct UserInput(String);
impl UserInput {
    pub fn new(input: impl Into<String>) -> Self {
        Self(input.into())
    }
    pub fn content(&self) -> &str {
        &self.0
    }
    pub fn to_messages(self) -> Vec<Message> {
        if self.content().len() <= GPT_REQUEST_LIMIT {
            return vec![Message::new(Role::User, self.content())];
        }
        let role = Role::User;
        // TODO split char is not only dot.
        self.content().split_inclusive('.').fold(
            vec![Message::new(role, "")],
            |mut acc, sentence| {
                let last = acc.last_mut().unwrap();
                // case last content is empty, push sentence to last content even if it is over limit.
                if last.content.is_empty() {
                    last.content.push_str(sentence);
                    return acc;
                };
                // case last content is not empty, push sentence to new content if it is over limit.
                if last.content.len() + sentence.len() >= GPT_REQUEST_LIMIT {
                    acc.push(Message::new(role, sentence));
                    return acc;
                };
                // case last content is not empty, push sentence to last content if it is not over limit.
                acc.last_mut().unwrap().content.push_str(sentence);
                acc
            },
        )
    }
}
pub trait GptFunction {
    fn input_to_messages(&self, input: UserInput) -> Vec<Message> {
        input.to_messages()
    }
    fn setup_for_action(&mut self, _input: &UserInput) {}
    fn can_action(&self) -> bool {
        false
    }
    fn handle_stream(&mut self, response: &ChatResponse) -> HandleResult {
        HandleResult::from(response)
    }
    fn action_at_end(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        Ok(())
    }
}
