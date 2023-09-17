use term_ai::{
    functions::repl::ChatGptRepl,
    gpt::client::{ChatResponse, HandleResult, OpenAIModel},
};

fn main() {
    println!("Welcome to GPT4 REPL");
    let mut repl = ChatGptRepl::new();
    let mut s = String::new();
    repl.repl_gpt4().unwrap();
    println!("{:#?}", s);
}
