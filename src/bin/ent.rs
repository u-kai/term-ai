use term_ai::gpt::GptClient;

fn main() {
    println!("Welcome to GPT3 English Teacher REPL");
    let mut gpt = GptClient::from_env().unwrap();
    let message =
        String::from("あなたは英語の先生です.今後全ての私の発言を英語に翻訳して,翻訳した結果をなぜその英訳になったのか,日本語で解説してください.");
    gpt.repl_gpt3_5_with_first_command(message.as_str())
        .unwrap();
}
