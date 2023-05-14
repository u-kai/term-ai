use std::io::Write;

use term_ai::gpt::{GptClient, OpenAIModel};

fn main() {
    let mut gpt = GptClient::from_env().unwrap();
    gpt.change_model(OpenAIModel::Gpt4);
    let mut err_num = 0;
    loop {
        let mut message = String::new();
        print!("{} > ", std::env::var("USER").unwrap_or_default());
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut message).unwrap();
        print!("gpt > ");
        std::io::stdout().flush().unwrap();
        match gpt.stream_chat(&message, |res| {
            print!("{}", res);
            std::io::stdout().flush().unwrap();
            Ok(())
        }) {
            Ok(()) => {
                println!();
                continue;
            }
            Err(e) => {
                err_num += 1;
                println!("network error");
                println!("request is : {}", message);
                if err_num > 3 {
                    panic!("{:#?}", e);
                }
                println!("retrying...");
                continue;
            }
        }
    }
}
