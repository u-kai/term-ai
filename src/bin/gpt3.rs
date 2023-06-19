use term_ai::{gpt::OpenAIModel, repl::GptRepl};

fn main() {
    println!("Welcome to GPT3 REPL");
    let mut gpt = GptRepl::from_env(OpenAIModel::Gpt3Dot5Turbo).unwrap();
    gpt.repl().unwrap();
}
