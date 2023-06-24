use term_ai::{
    repl::GptRepl,
    wrapper::code_capture::{CodeCaptureGpt, SampleFileMaker},
};

fn main() {
    let mut gpt = GptRepl::new(CodeCaptureGpt::from_env(SampleFileMaker::new()).unwrap());
    gpt.repl().unwrap();
}
