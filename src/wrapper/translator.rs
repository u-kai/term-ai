use std::{fs::read_to_string, io::Write};

use crate::{
    gpt::{GptClient, GptClientError, OpenAIModel, Role},
    repl::GptMessageHandler,
};

use super::common::is_file_path;

#[derive(Debug, Clone)]
pub struct TranslateWriter {
    translator: Translator,
}

impl TranslateWriter {
    pub fn from_env() -> Result<Self, crate::gpt::GptClientError> {
        Ok(Self {
            translator: Translator::from_env()?,
        })
    }
}

impl GptMessageHandler<GptClientError> for TranslateWriter {
    fn handle<F>(&mut self, message: &str, f: &F) -> Result<(), GptClientError>
    where
        F: Fn(&str) -> (),
    {
        let result = self
            .translator
            .chat(OpenAIModel::Gpt3Dot5Turbo, message, f)?;
        if is_file_path(message.trim()) {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(message.trim())
                .unwrap();
            file.write_all(result.as_bytes()).unwrap();
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct Translator {
    gpt: GptClient,
}

impl Translator {
    const PREFIX: &'static str = "以下の文章を翻訳してください";
    pub fn from_env() -> Result<Self, crate::gpt::GptClientError> {
        let gpt = GptClient::from_env()?;
        Ok(Self { gpt })
    }
    fn chat<F: Fn(&str) -> ()>(
        &mut self,
        model: OpenAIModel,
        message: &str,
        f: &F,
    ) -> Result<String, GptClientError> {
        let mut message = message.trim().to_string();
        if is_file_path(&message) {
            let request = Self::path_to_translate_request(&mut message);
            self.gpt.chat(model, Role::User, request, f)
        } else {
            self.gpt.chat(model, Role::User, message, f)
        }
    }
    fn path_to_translate_request(path: &str) -> String {
        format!("{}\n{}", Self::PREFIX, read_to_string(path).unwrap())
    }
}

impl GptMessageHandler<GptClientError> for Translator {
    fn handle<F>(&mut self, message: &str, f: &F) -> Result<(), GptClientError>
    where
        F: Fn(&str) -> (),
    {
        self.chat(OpenAIModel::Gpt3Dot5Turbo, &message, f)?;
        Ok(())
    }
}
