use term_ai::gpt::GptClient;

fn main() {
    println!("Welcome to GPT3 REPL");
    let mut gpt = GptClient::from_env().unwrap();
    gpt.repl_gpt3_5().unwrap();
}
