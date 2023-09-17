use crate::gpt::client::{ChatResponse, HandleResult, Message};

pub mod code_reviewer;
mod common;
pub mod repl;

pub struct GptDefaultFunction {}

impl GptDefaultFunction {
    pub fn new() -> Self {
        Self {}
    }
}
impl GptFunction for GptDefaultFunction {}

pub trait GptFunction {
    fn switch_do_action(&mut self, request: &Message) {}
    fn change_request(&self, request: &mut Message) {}
    fn handle_stream(&mut self, response: &ChatResponse) -> HandleResult {
        match response {
            ChatResponse::DeltaContent(_content) => HandleResult::Progress,
            ChatResponse::Done => HandleResult::Done,
        }
    }
    fn action_at_end(&mut self) {}
}
