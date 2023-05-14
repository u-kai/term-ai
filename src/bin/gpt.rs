use std::io::Write;

use term_ai::gpt::GptClient;

#[tokio::main]
async fn main() {
    let gpt = GptClient::from_env().unwrap();
    loop {
        let mut message = String::new();
        print!("{} > ", std::env::var("USER").unwrap_or_default());
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut message).unwrap();
        gpt.chat(message).await.unwrap();
    }
}
