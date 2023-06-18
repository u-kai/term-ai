use std::{cell::RefCell, io::Write};

pub struct GptRepl<T: Chat> {
    chat: T,
    user: String,
}

pub trait Chat {
    fn chat<F>(&self, message: &str, f: &F) -> ()
    where
        F: Fn(&str);
}

pub struct StubChat {
    count: RefCell<usize>,
    messages: Vec<String>,
}

impl StubChat {
    pub fn new() -> Self {
        StubChat {
            messages: Vec::new(),
            count: RefCell::new(0),
        }
    }
    pub fn add(&mut self, message: impl Into<String>) {
        self.messages.push(message.into());
    }
}

impl Chat for StubChat {
    fn chat<F>(&self, message: &str, f: &F) -> ()
    where
        F: Fn(&str),
    {
        let index = self.count.borrow().clone();
        match self.messages.get(index) {
            Some(message) => {
                f(message);
                *self.count.borrow_mut() += 1;
            }
            None => f(message),
        }
    }
}

impl<T: Chat> GptRepl<T> {
    pub fn new(c: T) -> Self {
        GptRepl {
            chat: c,
            user: std::env::var("USER").unwrap_or("you".to_string()),
        }
    }

    pub fn repl(&self) -> () {
        loop {
            self.user_first();
            let message = self.user_input();
            self.gpt_first();
            self.chat.chat(&message, &|event| self.gpt_message(event));
            self.gpt_finish();
        }
    }
    fn user_first(&self) {
        print!("{} > ", self.user);
        std::io::stdout().flush().unwrap();
    }
    fn user_input(&self) -> String {
        let mut message = String::new();
        std::io::stdin().read_line(&mut message).unwrap();
        message
    }
    fn gpt_first(&self) {
        print!("gpt > ");
        std::io::stdout().flush().unwrap();
    }
    fn gpt_message(&self, message: &str) {
        print!("{}", message);
        std::io::stdout().flush().unwrap();
    }
    fn gpt_finish(&self) {
        println!();
    }
}
