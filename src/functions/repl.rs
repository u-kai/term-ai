use std::{io::Write, thread};

use crate::gpt::{
    chat::ChatGpt,
    client::{GptClientError, GptClientErrorKind, Message, OpenAIModel},
};

use super::{GptFunction, GptFunctionContainer, UserInput};

pub struct ChatGptRepl {
    chat_gpt: ChatGpt,
    display_gpt: String,
    display_user: String,
    container: GptFunctionContainer,
}
impl Default for ChatGptRepl {
    fn default() -> Self {
        Self::new()
    }
}
impl ChatGptRepl {
    pub fn new() -> Self {
        Self {
            chat_gpt: ChatGpt::from_env().unwrap(),
            display_gpt: Self::display_gpt_from_env(),
            display_user: Self::display_user_from_env(),
            container: GptFunctionContainer::new(),
        }
    }
    pub fn new_with_functions(gpt: ChatGpt, functions: GptFunctionContainer) -> Self {
        Self {
            chat_gpt: gpt,
            display_gpt: Self::display_gpt_from_env(),
            display_user: Self::display_user_from_env(),
            container: functions,
        }
    }
    fn display_user_from_env() -> String {
        std::env::var("USER").unwrap_or_else(|_| "you".to_string())
    }
    fn display_gpt_from_env() -> String {
        std::env::var("DISPLAY_GPT").unwrap_or_else(|_| "gpt".to_string())
    }
    pub fn add_functions(&mut self, f: Box<dyn GptFunction>) {
        self.container.add_functions(f);
    }
    pub fn repl_gpt4(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        self.repl(OpenAIModel::Gpt4)
    }
    pub fn repl_gpt3(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        self.repl(OpenAIModel::Gpt3Dot5Turbo)
    }
    // TODO: integrate with repl
    pub fn repl_with_input_fn<F: Fn(&str) + Send + Sync + 'static + Copy>(
        &mut self,
        model: OpenAIModel,
        input_fn: F,
    ) -> Result<(), Box<dyn std::error::Error + 'static>> {
        loop {
            self.user_first();
            let Ok(message) = Self::user_input() else {
                println!("invalid input. please input again");
                continue;
            };
            if Self::is_exit(&message) {
                return Ok(());
            }
            if Self::is_clear(&message) {
                self.chat_gpt.clear();
                println!("clear chat history");
                continue;
            }
            let mes = message.clone();
            let handle = thread::spawn(move || {
                input_fn(&mes);
            });
            let input = UserInput::new(&message);

            self.gpt_first();

            for message in self.container.input_to_messages(input) {
                self.chat_with_retry(model, &message)?;
            }

            self.container.action_at_end()?;
            Self::gpt_finish();
            handle.join().unwrap();
        }
    }
    pub fn repl(&mut self, model: OpenAIModel) -> Result<(), Box<dyn std::error::Error + 'static>> {
        loop {
            self.user_first();
            let Ok(message) = Self::user_input() else {
                println!("invalid input. please input again");
                continue;
            };
            if Self::is_exit(&message) {
                return Ok(());
            }
            if Self::is_clear(&message) {
                self.chat_gpt.clear();
                println!("clear chat history");
                continue;
            }
            let input = UserInput::new(&message);

            self.gpt_first();

            for message in self.container.input_to_messages(input) {
                self.chat_with_retry(model, &message)?;
            }

            self.container.action_at_end()?;
            Self::gpt_finish();
        }
    }

    fn chat(&mut self, model: OpenAIModel, message: &Message) -> Result<(), GptClientError> {
        self.chat_gpt.chat(model, &message, &mut |res| {
            Self::gpt_message(res.delta_content());
            self.container.handle_stream(res)
        })
    }
    fn chat_with_retry(
        &mut self,
        model: OpenAIModel,
        message: &Message,
    ) -> Result<(), GptClientError> {
        // chat is retry 3 times
        self.chat(model, message)
            .or_else(|e| self.maybe_retry(e, model, message))
            .or_else(|e| self.maybe_retry(e, model, message))
            .or_else(|e| self.maybe_retry(e, model, message))
    }
    fn maybe_retry(
        &mut self,
        e: GptClientError,
        model: OpenAIModel,
        message: &Message,
    ) -> Result<(), GptClientError> {
        match &e.kind {
            GptClientErrorKind::ReadStreamError(_)
            | GptClientErrorKind::NoResponse
            | GptClientErrorKind::ResponseError(_)
            | GptClientErrorKind::RequestError(_) => {
                self.chat_gpt.re_connect()?;
                self.chat(model, message)
            }
            _ => {
                return Err(e);
            }
        }
    }
    pub fn history(&self) -> &[Message] {
        self.chat_gpt.chat_history()
    }
    pub fn set_user_name(&mut self, name: &str) {
        self.display_user = name.to_string();
    }
    pub fn set_gpt_display(&mut self, name: &str) {
        self.display_gpt = name.to_string();
    }
    fn user_first(&self) {
        print!("{} > ", self.display_user);
        std::io::stdout().flush().unwrap();
    }
    fn user_input() -> std::io::Result<String> {
        let mut message = String::new();
        std::io::stdin().read_line(&mut message)?;
        Ok(message)
    }
    fn is_clear(message: &str) -> bool {
        message == "clear\n"
    }
    fn is_exit(message: &str) -> bool {
        message == "exit\n"
    }
    fn gpt_first(&self) {
        print!("{} > ", self.display_gpt);
        std::io::stdout().flush().unwrap();
    }
    fn gpt_message(message: &str) {
        print!("{}", message);
        std::io::stdout().flush().unwrap();
    }
    fn gpt_finish() {
        println!();
    }
}
