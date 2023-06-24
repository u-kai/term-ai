use std::fs::File;

use term_ai::{repl::GptRepl, wrapper::code_capture::CodeCaptureGpt};

fn main() {
    let mut gpt = GptRepl::new(CodeCaptureGpt::from_env(File::create("test.py").unwrap()).unwrap());
    gpt.repl().unwrap();
}
