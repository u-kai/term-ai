use std::io::Write;

use term_ai::gpt::GptClient;

fn main() {
    let mut gpt = GptClient::from_env().unwrap();
    loop {
        let mut message = String::new();
        print!("{} > ", std::env::var("USER").unwrap_or_default());
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut message).unwrap();
        print!("gpt > ");
        std::io::stdout().flush().unwrap();
        gpt.stream_chat(message, |res| {
            print!("{}", res);
            std::io::stdout().flush().unwrap();
            Ok(())
        })
        .unwrap();
        println!();
    }
}
