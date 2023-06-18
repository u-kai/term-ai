use std::{cell::RefCell, io::Write};

use crate::gpt::{GptClient, OpenAIModel, Role};

pub struct GptRepl<T: Chat> {
    chat: T,
    user: String,
}

pub trait Chat {
    fn chat<F>(&mut self, message: &str, f: &F) -> ()
    where
        F: Fn(&str);
}

pub struct StubChat {
    count: usize,
    messages: Vec<String>,
}

impl StubChat {
    pub fn new() -> Self {
        StubChat {
            messages: Vec::new(),
            count: 0,
        }
    }
    pub fn add(&mut self, message: impl Into<String>) {
        self.messages.push(message.into());
    }
}

impl Chat for StubChat {
    fn chat<F>(&mut self, message: &str, f: &F) -> ()
    where
        F: Fn(&str),
    {
        let index = self.count;
        match self.messages.get(index) {
            Some(message) => {
                f(message);
                self.count += 1;
            }
            None => f(message),
        }
    }
}

pub struct GptChat {
    client: GptClient,
}

impl Chat for GptChat {
    fn chat<F>(&mut self, message: &str, f: &F) -> ()
    where
        F: Fn(&str),
    {
        self.client
            .chat(OpenAIModel::Gpt3Dot5Turbo, Role::User, message, f)
            .unwrap();
    }
}

impl<T: Chat> GptRepl<T> {
    pub fn new(c: T) -> Self {
        GptRepl {
            chat: c,
            user: std::env::var("USER").unwrap_or("you".to_string()),
        }
    }

    pub fn repl(&mut self) -> () {
        loop {
            self.user_first();
            let message = Self::user_input();
            self.gpt_first();
            self.chat.chat(&message, &|event| Self::gpt_message(event));
            Self::gpt_finish();
        }
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
    fn gpt_first(&self) {
        print!("gpt > ");
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
