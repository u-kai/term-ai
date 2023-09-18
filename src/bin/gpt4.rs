use term_ai::{
    functions::{code_capture::GptCodeCapture, code_reviewer::CodeReviewer, repl::ChatGptRepl},
    gpt::client::{ChatResponse, HandleResult, OpenAIModel},
};

fn main() {
    println!("Welcome to GPT4 REPL");
    let mut repl = ChatGptRepl::new();
    repl.add_functions(Box::new(CodeReviewer::default()));
    repl.add_functions(Box::new(GptCodeCapture::new_with_file_writer(".")));
    let mut s = String::new();
    repl.repl_gpt4().unwrap();
    println!("{:#?}", s);
}
