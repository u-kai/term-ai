use term_ai::{repl::GptRepl, wrapper::code_reviewer::CodeReviewer};

fn main() {
    println!("Welcome to GPT3 REPL");
    let mut gpt = GptRepl::new(CodeReviewer::from_env().unwrap());
    gpt.repl().unwrap();
}
