use std::io::Write;

use crate::gpt::client::HandleResult;

use super::{
    common::{change_request_to_file_content, is_file_path},
    GptFunction,
};

#[derive(Debug, PartialEq)]
pub struct FileTranslator {
    source_path: String,
    do_action: bool,
    inner: String,
}

impl FileTranslator {
    const PREFIX: &'static str = "以下の文章を日本語に翻訳してください";
    pub fn new() -> Self {
        Self {
            do_action: false,
            source_path: String::new(),
            inner: String::new(),
        }
    }
    fn append_result(&mut self) -> Result<(), std::io::Error> {
        let path = self.source_path.as_str();
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(self.source_path.as_str())?;
        file.write_all(format!("\n{}", self.results()).as_bytes())
    }
    fn results(&self) -> &str {
        &self.inner
    }
    fn do_action(&self) -> bool {
        self.do_action
    }
}
impl GptFunction for FileTranslator {
    fn switch_do_action(&mut self, request: &crate::gpt::client::Message) {
        if is_file_path(&request.content) {
            self.do_action = true;
            self.source_path = request.content.trim().to_string();
        }
    }
    fn action_at_end(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        if self.do_action() {
            self.append_result()?;
            *self = Self::new();
        }
        Ok(())
    }
    fn handle_stream(
        &mut self,
        response: &crate::gpt::client::ChatResponse,
    ) -> crate::gpt::client::HandleResult {
        if self.do_action() {
            match response {
                crate::gpt::client::ChatResponse::DeltaContent(content) => {
                    self.inner.push_str(content);
                    crate::gpt::client::HandleResult::Progress
                }
                crate::gpt::client::ChatResponse::Done => crate::gpt::client::HandleResult::Done,
            }
        } else {
            HandleResult::from(response)
        }
    }
    fn change_request(&self, request: &mut crate::gpt::client::Message) {
        if let Err(err) = change_request_to_file_content(Self::PREFIX, request) {
            eprintln!("{}", err);
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        functions::{common::test_tool::TestFileFactory, GptFunction},
        gpt::client::{ChatResponse, HandleResult, Message, Role},
    };

    use super::FileTranslator;

    #[test]
    #[ignore]
    fn fileのパスが来たらactionをonにする() {
        let test_file = TestFileFactory::create("tmp");
        test_file.create_root();
        test_file.create_file_under_root("test.rs", "hello");

        let message = Message::new(Role::User, "tmp/test.rs");

        let mut sut = FileTranslator::new();

        sut.switch_do_action(&message);

        test_file.remove_dir_all();

        assert_eq!(sut.do_action(), true);
    }
    #[test]
    #[ignore]
    fn gptのレスポンスが終了したら内部の結果をfileに追記し自身を初期化する() {
        let mut sut = FileTranslator::new();
        let test_file = TestFileFactory::create("tmp");
        test_file.create_file_under_root("hello.txt", "hello");

        let request = Message::new(Role::User, "tmp/hello.txt");

        sut.switch_do_action(&request);

        sut.handle_stream(&ChatResponse::DeltaContent("こん".to_string()));
        sut.handle_stream(&ChatResponse::DeltaContent("にちは".to_string()));
        sut.handle_stream(&ChatResponse::Done);

        sut.action_at_end().unwrap();

        let content = std::fs::read_to_string("tmp/hello.txt").unwrap();

        test_file.remove_dir_all();
        assert_eq!(content, "hello\nこんにちは");
        assert_eq!(sut.do_action(), false);
    }
    #[test]
    fn actionがoffであれば何もしない() {
        let mut message = Message::new(Role::User, "none");

        let mut sut = FileTranslator::new();
        sut.switch_do_action(&message);
        assert_eq!(sut.do_action(), false);

        sut.change_request(&mut message);
        assert_eq!(message.content, "none");

        let progress = sut.handle_stream(&ChatResponse::DeltaContent("こん".to_string()));
        assert_eq!(progress, HandleResult::Progress);

        let progress = sut.handle_stream(&ChatResponse::DeltaContent("にちは".to_string()));
        assert_eq!(progress, HandleResult::Progress);

        let progress = sut.handle_stream(&ChatResponse::Done);
        assert_eq!(progress, HandleResult::Done);

        assert_eq!(sut, FileTranslator::new());
        sut.action_at_end().unwrap();
    }
    #[test]
    fn gptからの結果を内部に格納する() {
        let mut sut = FileTranslator::new();
        sut.do_action = true;

        sut.handle_stream(&ChatResponse::DeltaContent("こん".to_string()));
        sut.handle_stream(&ChatResponse::DeltaContent("にちは".to_string()));
        sut.handle_stream(&ChatResponse::Done);

        assert_eq!(sut.results(), "こんにちは");
    }
}