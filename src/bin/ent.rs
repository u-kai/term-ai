use term_ai::{repl::GptRepl, wrapper::first_command::FirstSystemCommand};

fn main() {
    println!("Welcome to GPT3 English Teacher REPL");
    let first_command =
        "今後私が記述する文章を英語に翻訳して，それぞれの部分がなぜそのように翻訳されたのかを日本語で詳しく説明してください";
    let mut gpt = GptRepl::new(FirstSystemCommand::from_env(first_command).unwrap());
    gpt.repl().unwrap();
}
