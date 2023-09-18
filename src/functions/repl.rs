use std::io::Write;

use crate::gpt::{
    chat::ChatGpt,
    client::{GptClientError, HandleResult, Message, OpenAIModel, Role},
};

use super::{GptDefaultFunction, GptFunction};

pub struct ChatGptRepl {
    chat_gpt: ChatGpt,
    display_gpt: String,
    display_user: String,
    functions: Vec<Box<dyn GptFunction>>,
}
impl ChatGptRepl {
    pub fn new() -> Self {
        Self {
            chat_gpt: ChatGpt::from_env().unwrap(),
            display_gpt: std::env::var("DISPLAY_GPT").unwrap_or("gpt".to_string()),
            display_user: std::env::var("USER").unwrap_or("you".to_string()),
            functions: vec![Box::new(GptDefaultFunction::new())],
        }
    }
    pub fn add_functions(&mut self, f: Box<dyn GptFunction>) {
        self.functions.push(f);
    }
    pub fn repl_gpt4(&mut self) -> Result<(), GptClientError> {
        self.repl(OpenAIModel::Gpt4)
    }
    pub fn repl_gpt3(&mut self) -> Result<(), GptClientError> {
        self.repl(OpenAIModel::Gpt3Dot5Turbo)
    }
    pub fn repl(&mut self, model: OpenAIModel) -> Result<(), GptClientError> {
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
            let mut message = Message::new(Role::User, &message);

            self.functions
                .iter_mut()
                .for_each(|f| f.switch_do_action(&message));

            self.functions.iter().for_each(|f| {
                f.change_request(&mut message);
            });

            self.gpt_first();
            self.chat_gpt.chat(model, message, &mut |res| {
                Self::gpt_message(&res.delta_content());
                self.functions
                    .iter_mut()
                    .fold(HandleResult::Progress, |acc, f| f.handle_stream(&res))
            })?;

            self.functions.iter_mut().for_each(|f| {
                f.action_at_end();
            });
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
