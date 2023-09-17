use crate::gpt::client::ChatResponse;

pub mod repl;

pub trait GptFunction {
    fn change_request(&self, request_message: &mut String);
    fn handle_stream(&mut self, response: &ChatResponse);
    fn action_at_end(&mut self);
}

#[cfg(test)]
mod tests {}
