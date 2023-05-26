use std::io::Write;

use term_ai::gpt::GptClient;

fn main() {
    println!("Welcome to GPT3 REPL");
    let mut gpt = GptClient::from_env().unwrap();
    loop {
        let mut message = String::new();
        print!("{} > ", std::env::var("USER").unwrap_or("you".to_string()));
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut message).unwrap();
        print!("gpt > ");
        std::io::stdout().flush().unwrap();
        let result = gpt
            .chat(&message, &|event| {
                print!("{}", event);
                std::io::stdout().flush().unwrap();
            })
            .unwrap();
        println!("{}", result);
    }
    //match gpt.chat(&message, |res| {
    //println!("{}", res);
    ////std::io::stdout().flush().unwrap();
    //Ok(())
    //}) {
    //Ok(()) => {
    //println!();
    //continue;
    //}
    //Err(e) => {
    //err_num += 1;
    //println!("network error");
    //println!("request is : {}", message);
    //if err_num > 3 {
    //panic!("{:#?}", e);
    //}
    //println!("retrying...");
    //continue;
    //}
    //}
    //}
}
