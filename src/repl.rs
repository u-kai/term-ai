use std::io::Write;

use crate::gpt::{GptClient, GptClientError, OpenAIModel, Role};

pub struct GptRepl<E: std::error::Error, T: MessageHandler<E>> {
    chat: T,
    user: String,
    display_gpt: String,
    _phantom: std::marker::PhantomData<E>,
}

pub trait MessageHandler<E: std::error::Error> {
    fn handle<F>(&mut self, message: &str, f: &F) -> Result<(), E>
    where
        F: Fn(&str);
}

pub struct GptChat {
    model: OpenAIModel,
    client: GptClient,
}

impl GptChat {
    pub fn from_env(model: OpenAIModel) -> Result<Self, crate::gpt::GptClientError> {
        let client = GptClient::from_env()?;
        Ok(Self { client, model })
    }
    pub fn first_command(&mut self, message: &str) -> () {
        self.client
            .chat(self.model, Role::System, message, &|_event| ())
            .unwrap();
    }
}
impl MessageHandler<GptClientError> for GptChat {
    fn handle<F>(&mut self, message: &str, f: &F) -> Result<(), GptClientError>
    where
        F: Fn(&str),
    {
        self.client.chat(self.model, Role::User, message, f)?;
        Ok(())
    }
}

impl GptRepl<GptClientError, GptChat> {
    pub fn from_env(model: OpenAIModel) -> Result<Self, crate::gpt::GptClientError> {
        let chat = GptChat::from_env(model)?;
        Ok(Self::new(chat))
    }
}

impl<E: std::error::Error, T: MessageHandler<E>> GptRepl<E, T> {
    pub fn new(c: T) -> Self {
        GptRepl {
            chat: c,
            display_gpt: std::env::var("DISPLAY_GPT").unwrap_or("gpt".to_string()),
            user: std::env::var("USER").unwrap_or("you".to_string()),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn repl(&mut self) -> Result<(), E> {
        loop {
            self.user_first();
            let message = Self::user_input();
            if Self::is_exit(&message) {
                return Ok(());
            }
            self.gpt_first();
            self.chat
                .handle(&message, &|event| Self::gpt_message(event))?;
            Self::gpt_finish();
        }
    }
    pub fn set_user_name(&mut self, name: &str) {
        self.user = name.to_string();
    }
    pub fn set_gpt_display(&mut self, name: &str) {
        self.display_gpt = name.to_string();
    }
    fn user_first(&self) {
        print!("{} > ", self.user);
        std::io::stdout().flush().unwrap();
    }
    fn user_input() -> String {
        let mut message = String::new();
        std::io::stdin().read_line(&mut message).unwrap();
        message
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
