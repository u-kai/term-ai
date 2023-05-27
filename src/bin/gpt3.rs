use std::io::Write;

use term_ai::gpt::GptClient;

fn main() {
    println!("Welcome to GPT3 REPL");
    let mut gpt = GptClient::from_env().unwrap();
    gpt.repl_gpt3_5().unwrap();
    //loop {
    //let mut message = String::new();
    //print!("{} > ", std::env::var("USER").unwrap_or("you".to_string()));
    //std::io::stdout().flush().unwrap();
    //std::io::stdin().read_line(&mut message).unwrap();
    //print!("gpt > ");
    //std::io::stdout().flush().unwrap();
    //let result = gpt
    //.gpt3_5(&message, &|event| {
    //print!("{}", event);
    //std::io::stdout().flush().unwrap();
    //})
    //.unwrap();
    //println!("{}", result);
    //}
}
