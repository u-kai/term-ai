use crate::gpt::client::Message;

use super::{common::is_file_path, GptFunction};
use std::{fs::File, io::Read};
//
#[derive(Debug, Clone)]
pub struct CodeReviewer {
    prefix: &'static str,
}

impl CodeReviewer {
    const PREFIX: &'static str = "以下のコードをレビューしてください";
    pub fn new(prefix: &'static str) -> Self {
        Self { prefix }
    }
}

impl Default for CodeReviewer {
    fn default() -> Self {
        Self::new(Self::PREFIX)
    }
}
impl GptFunction for CodeReviewer {
    fn change_request(&self, request: &mut Message) {
        if is_file_path(&request.content) {
            let path = request.content.trim();
            let mut file = File::open(path).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            let message_content = request.change_content();
            *message_content = format!("{}\n{}", self.prefix, content);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::gpt::client::{Message, Role};

    use super::*;
    #[test]
    #[ignore]
    fn messageの入力がfile_pathであればcode_reviewerはmessageの内容をコードレビュー依頼に変換する()
    {
        std::fs::remove_dir_all("tmp").unwrap_or_default();
        std::fs::create_dir("tmp").unwrap();
        let mut file = std::fs::File::create("tmp/test.rs").unwrap();
        let file_content = "fn main() { println!(\"Hello, world!\"); }";
        file.write_all(file_content.as_bytes()).unwrap();

        let prefix = "以下のコードをレビューしてください";
        let mut code_reviewer = CodeReviewer::new(prefix);
        let mut message = Message::new(Role::User, "tmp/test.rs");
        code_reviewer.change_request(&mut message);

        assert_eq!(
            message,
            Message::new(Role::User, format!("{}\n{}", prefix, file_content))
        );
    }
}

//
//impl GptMessageHandler<GptClientError> for CodeReviewer {
//    fn clear_history(&mut self) {
//        self.gpt.clear_history();
//    }
//    fn handle<F>(&mut self, message: &str, f: &F) -> Result<(), GptClientError>
//    where
//        F: Fn(&str) -> (),
//    {
//        let mut message = message.trim().to_string();
//        if is_file_path(&message) {
//            Self::path_to_code_review_request(&mut message);
//        }
//        self.review(OpenAIModel::Gpt3Dot5Turbo, &message, f)?;
//        Ok(())
//    }
//}
//
//impl CodeReviewer {
//    const PREFIX: &'static str = "以下のコードをレビューしてください";
//    pub fn from_env() -> Result<Self, GptClientError> {
//        let gpt = GptClient::from_env()?;
//        Ok(Self { gpt })
//    }
//    pub fn review<F: Fn(&str)>(
//        &mut self,
//        model: OpenAIModel,
//        code: &str,
//        f: &F,
//    ) -> Result<String, GptClientError> {
//        let response = self.gpt.chat(model, Role::User, code, f)?;
//        Ok(response)
//    }
//    fn path_to_code_review_request(path: &mut String) {
//        if is_file_path(&path) {
//            let mut file = File::open(&path).unwrap();
//            let mut code = String::new();
//            file.read_to_string(&mut code).unwrap();
//            *path = format!("{}\n{}", Self::PREFIX, code);
//        }
//    }
//}
