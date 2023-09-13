//#[cfg(target_os = "macos")]
//use crate::wrapper::speaker::Speaker;
use clap::{Parser, Subcommand};
//
//use crate::{
//    gpt::{GptClient, OpenAIModel},
//    repl::{GptMessageHandler, GptRepl},
//    wrapper::{
//        any::{AnyHandler, GptInput, Printer},
//        code_capture::{CodeCaptureGpt, SampleFileMaker},
//        code_reviewer::CodeReviewer,
//        first_command::FirstSystemCommand,
//        translator::{FileTranslator, TranslateWriter, Translator},
//    },
//};
#[derive(Parser)]
pub struct TermAi {
    #[clap(subcommand)]
    sub: Sub,
}

impl TermAi {
    pub fn new() -> Self {
        Self::parse()
    }
    pub fn run(&self) {
        //match &self.sub {
        //    Sub::Translate { path } => {
        //        let mut any = AnyHandler::new(GptClient::from_env().unwrap());
        //        let file_translator = FileTranslator::new();
        //        let printer = Printer::new();
        //        any.add_event_handler(Box::new(printer));
        //        any.add_input_convertor(Box::new(file_translator.clone()));
        //        any.add_response_handler(Box::new(file_translator.clone()));
        //        any.handle(GptInput::new(
        //            path,
        //            OpenAIModel::Gpt3Dot5Turbo,
        //            crate::gpt::Role::User,
        //        ))
        //        .unwrap();
        //    }
        //    Sub::Gpt3(option) => {
        //        Self::print_init("GPT3 REPL");
        //        let mut gpt = GptRepl::from_env(OpenAIModel::Gpt3Dot5Turbo).unwrap();
        //        self.set_option(&mut gpt, option);
        //        gpt.repl().unwrap();
        //    }
        //    Sub::EnglishTeacher(option) => {
        //        Self::print_wait_settings("GPT3 English Teacher");
        //        let first_command =
        //"今後私が記述する文章を英語に翻訳して，それぞれの部分がなぜそのように翻訳されたのかを日本語で詳しく説明してください";
        //        let mut gpt = GptRepl::new(FirstSystemCommand::from_env(first_command).unwrap());
        //        self.set_option(&mut gpt, option);
        //        gpt.repl().unwrap();
        //    }
        //    Sub::Review(option) => {
        //        Self::print_init("Code Reviewer");
        //        let mut gpt = GptRepl::new(CodeReviewer::from_env().unwrap());
        //        self.set_option(&mut gpt, option);
        //        gpt.repl().unwrap();
        //    }
        //    Sub::Capture(option) => {
        //        Self::print_wait_settings("Code Capture");
        //        let mut gpt =
        //            GptRepl::new(CodeCaptureGpt::from_env(SampleFileMaker::new()).unwrap());
        //        self.set_option(&mut gpt, option);
        //        gpt.repl().unwrap();
        //    }
        //    #[cfg(target_os = "macos")]
        //    Sub::Speaker(option) => {
        //        Self::print_init("Speaker");
        //        let mut gpt = GptRepl::new(Speaker::from_env().unwrap());
        //        self.set_option(&mut gpt, option);
        //        gpt.repl().unwrap();
        //    }
        //    Sub::Translator {
        //        your_display,
        //        ai_display,
        //        write_mode,
        //    } => {
        //        if *write_mode {
        //            Self::print_init("Translator");
        //            let mut gpt = GptRepl::new(TranslateWriter::from_env().unwrap());
        //            let option = CommandOption {
        //                ai_display: ai_display.clone(),
        //                your_display: your_display.clone(),
        //            };
        //            self.set_option(&mut gpt, &option);
        //            gpt.repl().unwrap();
        //        } else {
        //            Self::print_init("Translator");
        //            let mut gpt = GptRepl::new(Translator::from_env().unwrap());
        //            let option = CommandOption {
        //                ai_display: ai_display.clone(),
        //                your_display: your_display.clone(),
        //            };
        //            self.set_option(&mut gpt, &option);
        //            gpt.repl().unwrap();
        //        }
        //    }
        //    Sub::FirstSystemCommand {
        //        ai_display,
        //        your_display,
        //        first_command,
        //    } => {
        //        Self::print_init("First System Command");
        //        let option = CommandOption {
        //            ai_display: ai_display.clone(),
        //            your_display: your_display.clone(),
        //        };

        //        let first_command = if first_command.is_some() {
        //            first_command.clone().unwrap()
        //        } else {
        //            std::env::var("GPT_FIRST_COMMAND").unwrap()
        //        };
        //        println!("request ->  {}", first_command);
        //        let mut gpt = GptRepl::new(
        //            FirstSystemCommand::with_display_first_response(&first_command).unwrap(),
        //        );
        //        self.set_option(&mut gpt, &option);
        //        gpt.repl().unwrap();
        //    }
        //}
    }
    fn print_init(client: &str) {
        println!("Welcome to {} REPL", client);
    }
    fn print_wait_settings(client: &str) {
        Self::print_init(client);
        println!("connecting to GPT3...");
        println!("Please wait a few seconds...");
    }
    //fn set_option<E: std::error::Error, T: GptMessageHandler<E>>(
    //    &self,
    //    gpt: &mut GptRepl<E, T>,
    //    option: &CommandOption,
    //) {
    //    if let Some(your_display) = &option.your_display {
    //        gpt.set_user_name(your_display);
    //    }
    //    if let Some(ai_display) = &option.ai_display {
    //        gpt.set_gpt_display(ai_display);
    //    }
    //}
}

#[derive(Subcommand)]
enum Sub {
    #[clap(name = "translate", about = "Translate")]
    Translate {
        #[clap(short, long)]
        path: String,
    },
    Gpt3(CommandOption),
    #[clap(name = "trans", about = "Translator")]
    Translator {
        #[clap(short, long)]
        your_display: Option<String>,
        #[clap(short, long)]
        ai_display: Option<String>,
        #[clap(short, long)]
        write_mode: bool,
    },
    #[clap(name = "ent", about = "English Teacher")]
    EnglishTeacher(CommandOption),
    Review(CommandOption),
    #[clap(
        name = "capt",
        about = "capture your code and dist to sample_for_gpt_xxxxxx.LANG"
    )]
    Capture(CommandOption),
    #[clap(name = "first", about = "First System Command")]
    FirstSystemCommand {
        #[clap(short, long)]
        your_display: Option<String>,
        #[clap(short, long)]
        ai_display: Option<String>,
        #[clap(short, long)]
        first_command: Option<String>,
    },
    #[cfg(target_os = "macos")]
    #[clap(name = "speaker", about = "Speaker for macos")]
    Speaker(CommandOption),
}

#[derive(Debug, clap::Args)]
struct CommandOption {
    #[clap(short, long)]
    your_display: Option<String>,
    #[clap(short, long)]
    ai_display: Option<String>,
}
