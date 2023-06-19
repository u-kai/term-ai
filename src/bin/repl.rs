use term_ai::{
    gpt::OpenAIModel,
    repl::{GptChat, GptRepl},
};

fn main() {
    let mut gpt = GptRepl::new(GptChat::from_env(OpenAIModel::Gpt3Dot5Turbo).unwrap());
    gpt.repl().unwrap();
}
