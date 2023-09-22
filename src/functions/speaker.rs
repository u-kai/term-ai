use std::fmt::Display;
#[cfg(target_os = "macos")]
use std::process::Command;

use super::GptFunction;

#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct MacSpeaker {
    inner: String,
}
impl MacSpeaker {
    pub fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }
    fn results(&self) -> &str {
        &self.inner
    }
}
impl GptFunction for MacSpeaker {
    fn handle_stream(
        &mut self,
        response: &crate::gpt::client::ChatResponse,
    ) -> crate::gpt::client::HandleResult {
        match response {
            crate::gpt::client::ChatResponse::DeltaContent(content) => {
                self.inner.push_str(content.as_str());
                crate::gpt::client::HandleResult::Progress
            }
            crate::gpt::client::ChatResponse::Done => crate::gpt::client::HandleResult::Done,
        }
    }
    fn action_at_end(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        multi_lang_say(self.results())?;
        *self = Self::new();
        Ok(())
    }
}
#[cfg(target_os = "macos")]
fn multi_lang_say(message: &str) -> Result<(), std::io::Error> {
    MultiLangSentence::from(message)
        .iter()
        .for_each(|s| match s {
            MultiLang::Japanese(s) => say_command(s, &MacSayCommandSpeaker::Kyoko).unwrap(),
            MultiLang::English(s) => say_command(s, &MacSayCommandSpeaker::Karen).unwrap(),
        });
    Ok(())
}
#[cfg(target_os = "macos")]
fn say_command(message: &str, speaker: &MacSayCommandSpeaker) -> Result<(), std::io::Error> {
    let result = Command::new("say")
        .args(["-v", speaker.to_name(), message])
        .output()?;
    if !result.status.success() {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "say command failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
        ))
    } else {
        Ok(())
    }
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub enum MacSayCommandSpeaker {
    Karen,
    Tessa,
    Kyoko,
}
#[cfg(target_os = "macos")]

impl MacSayCommandSpeaker {
    fn to_name(&self) -> &'static str {
        match self {
            Self::Karen => "Karen",
            Self::Tessa => "Tessa",
            Self::Kyoko => "Kyoko",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiLangSentence {
    inner: Vec<MultiLang>,
}

impl MultiLangSentence {
    pub fn from(src: &str) -> Self {
        let mut inner = vec![];
        let mut chars = src.chars();
        let first = chars.next();
        let Some(first) = first else {
            return Self { inner };
        };
        inner.push(MultiLang::from_char(first));
        chars.for_each(|c| {
            if inner.last().unwrap().same_lang(c) {
                inner.last_mut().unwrap().add_char(c);
            } else {
                inner.push(MultiLang::from_char(c));
            };
        });
        Self { inner }
    }
    pub fn iter(&self) -> impl Iterator<Item = &MultiLang> {
        self.inner.iter()
    }
}
impl Display for MultiLangSentence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .inner
            .iter()
            .map(|lang| match lang {
                MultiLang::English(s) => s.to_string(),
                MultiLang::Japanese(s) => s.to_string(),
            })
            .collect::<Vec<String>>()
            .join("");
        write!(f, "{}", s)
    }
}
impl Default for MacSpeaker {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultiLang {
    English(String),
    Japanese(String),
}

impl MultiLang {
    fn from_char(c: char) -> Self {
        if c.is_ascii() {
            Self::English(c.to_string())
        } else {
            Self::Japanese(c.to_string())
        }
    }
    fn same_lang(&self, c: char) -> bool {
        match self {
            Self::English(_) => c.is_ascii(),
            Self::Japanese(_) => !c.is_ascii(),
        }
    }
    fn add_char(&mut self, c: char) {
        match self {
            Self::English(s) => s.push(c),
            Self::Japanese(s) => s.push(c),
        }
    }
}

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
    use crate::gpt::client::ChatResponse;

    use super::*;
    #[test]
    #[ignore]
    fn action_endでsayコマンドを発行して内部状態を初期化する() {
        let mut sut = MacSpeaker::new();
        sut.handle_stream(&ChatResponse::DeltaContent("hello".to_string()));
        sut.handle_stream(&ChatResponse::DeltaContent("こんにちは".to_string()));
        sut.handle_stream(&ChatResponse::Done);

        sut.action_at_end().unwrap();

        assert_eq!(sut.results(), "");
    }
    #[test]
    fn sseのレスポンスを内部に格納する() {
        let mut sut = MacSpeaker::new();

        sut.handle_stream(&ChatResponse::DeltaContent("hello".to_string()));
        sut.handle_stream(&ChatResponse::DeltaContent("こんにちは".to_string()));
        sut.handle_stream(&ChatResponse::Done);

        assert_eq!(sut.results(), "helloこんにちは");
    }
    #[test]
    fn 英語と日本語が混ざった文章を構造体にする() {
        let src = "Hello,こんにちは,what time is it now? 今何時？";
        let expected = MultiLangSentence {
            inner: vec![
                MultiLang::English("Hello,".to_string()),
                MultiLang::Japanese("こんにちは".to_string()),
                MultiLang::English(",what time is it now? ".to_string()),
                MultiLang::Japanese("今何時？".to_string()),
            ],
        };
        assert_eq!(expected, MultiLangSentence::from(src));
        assert_eq!(src, MultiLangSentence::from(src).to_string());
    }
}
