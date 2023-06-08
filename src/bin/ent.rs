use term_ai::gpt::GptClient;

fn main() {
    println!("Welcome to GPT3 English Teacher REPL");
    let mut gpt = GptClient::from_env().unwrap();
    let message =
        String::from("今後私が記述する文章を英語に翻訳して，それぞれの部分がなぜそのように翻訳されたのかを日本語で詳しく説明してください");
    gpt.repl_gpt3_5_with_first_command(message.as_str())
        .unwrap();
}
