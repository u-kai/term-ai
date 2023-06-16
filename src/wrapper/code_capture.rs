use std::{cell::RefCell, io::Write};

use crate::gpt::{GptClient, GptClientError, OpenAIModel, Role};

#[derive(Debug, Clone)]
pub struct CodeCaptureGpt {
    gpt: GptClient,
    code_capture: RefCell<CodeCapture>,
}

impl CodeCaptureGpt {
    pub fn from_env() -> Result<Self, GptClientError> {
        let gpt = GptClient::from_env()?;
        Ok(Self {
            gpt,
            code_capture: RefCell::new(CodeCapture::new()),
        })
    }
    pub fn repl(&mut self, filename: &str) -> Result<(), GptClientError> {
        let user = std::env::var("USER").unwrap_or("you".to_string());
        loop {
            let mut message = String::new();
            print!("{} > ", user);
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut message).unwrap();
            if message.trim() == "exit" {
                return Ok(());
            }
            print!("gpt > ");
            std::io::stdout().flush().unwrap();
            self.chat_and_capture_code_to_file(
                OpenAIModel::Gpt3Dot5Turbo,
                message.as_str(),
                filename,
            )?;
            println!();
            if let Some(code) = self.code_capture.borrow().code() {
                println!("code: \n{:#?}", code.code);
            }
            message.clear();
        }
    }
    pub fn chat_and_capture_code_to_file(
        &mut self,
        model: OpenAIModel,
        message: &str,
        filename: &str,
    ) -> Result<(), GptClientError> {
        self.gpt.chat(model, Role::User, message, &|event| {
            self.code_capture.borrow_mut().add(event);
            print!("{}", event);
            std::io::stdout().flush().unwrap();
        })?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CodeCapture {
    inner: String,
}
impl CodeCapture {
    pub fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }
    pub fn add(&mut self, line: &str) {
        self.inner.push_str(line);
    }
    pub fn code(&self) -> Option<Code> {
        let mut chars = self.inner.chars().skip_while(|c| c != &'`');
        let (Some('`'), Some('`'),Some('`')) = (chars.next(),chars.next(), chars.next()) else {
            return None;
        };
        // not reach code
        if !chars.clone().any(|c| c == '\n') {
            return None;
        }
        let lang = chars
            .by_ref()
            .take_while(|c| c != &'\n')
            .collect::<String>();
        let code = chars.collect::<String>();
        let code = code.split("```").next()?;
        Some(Code {
            code: code.to_string(),
            lang: Lang::from_str(lang.as_str())?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code {
    code: String,
    lang: Lang,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lang {
    Rust,
}
impl Lang {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "rust" => Some(Self::Rust),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_code_capture() {
        let mut sut = CodeCapture::new();
        let line = "以下のコードは，1から10までの整数の和を求めるプログラムです。";
        sut.add(line);
        let before_code = "`";
        sut.add(before_code);
        sut.add(before_code);
        let code = "`ru";
        sut.add(code);
        let code = "st";
        sut.add(code);
        let code = "\n";
        sut.add(code);
        let code = "print";
        sut.add(code);
        let code = "ln!(\"Hello, world!\");\n";
        sut.add(code);
        let code = "```";
        sut.add(code);
        assert_eq!(
            sut.code(),
            Some(Code {
                code: "println!(\"Hello, world!\");\n".to_string(),
                lang: Lang::Rust
            })
        );
    }
}
