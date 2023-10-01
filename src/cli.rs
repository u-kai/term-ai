#[cfg(target_os = "macos")]
use crate::functions::speaker::MacSpeaker;
use crate::{
    functions::{
        code_capture::GptCodeCapture,
        code_reviewer::CodeReviewer,
        repl::ChatGptRepl,
        translator::{FileTranslator, TranslateMode, Translator},
        GptFunction, GptFunctionContainer,
    },
    gpt::{
        chat::ChatGpt,
        client::{Message, OpenAIModel, Role},
    },
};
use clap::Parser;
use std::{io::Write, str::FromStr};
#[derive(Parser)]
pub struct Gpt {
    #[clap(short = 'c', long = "code-capture")]
    code_capture: bool,
    #[clap(short = 'r', long = "code-reviewer")]
    code_reviewer: bool,
    #[clap(short = 'f', long = "file-translator")]
    file_translator: bool,
    #[clap(short = 'e', long = "english-teacher")]
    english_teacher: bool,
    #[clap(long = "translator")]
    translator: Option<TranslateMode>,
    #[clap(short = 't', long = "translate")]
    translate: bool,
    #[clap(short = 'p', long = "repl")]
    repl: bool,
    #[clap(short = 'v', long = "gpt", default_value = "gpt3")]
    gpt: GptVersion,
    #[cfg(target_os = "macos")]
    #[clap(short = 's', long = "speaker")]
    speaker: bool,
    source: Option<String>,
}
impl FromStr for TranslateMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Self::ToEnglish),
            "ko" => Ok(Self::ToKorean),
            "ch" => Ok(Self::ToChinese),
            _ => Err(format!("{} is not supported", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GptVersion {
    Gpt3,
    Gpt4,
}
impl FromStr for GptVersion {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gpt3" | "3" => Ok(Self::Gpt3),
            "gpt4" | "4" => Ok(Self::Gpt4),
            _ => Err(format!("{} is not supported", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[allow(non_snake_case)]
    fn cliのoptionからGptFunctionContainerを生成できる() {
        let termai = Gpt::parse_from(&["gpt", "-c", "-r", "-f", "-t", "-v", "4"]);
        let container = termai.gen_functions();
    }
    #[test]
    fn cliはcode_capture機能を利用するか選択できる() {}
    #[cfg(target_os = "macos")]
    #[test]
    fn cliはspeaker機能を利用するか選択できる() {}
    #[test]
    fn cliはcode_reviewer機能を利用するか選択できる() {}
    #[test]
    fn cliはfile_translator機能を利用するか選択できる() {}
    #[test]
    fn cliは翻訳機能を利用するか選択できる() {}
    #[test]
    fn cliはreplか選択できる() {}
    #[test]
    fn cliはgpt3か選択できる() {}
    #[test]
    fn cliはgpt4か選択できる() {}
}

impl Default for Gpt {
    fn default() -> Self {
        Self::new()
    }
}

impl Gpt {
    pub fn new() -> Self {
        Self::parse()
    }
    fn gen_functions(&self) -> GptFunctionContainer {
        let mut result = GptFunctionContainer::new();
        if self.code_capture {
            result.add_functions(Box::new(GptCodeCapture::new_with_file_writer(".")));
        };
        if self.code_reviewer {
            result.add_functions(Box::new(CodeReviewer::default()));
        };
        if self.file_translator {
            result.add_functions(Box::new(FileTranslator::default()));
        };
        if self.speaker {
            #[cfg(target_os = "macos")]
            result.add_functions(Box::new(MacSpeaker::default()));
        }
        if self.english_teacher {
            result.add_functions(Box::new(Translator::default()));
        }
        if let Some(mode) = &self.translator {
            result.add_functions(Box::new(Translator::new(mode.clone())));
        }
        result
    }
    pub fn run(&self) {
        let mut gpt = ChatGpt::from_env().unwrap();
        let model = if self.gpt == GptVersion::Gpt3 {
            OpenAIModel::Gpt3Dot5Turbo
        } else {
            OpenAIModel::Gpt4
        };
        let mut functions = self.gen_functions();
        if self.repl {
            let mut repl = ChatGptRepl::new_with_functions(gpt, functions);
            match repl.repl(model) {
                Ok(_) => {}
                Err(e) => {
                    println!("GPT ERROR : {}", e.to_string());
                    println!("Re Run GPT REPL");
                    self.run();
                }
            }
        } else {
            let mut message = self
                .make_message()
                .expect("gpt source is not found, so you want to use gpt, you must set argument");
            functions.switch_do_action(&message);
            functions.change_request(&mut message);
            gpt.chat(model, message, &mut |res| {
                print!("{}", res.delta_content());
                std::io::stdout().flush().unwrap();
                functions.handle_stream(res)
            })
            .unwrap();
            functions.action_at_end().unwrap();
        }
    }
    fn make_message(&self) -> Result<Message, String> {
        if self.translate {
            Ok(Message::new(
                Role::User,
                format!(
                    "以下を日本語に翻訳してください\n {}",
                    self.source.as_ref().unwrap_or(&String::new())
                ),
            ))
        } else {
            Ok(Message::new(
                Role::User,
                self.source.as_ref().unwrap_or(&String::new()),
            ))
        }
    }
}
