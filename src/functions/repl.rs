use std::io::Write;

use crate::gpt::{
    chat::ChatGpt,
    client::{ChatResponse, GptClientError, HandleResult, Message, OpenAIModel, Role},
};

pub struct ChatGptRepl {
    chat_gpt: ChatGpt,
    display_gpt: String,
    display_user: String,
}
impl ChatGptRepl {
    pub fn new() -> Self {
        Self {
            chat_gpt: ChatGpt::from_env().unwrap(),
            display_gpt: std::env::var("DISPLAY_GPT").unwrap_or("gpt".to_string()),
            display_user: std::env::var("USER").unwrap_or("you".to_string()),
        }
    }
    pub fn repl_gpt4<F: FnMut(&ChatResponse) -> HandleResult>(
        &mut self,
        f: &mut F,
    ) -> Result<(), GptClientError> {
        self.repl(OpenAIModel::Gpt4, f)
    }
    pub fn repl_gpt3<F: FnMut(&ChatResponse) -> HandleResult>(
        &mut self,
        f: &mut F,
    ) -> Result<(), GptClientError> {
        self.repl(OpenAIModel::Gpt3Dot5Turbo, f)
    }
    pub fn repl<F: FnMut(&ChatResponse) -> HandleResult>(
        &mut self,
        model: OpenAIModel,
        f: &mut F,
    ) -> Result<(), GptClientError> {
        loop {
            self.user_first();
            let message = Self::user_input();
            if Self::is_exit(&message) {
                return Ok(());
            }
            if Self::is_clear(&message) {
                self.chat_gpt.clear();
                println!("clear chat history");
                continue;
            }
            let message = Message::new(Role::User, &message);
            self.gpt_first();
            self.chat_gpt.chat(model, message, &mut |res| {
                Self::gpt_message(&res.delta_content());
                f(res)
            })?;
            // If above line process is heavy,I would like to proceed first below
            // It may be necessary to have an input that can receive the results of parallel processing.
            Self::gpt_finish();
        }
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
    fn user_input() -> String {
        let mut message = String::new();
        std::io::stdin().read_line(&mut message).unwrap();
        message
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
