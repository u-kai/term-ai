#[cfg(target_os = "macos")]
use crate::functions::speaker::MacSpeaker;
use crate::{
    functions::{
        code_capture::GptCodeCapture,
        code_reviewer::CodeReviewer,
        repl::ChatGptRepl,
        translator::{FileTranslator, TranslateMode, Translator},
        GptFunction, UserInput,
    },
    gpt::{
        chat::ChatGpt,
        client::{ChatRequest, GptClient, Message, OpenAIModel, Role},
    },
};
use clap::{Parser, Subcommand};
use std::{io::Write, str::FromStr, thread::sleep, time::Duration};

#[derive(Parser)]
pub struct TermAI {
    #[clap(subcommand)]
    subcommand: SubCommands,
}

#[derive(Subcommand)]
enum SubCommands {
    Chat {
        #[clap(short = 'v', long = "gpt-version", default_value = "gpt3")]
        gpt_version: GptVersion,
        #[clap(short = 'c', long = "code-capture", default_value = "false")]
        code_capture: bool,
        #[clap(short = 'r', long = "code-reviewer", default_value = "false")]
        code_reviewer: bool,
        #[clap(short = 't', long = "translator")]
        translator: Option<TranslateMode>,
        #[clap(short = 's', long = "speaker", default_value = "false")]
        speaker: bool,
    },
    Speaker {
        #[clap(short = 'v', long = "gpt-version", default_value = "gpt3")]
        gpt_version: GptVersion,
        source: String,
    },
    #[clap(name = "tjp")]
    TranslatorJp {
        #[clap(short = 'v', long = "gpt-version", default_value = "gpt3")]
        gpt_version: GptVersion,
        #[clap(short = 'f', long = "file-source")]
        file_path: Option<String>,
        source: Option<String>,
    },
    #[clap(name = "ten")]
    TranslatorEn {
        #[clap(short = 'v', long = "gpt-version", default_value = "gpt3")]
        gpt_version: GptVersion,
        #[clap(short = 'f', long = "file-source")]
        file_path: Option<String>,
        source: Option<String>,
    },
    #[clap(name = "cc")]
    CodeCapture {
        #[clap(short = 'v', long = "gpt-version", default_value = "gpt3")]
        gpt_version: GptVersion,
        source: String,
    },
    #[clap(name = "cr")]
    CodeReviewer {
        #[clap(short = 'v', long = "gpt-version", default_value = "gpt3")]
        gpt_version: GptVersion,
        file_path: String,
    },
}

fn exec_with_function(
    client: &mut GptClient,
    model: OpenAIModel,
    input: UserInput,
    f: &mut impl GptFunction,
) {
    fn display_result_and_handle_stream(
        client: &mut GptClient,
        f: &mut impl GptFunction,
        req: ChatRequest,
    ) -> crate::gpt::client::Result<()> {
        client.request_mut_fn(req, |res| {
            print!("{}", res.delta_content());
            std::io::stdout().flush().unwrap();
            f.handle_stream(res)
        })
    }
    fn retry_request(
        client: &mut GptClient,
        req: ChatRequest,
        f: &mut impl GptFunction,
    ) -> crate::gpt::client::Result<()> {
        client.re_connect()?;
        sleep(Duration::from_secs(1));
        display_result_and_handle_stream(client, f, req.clone())
    }
    f.setup_for_action(&input);
    let messages = f.input_to_messages(input);
    messages.into_iter().for_each(|message| {
        let req = ChatRequest::from_message(model, message);
        display_result_and_handle_stream(client, f, req.clone())
            .or_else(|_e| retry_request(client, req.clone(), f))
            .or_else(|_e| retry_request(client, req.clone(), f))
            .or_else(|e| {
                f.action_at_end().unwrap();
                Err(e)
            })
            .unwrap();
    });
    f.action_at_end().unwrap();
}

impl TermAI {
    pub fn new() -> Self {
        Self::parse()
    }

    pub fn run(&self) {
        match &self.subcommand {
            SubCommands::Chat {
                gpt_version,
                code_capture,
                code_reviewer,
                translator,
                speaker,
            } => {
                let mut repl = ChatGptRepl::new();
                if *code_capture {
                    repl.add_functions(Box::new(GptCodeCapture::new_with_file_writer(".")));
                };
                if *code_reviewer {
                    repl.add_functions(Box::new(CodeReviewer::default()));
                };
                if *speaker {
                    #[cfg(target_os = "macos")]
                    repl.add_functions(Box::new(MacSpeaker::default()));
                }
                if let Some(mode) = translator.as_ref() {
                    repl.add_functions(Box::new(Translator::new(mode.clone())));
                }
                if *gpt_version == GptVersion::Gpt3 {
                    repl.repl_gpt3().unwrap();
                } else {
                    repl.repl_gpt4().unwrap();
                }
            }
            SubCommands::Speaker {
                gpt_version,
                source,
            } => {
                let mut gpt = ChatGpt::from_env().unwrap();
                let model = if *gpt_version == GptVersion::Gpt3 {
                    OpenAIModel::Gpt3Dot5Turbo
                } else {
                    OpenAIModel::Gpt4
                };
                let mut function = MacSpeaker::default();
                let mut message = Message::new(Role::User, source);
                function.switch_do_action(&message);
                function.change_request(&mut message);
                gpt.chat(model, &message, &mut |res| {
                    print!("{}", res.delta_content());
                    std::io::stdout().flush().unwrap();
                    function.handle_stream(res)
                })
                .unwrap();
                function.action_at_end().unwrap();
            }
            SubCommands::CodeCapture {
                gpt_version,
                source,
            } => {
                let mut gpt = ChatGpt::from_env().unwrap();
                let model = if *gpt_version == GptVersion::Gpt3 {
                    OpenAIModel::Gpt3Dot5Turbo
                } else {
                    OpenAIModel::Gpt4
                };
                let mut function = GptCodeCapture::new_with_file_writer(".");
                let mut message = Message::new(Role::User, source);
                function.switch_do_action(&message);
                function.change_request(&mut message);
                gpt.chat(model, &message, &mut |res| {
                    print!("{}", res.delta_content());
                    std::io::stdout().flush().unwrap();
                    function.handle_stream(res)
                })
                .unwrap();
                function.action_at_end().unwrap();
            }
            SubCommands::TranslatorJp {
                gpt_version,
                file_path,
                source,
            } => {
                let mut gpt = ChatGpt::from_env().unwrap();
                let model = if *gpt_version == GptVersion::Gpt3 {
                    OpenAIModel::Gpt3Dot5Turbo
                } else {
                    OpenAIModel::Gpt4
                };
                if let Some(file_path) = file_path.as_ref() {
                    let mut function = FileTranslator::default();
                    let mut client = GptClient::from_env().unwrap();
                    let input = UserInput::new(file_path);
                    exec_with_function(&mut client, model, input, &mut function)
                } else {
                    let mut function = Translator::new(TranslateMode::ToJapanese);
                    let mut message =
                        Message::new(Role::User, source.as_ref().expect("source is required"));
                    function.switch_do_action(&message);
                    function.change_request(&mut message);
                    gpt.chat(model, &message, &mut |res| {
                        print!("{}", res.delta_content());
                        std::io::stdout().flush().unwrap();
                        function.handle_stream(res)
                    })
                    .unwrap();
                    function.action_at_end().unwrap();
                };
            }
            SubCommands::TranslatorEn {
                gpt_version,
                file_path,
                source,
            } => {
                let model = if *gpt_version == GptVersion::Gpt3 {
                    OpenAIModel::Gpt3Dot5Turbo
                } else {
                    OpenAIModel::Gpt4
                };

                if let Some(file_path) = file_path.as_ref() {
                    let mut function = FileTranslator::default();
                    let mut client = GptClient::from_env().unwrap();
                    let input = UserInput::new(file_path);
                    exec_with_function(&mut client, model, input, &mut function)
                } else {
                    let mut gpt = ChatGpt::from_env().unwrap();
                    let mut function = Translator::new(TranslateMode::ToEnglish);
                    let mut message =
                        Message::new(Role::User, source.as_ref().expect("source is required"));
                    function.switch_do_action(&message);
                    function.change_request(&mut message);
                    gpt.chat(model, &message, &mut |res| {
                        print!("{}", res.delta_content());
                        std::io::stdout().flush().unwrap();
                        function.handle_stream(res)
                    })
                    .unwrap();
                    function.action_at_end().unwrap();
                };
            }
            SubCommands::CodeReviewer {
                gpt_version,
                file_path,
            } => {
                let mut gpt = ChatGpt::from_env().unwrap();
                let model = if *gpt_version == GptVersion::Gpt3 {
                    OpenAIModel::Gpt3Dot5Turbo
                } else {
                    OpenAIModel::Gpt4
                };
                let mut function = CodeReviewer::default();
                let mut message = Message::new(Role::User, file_path);
                function.switch_do_action(&message);
                function.change_request(&mut message);
                gpt.chat(model, &message, &mut |res| {
                    print!("{}", res.delta_content());
                    std::io::stdout().flush().unwrap();
                    function.handle_stream(res)
                })
                .unwrap();
                function.action_at_end().unwrap();
            }
        }
    }
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
