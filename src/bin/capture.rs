use term_ai::wrapper::code_capture::CodeCaptureGpt;

fn main() {
    let mut gpt = CodeCaptureGpt::from_env().unwrap();
    gpt.repl().unwrap();
}
