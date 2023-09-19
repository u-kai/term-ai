#[cfg(target_os = "macos")]
use crate::functions::speaker::MacSpeaker;
use crate::{
    functions::{
        code_capture::GptCodeCapture, code_reviewer::CodeReviewer, repl::ChatGptRepl,
        translator::FileTranslator, GptFunction, GptFunctionContainer,
    },
    gpt::{
        chat::ChatGpt,
        client::{HandleResult, Message, OpenAIModel, Role},
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

#[derive(Debug, Clone, Copy, PartialEq)]
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
            result.add_functions(Box::new(CodeReviewer::new(".")));
        };
        if self.file_translator {
            result.add_functions(Box::new(FileTranslator::new()));
        };
        if self.speaker {
            #[cfg(target_os = "macos")]
            result.add_functions(Box::new(MacSpeaker::new()));
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
        if self.english_teacher {
            gpt.chat(
                model,
                Message::new(
                    Role::User,
                    "これから私が記述する全ての英語を日本語でわかりやすく翻訳してください.",
                ),
                &mut |res| HandleResult::from(res),
            )
            .unwrap();
        }
        let mut functions = self.gen_functions();
        if self.repl {
            let mut repl = ChatGptRepl::new_with_functions(gpt, functions);
            repl.repl(model);
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
    fn print_init(client: &str) {
        println!("Welcome to {} REPL", client);
    }
    fn print_wait_settings(client: &str) {
        Self::print_init(client);
        println!("connecting to GPT3...");
        println!("Please wait a few seconds...");
    }
}
