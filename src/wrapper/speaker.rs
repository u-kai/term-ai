#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(target_os = "macos")]
use crate::{
    gpt::{GptClient, GptClientError},
    repl::GptMessageHandler,
};

#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct Speaker {
    model: crate::gpt::OpenAIModel,
    client: GptClient,
}

#[cfg(target_os = "macos")]
impl GptMessageHandler<GptClientError> for Speaker {
    fn clear_history(&mut self) {
        self.client.clear_history();
    }
    fn handle<F>(&mut self, message: &str, f: &F) -> Result<(), GptClientError>
    where
        F: Fn(&str),
    {
        self.say(message, f)
    }
}

#[cfg(target_os = "macos")]
impl Speaker {
    pub fn from_env() -> Result<Self, crate::gpt::GptClientError> {
        let client = GptClient::from_env()?;
        let model = crate::gpt::OpenAIModel::Gpt3Dot5Turbo;
        Ok(Self { client, model })
    }
    pub fn say<F: Fn(&str)>(
        &mut self,
        message: &str,
        f: &F,
    ) -> Result<(), crate::gpt::GptClientError> {
        let result = self
            .client
            .chat(self.model, crate::gpt::Role::System, message, f)?;
        multi_lang_say(result.as_str());
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn multi_lang_say(message: &str) {
    MultiLangSentence::from(message)
        .iter()
        .for_each(|s| match s {
            MultiLang::Japanese(s) => say_command(s, &MacSayCommandSpeaker::Kyoko),
            MultiLang::English(s) => say_command(s, &MacSayCommandSpeaker::Karen),
        });
}
#[cfg(target_os = "macos")]
fn say_command(message: &str, speaker: &MacSayCommandSpeaker) {
    let result = Command::new("say")
        .args(["-v", speaker.to_name(), message])
        .output()
        .unwrap();
    if !result.status.success() {
        panic!("failed to execute process: {}", result.status);
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

#[derive(Debug, Clone, PartialEq)]
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
    pub fn to_string(&self) -> String {
        self.inner
            .iter()
            .map(|lang| match lang {
                MultiLang::English(s) => s.to_string(),
                MultiLang::Japanese(s) => s.to_string(),
            })
            .collect::<Vec<String>>()
            .join("")
    }
}

#[derive(Debug, Clone, PartialEq)]
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
mod tests {
    use super::*;
    #[test]
    fn test_英語と日本語が混ざった文章を構造体にする() {
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
