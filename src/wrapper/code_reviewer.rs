use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use crate::gpt::{GptClient, GptClientError, OpenAIModel, Role};

#[derive(Debug, Clone)]
pub struct CodeReviewer {
    gpt: GptClient,
}

impl CodeReviewer {
    const PREFIX: &'static str = "以下のコードをレビューしてください";
    pub fn from_env() -> Result<Self, GptClientError> {
        let gpt = GptClient::from_env()?;
        Ok(Self { gpt })
    }

    pub fn repl_gpt3_5(&mut self) -> Result<(), GptClientError> {
        let user = std::env::var("USER").unwrap_or("you".to_string());
        loop {
            let mut message = String::new();
            print!("{} > ", user);
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut message).unwrap();
            if message.trim() == "exit" {
                return Ok(());
            }
            if is_file_path(&message.trim()) {
                let mut file = File::open(message.trim()).unwrap();
                let mut code = String::new();
                file.read_to_string(&mut code).unwrap();
                message = format!("{}\n{}", Self::PREFIX, code);
            }
            print!("gpt > ");
            std::io::stdout().flush().unwrap();
            self.review(OpenAIModel::Gpt3Dot5Turbo, &message, &|event| {
                print!("{}", event);
                std::io::stdout().flush().unwrap();
            })?;
            println!();
            message.clear();
        }
    }
    pub fn review<F: Fn(&str)>(
        &mut self,
        model: OpenAIModel,
        code: &str,
        f: &F,
    ) -> Result<String, GptClientError> {
        let response = self.gpt.chat(model, Role::User, code, f)?;
        Ok(response)
    }
}

fn is_file_path(path: &str) -> bool {
    let path = Path::new(path);
    path.exists() && path.is_file()
}
