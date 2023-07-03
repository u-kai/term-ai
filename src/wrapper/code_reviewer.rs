use std::{fs::File, io::Read};

use crate::{
    gpt::{GptClient, GptClientError, OpenAIModel, Role},
    repl::GptMessageHandler,
};

use super::common::is_file_path;

#[derive(Debug, Clone)]
pub struct CodeReviewer {
    gpt: GptClient,
}

impl GptMessageHandler<GptClientError> for CodeReviewer {
    fn handle<F>(&mut self, message: &str, f: &F) -> Result<(), GptClientError>
    where
        F: Fn(&str) -> (),
    {
        let mut message = message.trim().to_string();
        if is_file_path(&message) {
            Self::path_to_code_review_request(&mut message);
        }
        self.review(OpenAIModel::Gpt3Dot5Turbo, &message, f)?;
        Ok(())
    }
}

impl CodeReviewer {
    const PREFIX: &'static str = "以下のコードをレビューしてください";
    pub fn from_env() -> Result<Self, GptClientError> {
        let gpt = GptClient::from_env()?;
        Ok(Self { gpt })
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
    fn path_to_code_review_request(path: &mut String) {
        if is_file_path(&path) {
            let mut file = File::open(&path).unwrap();
            let mut code = String::new();
            file.read_to_string(&mut code).unwrap();
            *path = format!("{}\n{}", Self::PREFIX, code);
        }
    }
}
