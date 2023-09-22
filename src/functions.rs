use crate::gpt::client::{ChatResponse, HandleResult, Message};

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
    fn switch_do_action(&mut self, request: &Message) {
        self.functions
            .iter_mut()
            .for_each(|f| f.switch_do_action(request));
    }
    fn change_request(&self, request: &mut Message) {
        self.functions.iter().for_each(|f| {
            f.change_request(request);
        });
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
        functions::GptFunction,
        gpt::client::{ChatResponse, HandleResult},
    };

    use super::GptFunctionContainer;

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

pub trait GptFunction {
    fn switch_do_action(&mut self, _request: &Message) {}
    fn change_request(&self, _request: &mut Message) {}
    fn handle_stream(&mut self, response: &ChatResponse) -> HandleResult {
        HandleResult::from(response)
    }
    fn action_at_end(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        Ok(())
    }
}
