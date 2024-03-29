use std::io::Write;

use crate::gpt::client::{HandleResult, Message};

use super::{
    common::{get_file_content, is_file_path},
    GptFunction, UserInput,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Translator {
    mode: TranslateMode,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TranslateMode {
    ToJapanese,
    ToEnglish,
    ToKorean,
    ToChinese,
}
impl Translator {
    const TO_JAPANESE_PREFIX: &'static str = "以下の文章を日本語に翻訳してください。";
    const TO_KOREAN_PREFIX: &'static str = "다음 문장을 한국어로 번역하십시오.";
    const TO_CHINESE_PREFIX: &'static str = "请将以下句子翻译成中文。";
    const TO_ENGLISH_PREFIX: &'static str = "Please translate the following sentences into English";
    pub fn new(mode: TranslateMode) -> Self {
        Self { mode }
    }
}
impl Default for Translator {
    fn default() -> Self {
        Self::new(TranslateMode::ToEnglish)
    }
}
impl GptFunction for Translator {
    fn input_to_messages(&self, input: super::UserInput) -> Vec<Message> {
        let add_prefix = |mut message: Message| -> Message {
            let content = Lang::from(message.content.as_str());
            let change_content = message.change_content();
            match content {
                Lang::English(s) => {
                    *change_content = format!("{}\n{}", Self::TO_JAPANESE_PREFIX, s);
                }
                Lang::Japanese(s) => match self.mode {
                    TranslateMode::ToKorean => {
                        *change_content = format!("{}\n{}", Self::TO_KOREAN_PREFIX, s);
                    }
                    TranslateMode::ToChinese => {
                        *change_content = format!("{}\n{}", Self::TO_CHINESE_PREFIX, s);
                    }
                    TranslateMode::ToEnglish => {
                        *change_content = format!("{}\n{}", Self::TO_ENGLISH_PREFIX, s);
                    }
                    TranslateMode::ToJapanese => {}
                },
            };
            message
        };
        input.to_messages().into_iter().map(add_prefix).collect()
    }
    fn can_action(&self) -> bool {
        true
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct FileTranslator {
    source_path: String,
    do_action: bool,
    inner: String,
}
impl Default for FileTranslator {
    fn default() -> Self {
        Self::new()
    }
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
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(self.source_path.as_str())?;
        let result = self
            .results()
            .split('。')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        file.write_all(format!("\n{}", result).as_bytes())
    }
    fn results(&self) -> &str {
        &self.inner
    }
}
impl GptFunction for FileTranslator {
    fn setup_for_action(&mut self, input: &super::UserInput) {
        if is_file_path(input.content()) {
            self.do_action = true;
            self.source_path = input.content().trim().to_string();
        }
    }
    fn can_action(&self) -> bool {
        self.do_action
    }
    fn input_to_messages(&self, input: super::UserInput) -> Vec<Message> {
        if self.can_action() {
            // can_action() == true
            // self.source_path is not empty and is file path
            // so we can get file content safely
            let content = get_file_content(&self.source_path).unwrap();
            UserInput::new(content)
                .to_messages()
                .into_iter()
                .map(|mut message| {
                    let content = message.change_content();
                    *content = format!("{}\n{}", Self::PREFIX, content);
                    message
                })
                .collect()
        } else {
            input.to_messages()
        }
    }
    fn action_at_end(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        if self.can_action() {
            self.append_result()?;
            *self = Self::new();
        }
        Ok(())
    }
    fn handle_stream(
        &mut self,
        response: &crate::gpt::client::ChatResponse,
    ) -> crate::gpt::client::HandleResult {
        if self.can_action() {
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
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Lang {
    English(String),
    Japanese(String),
}

impl<T: AsRef<str>> From<T> for Lang {
    fn from(s: T) -> Self {
        let is_english = s.as_ref().chars().all(|c| c.is_ascii());
        if is_english {
            Self::English(s.as_ref().to_string())
        } else {
            Self::Japanese(s.as_ref().to_string())
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        functions::{common::test_tool::TestFileFactory, GptFunction, UserInput},
        gpt::client::{ChatResponse, HandleResult, Message, Role},
    };

    use super::*;
    #[test]
    fn 翻訳を促すmessageに自動変換する_分割ver() {
        let input = UserInput::new("hello world.good bye");
        let sut = Translator::default();
        let messages = sut.input_to_messages(input);

        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[0],
            Message::new(
                Role::User,
                format!("{}\n{}", Translator::TO_JAPANESE_PREFIX, "hello world.")
            )
        );
        assert_eq!(
            messages[1],
            Message::new(
                Role::User,
                format!("{}\n{}", Translator::TO_JAPANESE_PREFIX, "good bye")
            )
        );
    }
    #[test]
    fn 翻訳を促すmessageに自動変換する() {
        let input = UserInput::new("hello");
        let sut = Translator::default();
        let messages = sut.input_to_messages(input);
        assert_eq!(
            messages[0],
            Message::new(
                Role::User,
                format!("{}\n{}", Translator::TO_JAPANESE_PREFIX, "hello")
            )
        );
        let jp_input = UserInput::new("helloは日本語でこんにちはです");
        let sut = Translator::default();
        let messages = sut.input_to_messages(jp_input);
        assert_eq!(
            messages[0],
            Message::new(
                Role::User,
                format!(
                    "{}\n{}",
                    Translator::TO_ENGLISH_PREFIX,
                    "helloは日本語でこんにちはです"
                )
            )
        );
    }
    #[test]
    fn 英語か日本語か判断可能() {
        let en = "hello ";
        assert_eq!(Lang::from(en), Lang::English(en.to_string()));

        let jp = "helloは日本語でこんにちは";
        assert_eq!(Lang::from(jp), Lang::Japanese(jp.to_string()));
    }
    #[test]
    #[ignore]
    fn inputがファイルのパスであればmessageをfileのコンテンツに翻訳の依頼を付与したものに変更する_文字数によっては分割を行う(
    ) {
        let test_file = TestFileFactory::create("tmp");
        test_file.create_root();
        // test GptLimit is 10(written in src/gpt/client.rs)
        test_file.create_file_under_root("test.rs", "hello world.good bye.i love you.");

        let input = UserInput::new("tmp/test.rs");

        let mut sut = FileTranslator::new();

        sut.setup_for_action(&input);
        let messages = sut.input_to_messages(input);

        test_file.remove_dir_all();

        assert_eq!(sut.can_action(), true);
        assert_eq!(messages.len(), 3);
        assert_eq!(
            messages[0].content,
            format!("{}\n{}", FileTranslator::PREFIX, "hello world.")
        );
        assert_eq!(
            messages[1].content,
            format!("{}\n{}", FileTranslator::PREFIX, "good bye.")
        );
        assert_eq!(
            messages[2].content,
            format!("{}\n{}", FileTranslator::PREFIX, "i love you.")
        );
    }
    #[test]
    #[ignore]
    fn inputがファイルのパスであればmessageをfileのコンテンツに翻訳の依頼を付与したものに変更する()
    {
        let test_file = TestFileFactory::create("tmp");
        test_file.create_root();
        test_file.create_file_under_root("test.rs", "hello");

        let input = UserInput::new("tmp/test.rs");

        let mut sut = FileTranslator::new();

        sut.setup_for_action(&input);
        let messages = sut.input_to_messages(input);

        test_file.remove_dir_all();

        assert_eq!(sut.can_action(), true);
        assert_eq!(
            messages[0].content,
            format!("{}\n{}", FileTranslator::PREFIX, "hello")
        );
    }

    #[test]
    #[ignore]
    fn fileのパスが来たらactionをonにする() {
        let test_file = TestFileFactory::create("tmp");
        test_file.create_root();
        test_file.create_file_under_root("test.rs", "hello");

        let input = UserInput::new("tmp/test.rs");

        let mut sut = FileTranslator::new();

        sut.setup_for_action(&input);

        test_file.remove_dir_all();

        assert_eq!(sut.can_action(), true);
    }
    #[test]
    #[ignore]
    fn gptのレスポンスが終了したら内部の結果をfileに追記し自身を初期化する() {
        let mut sut = FileTranslator::new();
        let test_file = TestFileFactory::create("tmp");
        test_file.create_file_under_root("hello.txt", "hello");

        let input = UserInput::new("tmp/hello.txt");

        sut.setup_for_action(&input);

        sut.handle_stream(&ChatResponse::DeltaContent("こん".to_string()));
        sut.handle_stream(&ChatResponse::DeltaContent("にちは".to_string()));
        sut.handle_stream(&ChatResponse::Done);

        sut.action_at_end().unwrap();

        let content = std::fs::read_to_string("tmp/hello.txt").unwrap();

        test_file.remove_dir_all();
        assert_eq!(content, "hello\nこんにちは");
        assert_eq!(sut.can_action(), false);
    }
    #[test]
    fn actionがoffであれば何もしない() {
        let input = UserInput::new("none");

        let mut sut = FileTranslator::new();
        sut.setup_for_action(&input);
        assert!(!sut.can_action());

        let messages = sut.input_to_messages(input);
        assert_eq!(messages[0].content, "none");

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
