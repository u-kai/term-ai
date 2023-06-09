use term_ai::wrapper::code_reviewer::CodeReviewer;

fn main() {
    println!("Welcome to GPT3 REPL");
    let mut gpt = CodeReviewer::from_env().unwrap();
    gpt.repl_gpt3_5().unwrap();
}
