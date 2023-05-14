use term_ai::gpt::GptClient;

fn main() {
    let gpt = GptClient::from_env().unwrap();
    loop {
        let mut message = String::new();
        std::io::stdin().read_line(&mut message).unwrap();
        //gpt.chat(message)unwrap();
    }
}
