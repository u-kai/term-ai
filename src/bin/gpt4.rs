use term_ai::{
    functions::repl::ChatGptRepl,
    gpt::client::{ChatResponse, HandleResult, OpenAIModel},
};

fn main() {
    println!("Welcome to GPT4 REPL");
    let mut repl = ChatGptRepl::new();
    let mut s = String::new();
    repl.repl(OpenAIModel::Gpt4, &mut |res| match res {
        ChatResponse::DeltaContent(content) => {
            s.push_str(&content);
            HandleResult::Progress
        }
        ChatResponse::Done => HandleResult::Done,
    })
    .unwrap();
    println!("{:#?}", s);
}
