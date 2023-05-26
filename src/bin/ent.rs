use std::io::Write;

use term_ai::gpt::GptClient;

fn main() {
    println!("Welcome to GPT3 English Teacher REPL");
    let mut gpt = GptClient::from_env().unwrap();
    let mut message = String::from(
        "あなたは日本語ができる英語の先生です．次からの私の発言全てを英語に翻訳して,その翻訳結果について日本語で解説してください.",
    );
    println!("setting gpt3...");
    gpt.chat(&message, &|_event| {}).unwrap();
    println!("start conversation");
    loop {
        print!("{} > ", std::env::var("USER").unwrap_or("you".to_string()));
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut message).unwrap();
        print!("gpt > ");
        std::io::stdout().flush().unwrap();
        gpt.chat(&message, &|event| {
            print!("{}", event);
            std::io::stdout().flush().unwrap();
        })
        .unwrap();
        println!();
        message.clear();
    }
}
